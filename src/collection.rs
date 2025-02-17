use crate::analysis::collection_analysis;
use crate::cache::*;
use crate::error::AppError;
use crate::models::AnalysisDifference;
use crate::models::Nml;
use crate::models::Node;
use crate::progress::ProgressBar;
use clap::ArgMatches;
use directories::ProjectDirs;
use indicatif::ProgressStyle;
use log::{info, trace, warn};
use parking_lot::Mutex;
use quick_xml::de::from_reader;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::fs::create_dir_all;
use std::fs::{copy, File};
use std::io::Cursor;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

static DEFAULT_CACHE_FILE_NAME: &str = "dj-library-gain-calculator.db";

pub fn run(matches: &ArgMatches) -> Result<(), AppError> {
    let input_path = matches.get_one::<String>("input").ok_or("no input provided")?;
    let target_loudness: f32 = matches
        .get_one::<String>("target")
        .ok_or("no target loudness provided")?
        .parse()?;

    let temp_dir = TempDir::new()?;
    let output_temp_path = temp_dir.path().join("collection.nml");
    let output_stream = output_stream(matches, &output_temp_path)?;

    // try to find the cache
    let maybe_cache_file = matches.get_one::<String>("cache-file");
    let cache_file = match maybe_cache_file {
        Some(d) => PathBuf::from(d),
        None => match ProjectDirs::from("org", "rlustin", "dj-library-gain-calculator") {
            Some(d) => {
                match create_dir_all(d.cache_dir()) {
                    Ok(_) => info!("Creating directory hierarchy {}", d.cache_dir().display()),
                    Err(e) => info!(
                        "Could not create directory hierarchy {}, ({})",
                        d.cache_dir().display(),
                        e
                    ),
                }
                d.cache_dir().join(DEFAULT_CACHE_FILE_NAME)
            }
            None => {
                warn!("Can't find cache dir, using pwd");
                let mut p = PathBuf::from("./");
                p.push(DEFAULT_CACHE_FILE_NAME);
                p
            }
        },
    };

    info!("Database path: {}", cache_file.display());

    let mut flags = CachePolicy::empty();
    if matches.contains_id("no-cache-read") {
        flags |= CachePolicy::NO_READ;
    }
    if matches.contains_id("no-cache-write") {
        flags |= CachePolicy::NO_WRITE;
    }
    if matches.contains_id("purge-cache") {
        flags |= CachePolicy::PURGE;
    }
    let maybe_cache = Cache::new(&cache_file, flags)?;
    let cache = Arc::new(Mutex::new(maybe_cache));

    let mut nml = deserialize_collection(input_path)?;

    let progress_bar = ProgressBar::new(nml.track_count());
    progress_bar.set_style(ProgressStyle::default_bar().template(
        "{bar:60.cyan/blue} {pos:>5}/{len:5} [{elapsed_precise}/{eta_precise}] {wide_msg}",
    ));
    progress_bar.enable_steady_tick(500 /* ms */);
    let progress_bar_threadsafe = Arc::new(Mutex::new(progress_bar));

    let progress_bar_after = progress_bar_threadsafe.clone();

    let progress_callback = move |file_name: &str| {
        let pb = Arc::clone(&progress_bar_threadsafe);
        trace!("{} finished", file_name);
        pb.lock().inc(1);
        pb.lock().set_message(file_name);
    };

    let mut report_data = Vec::<AnalysisDifference>::new();

    let difference_report_path = if matches.contains_id("difference-report") {
        Some(
            matches
                .get_one::<String>("difference-report")
                .ok_or("difference_report.html"),
        )
    } else {
        None
    };

    collection_analysis(
        &mut nml,
        target_loudness,
        cache,
        progress_callback,
        &mut report_data,
    );

    progress_bar_after.lock().finish();

    trace!("Finished - serializing collection");

    if difference_report_path.is_some() {
        let report_file = File::create(difference_report_path.unwrap().unwrap())?;
        let mut writer = BufWriter::new(report_file);
        let serialized = serde_json::to_string(&report_data).unwrap();
        writer.write_all(serialized.as_bytes())?;
    }

    serialize_collection(nml, output_stream)?;

    trace!("Saving collection");

    if update_in_place(matches) {
        // Backup the collection.
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .to_string();
        let backup_path = input_path.to_owned().to_owned() + ".backup-" + &timestamp;
        copy(input_path, backup_path)?;

        // Replace the input collection.
        copy(&output_temp_path, input_path)?;
    } else if matches.get_one::<String>("output").is_some() {
        let output_path = matches.get_one::<String>("output").ok_or("no output provided")?;

        copy(&output_temp_path, output_path)?;
    }

    Ok(())
}

fn output_stream(
    matches: &ArgMatches,
    output_temp_path: &PathBuf,
) -> Result<Box<dyn Write>, AppError> {
    match matches.get_one::<String>("output") {
        Some(output_path) => match output_path.as_str() {
            "-" => Ok(Box::new(BufWriter::new(std::io::stdout()))),
            _ => Ok(Box::new(BufWriter::new(File::create(output_temp_path)?))),
        },
        None => {
            if update_in_place(matches) {
                Ok(Box::new(BufWriter::new(File::create(output_temp_path)?)))
            } else {
                Ok(Box::new(BufWriter::new(std::io::stdout())))
            }
        }
    }
}

fn update_in_place(matches: &ArgMatches) -> bool {
    matches.contains_id("write")
}

fn deserialize_collection(path: &str) -> Result<Nml, AppError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    match from_reader(buf_reader) {
        Ok(nml) => Ok(nml),
        Err(e) => {
            println!("{:?}", e);
            Err("deserialization error".into())
        }
    }
}

fn kv_to_tuple<'a>(k: &'a str, v: &'a Option<String>) -> (&'a str, &'a str) {
    (k, v.as_ref().unwrap().as_str())
}

#[allow(clippy::cognitive_complexity)]
fn serialize_collection(
    collection: Nml,
    mut output_stream: Box<dyn Write>,
) -> Result<(), AppError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let xml_declaration = BytesDecl::new("1.0", Some("UTF-8"), Some("no"));
    writer.write_event(Event::Decl(xml_declaration))?;

    let mut nml_start_tag = BytesStart::from_content("NML", "NML".len());
    nml_start_tag.push_attribute(("VERSION", collection.version.to_string().as_str()));
    writer.write_event(Event::Start(nml_start_tag))?;

    let mut head_start_tag = BytesStart::from_content("HEAD", "HEAD".len());
    head_start_tag.push_attribute(("COMPANY", collection.head.company.as_str()));
    head_start_tag.push_attribute(("PROGRAM", collection.head.program.as_str()));
    writer.write_event(Event::Start(head_start_tag))?;
    writer.write_event(Event::End(BytesEnd::new("HEAD")))?;

    writer.write_event(Event::Start(BytesStart::from_content(
        "MUSICFOLDERS",
        "MUSICFOLDERS".len(),
    )))?;
    writer.write_event(Event::End(BytesEnd::new("MUSICFOLDERS")))?;

    let mut collection_start_tag = BytesStart::from_content("COLLECTION", "COLLECTION".len());
    collection_start_tag.push_attribute((
        "ENTRIES",
        collection.collection.entries_count.to_string().as_str(),
    ));
    writer.write_event(Event::Start(collection_start_tag))?;

    for entry_ref in collection.collection.entries {
        let entry = entry_ref.lock();

        let mut entry_start_tag = BytesStart::from_content("ENTRY", "ENTRY".len());
        if entry.modified_date.is_some() {
            entry_start_tag.push_attribute((
                "MODIFIED_DATE",
                entry.modified_date.as_ref().unwrap().as_str(),
            ));
        }
        if entry.modified_time.is_some() {
            entry_start_tag.push_attribute((
                "MODIFIED_TIME",
                entry.modified_time.as_ref().unwrap().to_string().as_str(),
            ));
        }
        if entry.audio_id.is_some() {
            entry_start_tag.push_attribute(kv_to_tuple("AUDIO_ID", &entry.audio_id));
        }
        if entry.title.is_some() {
            entry_start_tag.push_attribute(kv_to_tuple("TITLE", &entry.title));
        }
        if entry.artist.is_some() {
            entry_start_tag.push_attribute(kv_to_tuple("ARTIST", &entry.artist));
        }
        writer.write_event(Event::Start(entry_start_tag))?;

        let mut location_start_tag = BytesStart::from_content("LOCATION", "LOCATION".len());
        location_start_tag.push_attribute(("DIR", entry.location.directory.as_str()));
        location_start_tag.push_attribute(("FILE", entry.location.file.as_str()));
        location_start_tag.push_attribute(("VOLUME", entry.location.volume.as_str()));
        location_start_tag.push_attribute(("VOLUMEID", entry.location.volume_id.as_str()));
        writer.write_event(Event::Start(location_start_tag))?;
        writer.write_event(Event::End(BytesEnd::new("LOCATION")))?;

        if entry.album.is_some() {
            let mut album_start_tag = BytesStart::from_content("ALBUM", "ALBUM".len());
            let album = entry.album.as_ref().unwrap();
            if album.track.is_some() {
                album_start_tag
                    .push_attribute(("TRACK", album.track.as_ref().unwrap().to_string().as_str()));
            }
            if album.title.is_some() {
                album_start_tag.push_attribute(kv_to_tuple("TITLE", &album.title));
            }
            writer.write_event(Event::Start(album_start_tag))?;
            writer.write_event(Event::End(BytesEnd::new("ALBUM")))?;
        }

        let mut modification_info_start_tag =
            BytesStart::from_content("MODIFICATION_INFO", "MODIFICATION_INFO".len());
        modification_info_start_tag
            .push_attribute(("AUTHOR_TYPE", entry.modification_info.author_type.as_str()));
        writer.write_event(Event::Start(modification_info_start_tag))?;
        writer.write_event(Event::End(BytesEnd::new("MODIFICATION_INFO")))?;

        let mut info_start_tag = BytesStart::from_content("INFO", "INFO".len());
        if entry.info.bitrate.is_some() {
            info_start_tag.push_attribute((
                "BITRATE",
                entry.info.bitrate.as_ref().unwrap().to_string().as_str(),
            ));
        }
        if entry.info.genre.is_some() {
            info_start_tag.push_attribute(kv_to_tuple("GENRE", &entry.info.genre));
        }
        if entry.info.label.is_some() {
            info_start_tag.push_attribute(kv_to_tuple("LABEL", &entry.info.label));
        }
        if entry.info.cover_art_id.is_some() {
            info_start_tag.push_attribute(kv_to_tuple("COVERARTID", &entry.info.cover_art_id));
        }
        if entry.info.key.is_some() {
            info_start_tag.push_attribute(kv_to_tuple("KEY", &entry.info.key));
        }
        if entry.info.play_count.is_some() {
            info_start_tag.push_attribute((
                "PLAYCOUNT",
                entry.info.play_count.as_ref().unwrap().to_string().as_str(),
            ));
        }
        if entry.info.play_time.is_some() {
            info_start_tag.push_attribute((
                "PLAYTIME",
                entry.info.play_time.as_ref().unwrap().to_string().as_str(),
            ));
        }
        if entry.info.play_time_float.is_some() {
            info_start_tag.push_attribute((
                "PLAYTIME_FLOAT",
                entry
                    .info
                    .play_time_float
                    .as_ref()
                    .unwrap()
                    .to_string()
                    .as_str(),
            ));
        }
        info_start_tag.push_attribute(("IMPORT_DATE", entry.info.import_date.as_str()));
        if entry.info.last_played.is_some() {
            info_start_tag.push_attribute(kv_to_tuple("LAST_PLAYED", &entry.info.last_played));
        }
        if entry.info.release_date.is_some() {
            info_start_tag.push_attribute(kv_to_tuple("RELEASE_DATE", &entry.info.release_date));
        }
        if entry.info.ranking.is_some() {
            info_start_tag.push_attribute(kv_to_tuple("RANKING", &entry.info.ranking));
        }
        if entry.info.rating.is_some() {
            info_start_tag.push_attribute(kv_to_tuple("RATING", &entry.info.rating));
        }
        if entry.info.comment.is_some() {
            info_start_tag.push_attribute(kv_to_tuple("COMMENT", &entry.info.comment));
        }
        if entry.info.flags.is_some() {
            info_start_tag.push_attribute((
                "FLAGS",
                entry.info.flags.as_ref().unwrap().to_string().as_str(),
            ));
        }
        if entry.info.file_size.is_some() {
            info_start_tag.push_attribute((
                "FILESIZE",
                entry.info.file_size.as_ref().unwrap().to_string().as_str(),
            ));
        }
        writer.write_event(Event::Start(info_start_tag))?;
        writer.write_event(Event::End(BytesEnd::new("INFO")))?;

        if entry.tempo.is_some() {
            let tempo = entry.tempo.as_ref().unwrap();
            let mut tempo_start_tag = BytesStart::from_content("TEMPO", "TEMPO".len());
            if tempo.bpm.is_some() {
                tempo_start_tag.push_attribute(("BPM", tempo.bpm.as_ref().unwrap().as_str()));
            }
            tempo_start_tag.push_attribute(("BPM_QUALITY", tempo.bpm_quality.as_str()));
            writer.write_event(Event::Start(tempo_start_tag))?;
            writer.write_event(Event::End(BytesEnd::new("TEMPO")))?;
        }

        if entry.loudness.is_some() {
            let loudness = entry.loudness.as_ref().unwrap();
            let mut loudness_start_tag = BytesStart::from_content("LOUDNESS", "LOUDNESS".len());
            loudness_start_tag.push_attribute((
                "PEAK_DB",
                &*format!("{:.6}", loudness.peak_db.unwrap_or(0.0)),
            ));
            loudness_start_tag.push_attribute((
                "PERCEIVED_DB",
                &*format!("{:.6}", loudness.perceived_db.unwrap_or(0.0)),
            ));
            loudness_start_tag.push_attribute((
                "ANALYZED_DB",
                &*format!("{:.6}", loudness.analyzed_db.unwrap_or(0.0)),
            ));
            writer.write_event(Event::Start(loudness_start_tag))?;
            writer.write_event(Event::End(BytesEnd::new("LOUDNESS")))?;
        }

        if entry.musical_key.is_some() {
            let musical_key = entry.musical_key.as_ref().unwrap();
            let mut musical_key_start_tag = BytesStart::from_content("MUSICAL_KEY", "MUSICAL_KEY".len());
            musical_key_start_tag.push_attribute(("VALUE", musical_key.value.as_ref()));
            writer.write_event(Event::Start(musical_key_start_tag))?;
            writer.write_event(Event::End(BytesEnd::new("MUSICAL_KEY")))?;
        }

        if entry.cue_v2.is_some() {
            for cue in entry.cue_v2.as_ref().unwrap() {
                let mut cue_start = BytesStart::from_content("CUE_V2", "CUE_V2".len());
                cue_start.push_attribute(("NAME", cue.name.as_ref()));
                cue_start.push_attribute(("DISPL_ORDER", cue.display_order.to_string().as_ref()));
                cue_start.push_attribute(("TYPE", cue.cue_type.to_string().as_ref()));
                cue_start.push_attribute(("START", cue.start.to_string().as_ref()));
                cue_start.push_attribute(("LEN", cue.length.to_string().as_ref()));
                cue_start.push_attribute(("REPEATS", cue.repeats.to_string().as_ref()));
                cue_start.push_attribute(("HOTCUE", cue.hotcue.to_string().as_ref()));
                writer.write_event(Event::Start(cue_start))?;
                writer.write_event(Event::End(BytesEnd::new("CUE_V2")))?;
            }
        }

        writer.write_event(Event::End(BytesEnd::new("ENTRY")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("COLLECTION")))?;

    if collection.sets.is_some() {
        let mut sets_tag = BytesStart::from_content("SETS", "SETS".len());
        sets_tag.push_attribute((
            "ENTRIES",
            collection
                .sets
                .as_ref()
                .unwrap()
                .entries
                .to_string()
                .as_ref(),
        ));
        writer.write_event(Event::Start(sets_tag))?;
        writer.write_event(Event::End(BytesEnd::new("SETS")))?;
    }

    if collection.playlists.is_some() {
        let playlists_tag = BytesStart::from_content("PLAYLISTS", "PLAYLISTS".len());
        writer.write_event(Event::Start(playlists_tag))?;

        for node in collection.playlists.unwrap().nodes {
            writer = write_node(writer, &node)?;
        }

        writer.write_event(Event::End(BytesEnd::new("PLAYLISTS")))?;
    }

    if collection.sorting_orders.is_some() {
        for sorting_order in collection.sorting_orders.as_ref().unwrap() {
            let mut sorting_order_tag = BytesStart::from_content("SORTING_ORDER", "SORTING_ORDER".len());
            sorting_order_tag.push_attribute(("PATH", sorting_order.path.as_str()));
            writer.write_event(Event::Start(sorting_order_tag))?;

            if sorting_order.sorting_data.is_some() {
                let sorting_data = sorting_order.sorting_data.as_ref().unwrap();
                let mut sorting_data_tag = BytesStart::from_content("SORTING_DATA", "SORTING_DATA".len());
                sorting_data_tag.push_attribute(("IDX", sorting_data.idx.as_ref()));
                sorting_data_tag.push_attribute(("ORD", sorting_data.ord.as_ref()));
                writer.write_event(Event::Start(sorting_data_tag))?;
                writer.write_event(Event::End(BytesEnd::new("SORTING_DATA")))?;
            }

            writer.write_event(Event::End(BytesEnd::new("SORTING_ORDER")))?;
        }
    }

    writer.write_event(Event::End(BytesEnd::new("NML")))?;

    output_stream.write_all(writer.into_inner().into_inner().as_ref())?;

    Ok(())
}

fn write_node(
    mut writer: Writer<Cursor<Vec<u8>>>,
    node: &Node,
) -> Result<Writer<Cursor<Vec<u8>>>, AppError> {
    let mut node_tag = BytesStart::from_content("NODE", "NODE".len());
    node_tag.push_attribute(("TYPE", node.node_type.as_str()));
    node_tag.push_attribute(("NAME", node.name.as_str()));
    writer.write_event(Event::Start(node_tag))?;

    if node.subnodes.is_some() {
        let subnodes = node.subnodes.as_ref().unwrap();
        let mut sub_node_tag = BytesStart::from_content("SUBNODES", "SUBNODES".len());
        sub_node_tag.push_attribute(("COUNT", subnodes.count.to_string().as_ref()));
        writer.write_event(Event::Start(sub_node_tag))?;

        for sub_node in &subnodes.nodes {
            writer = write_node(writer, sub_node)?;
        }

        writer.write_event(Event::End(BytesEnd::new("SUBNODES")))?;
    }

    if node.playlist.is_some() {
        let playlist = node.playlist.as_ref().unwrap();
        let mut playlist_tag = BytesStart::from_content("PLAYLIST", "PLAYLIST".len());
        playlist_tag.push_attribute(("ENTRIES", playlist.entries_count.to_string().as_str()));
        playlist_tag.push_attribute(("TYPE", playlist.playlist_type.as_str()));
        playlist_tag.push_attribute(("UUID", playlist.uuid.as_str()));
        writer.write_event(Event::Start(playlist_tag))?;

        if playlist.entries.is_some() {
            for entry in playlist.entries.as_ref().unwrap() {
                let entry_tag = BytesStart::from_content("ENTRY", "ENTRY".len());
                writer.write_event(Event::Start(entry_tag))?;

                let mut primary_key_tag = BytesStart::from_content("PRIMARYKEY", "PRIMARY_KEY".len());
                primary_key_tag
                    .push_attribute(("TYPE", entry.primary_key.primary_key_type.as_ref()));
                primary_key_tag.push_attribute(("KEY", entry.primary_key.key.as_ref()));
                writer.write_event(Event::Start(primary_key_tag))?;

                writer.write_event(Event::End(BytesEnd::new("PRIMARYKEY")))?;
                writer.write_event(Event::End(BytesEnd::new("ENTRY")))?;
            }
        }

        writer.write_event(Event::End(BytesEnd::new("PLAYLIST")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("NODE")))?;

    Ok(writer)
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::str::from_utf8;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn serialization_roundtrip_on_a_1_element_collection() {
        serialization_roundtrip_test("tests/vectors/1_element_collection.nml");
    }

    #[test]
    fn serialization_roundtrip_on_a_large_collection_with_1_playlist() {
        serialization_roundtrip_test("tests/vectors/collection_with_1_playlist.nml");
    }

    #[test]
    fn serialization_roundtrip_on_a_collection_with_2_sorting_orders() {
        serialization_roundtrip_test("tests/vectors/collection_with_2_sorting_orders.nml");
    }

    fn serialization_roundtrip_test(input_path: &str) {
        let output_dir = TempDir::new().unwrap();
        let output_path = output_dir.path().join("output.nml");
        let output_stream = Box::new(File::create(output_path.clone()).unwrap());

        let collection = deserialize_collection(input_path).unwrap();

        serialize_collection(collection, output_stream).unwrap();

        let formatted_input_path = output_dir.path().join("formatted_input.nml");
        let formatted_output_path = output_dir.path().join("formatted_output.nml");

        let formatted_input_file = File::create(&formatted_input_path).unwrap();
        let formatted_output_file = File::create(&formatted_output_path).unwrap();

        Command::new("xmllint")
            .arg("--format")
            .arg(input_path)
            .stdout(Stdio::from(formatted_input_file))
            .output()
            .expect("lint");

        Command::new("xmllint")
            .arg("--format")
            .arg(&output_path)
            .stdout(Stdio::from(formatted_output_file))
            .output()
            .expect("lint");

        let output = Command::new("diff")
            .arg("-U8")
            .arg(&formatted_input_path.as_os_str().to_str().unwrap())
            .arg(&formatted_output_path.as_os_str().to_str().unwrap())
            .output()
            .expect("diff");

        let stdout = from_utf8(output.stdout.as_ref()).unwrap();

        if output.status.code() != Some(0) {
            println!("{}", stdout);
        }

        assert_eq!(output.status.code().unwrap(), 0)
    }
}
