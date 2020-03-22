use std::fs::File;
use std::io::Cursor;
use std::io::{BufReader, Write};

use quick_xml::de::from_reader;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;

use crate::error::AppError;

use self::models::Nml;

pub mod models;

pub fn deserialize_collection(path: &str) -> Result<Nml, AppError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let nml: Nml = from_reader(buf_reader)?;

    Ok(nml)
}

fn kv_to_tuple<'a>(k: &'a str, v: &'a Option<String>) -> (&'a str, &'a str)
{
    (k, v.as_ref().unwrap().as_str())
}

pub fn serialize_collection(
    collection: Nml,
    mut output_stream: Box<dyn Write>,
) -> Result<(), AppError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let xml_declaration = BytesDecl::new(b"1.0", Some(b"UTF-8"), Some(b"no"));
    writer.write_event(Event::Decl(xml_declaration))?;

    let mut nml_start_tag = BytesStart::owned("NML", "NML".len());
    nml_start_tag.push_attribute(("VERSION", collection.version.to_string().as_str()));
    writer.write_event(Event::Start(nml_start_tag))?;

    let mut head_start_tag = BytesStart::owned("HEAD", "HEAD".len());
    head_start_tag.push_attribute(("COMPANY", collection.head.company.as_str()));
    head_start_tag.push_attribute(("PROGRAM", collection.head.program.as_str()));
    writer.write_event(Event::Start(head_start_tag))?;
    writer.write_event(Event::End(BytesEnd::borrowed(b"HEAD")))?;

    writer.write_event(Event::Start(BytesStart::borrowed(
        b"MUSICFOLDERS",
        "MUSICFOLDERS".len(),
    )))?;
    writer.write_event(Event::End(BytesEnd::borrowed(b"MUSICFOLDERS")))?;

    let mut collection_start_tag = BytesStart::owned("COLLECTION", "COLLECTION".len());
    collection_start_tag.push_attribute((
        "ENTRIES",
        collection.collection.entries_count.to_string().as_str(),
    ));
    writer.write_event(Event::Start(collection_start_tag))?;

    for entry_ref in collection.collection.entries {
        let entry = entry_ref.lock();

        let mut entry_start_tag = BytesStart::owned("ENTRY", "ENTRY".len());
        entry_start_tag.push_attribute(("MODIFIED_DATE", entry.modified_date.as_str()));
        entry_start_tag.push_attribute(("MODIFIED_TIME", entry.modified_time.to_string().as_str()));
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

        let mut location_start_tag = BytesStart::owned("LOCATION", "LOCATION".len());
        location_start_tag.push_attribute(("DIR", entry.location.directory.as_str()));
        location_start_tag.push_attribute(("FILE", entry.location.file.as_str()));
        location_start_tag.push_attribute(("VOLUME", entry.location.volume.as_str()));
        location_start_tag.push_attribute(("VOLUMEID", entry.location.volume_id.as_str()));
        writer.write_event(Event::Start(location_start_tag))?;
        writer.write_event(Event::End(BytesEnd::borrowed(b"LOCATION")))?;

        let mut album_start_tag = BytesStart::owned("ALBUM", "ALBUM".len());
        if entry.album.is_some() {
            let album = entry.album.as_ref().unwrap();
            if album.track.is_some() {
                album_start_tag
                    .push_attribute(("TRACK", album.track.as_ref().unwrap().to_string().as_str()));
            }
            if album.title.is_some() {
                album_start_tag.push_attribute(kv_to_tuple("TITLE", &album.title));
            }
        }
        writer.write_event(Event::Start(album_start_tag))?;
        writer.write_event(Event::End(BytesEnd::borrowed(b"ALBUM")))?;

        let mut modification_info_start_tag =
            BytesStart::owned("MODIFICATION_INFO", "MODIFICATION_INFO".len());
        modification_info_start_tag
            .push_attribute(("AUTHOR_TYPE", entry.modification_info.author_type.as_str()));
        writer.write_event(Event::Start(modification_info_start_tag))?;
        writer.write_event(Event::End(BytesEnd::borrowed(b"MODIFICATION_INFO")))?;

        let mut info_start_tag = BytesStart::owned("INFO", "INFO".len());
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
        info_start_tag.push_attribute(("FLAGS", entry.info.flags.to_string().as_str()));
        info_start_tag.push_attribute(("FILESIZE", entry.info.file_size.to_string().as_str()));
        writer.write_event(Event::Start(info_start_tag))?;
        writer.write_event(Event::End(BytesEnd::borrowed(b"INFO")))?;

        let mut tempo_start_tag = BytesStart::owned("TEMPO", "TEMPO".len());
        tempo_start_tag.push_attribute(("BPM", entry.tempo.bpm.as_str()));
        tempo_start_tag.push_attribute(("BPM_QUALITY", entry.tempo.bpm_quality.as_str()));
        writer.write_event(Event::Start(tempo_start_tag))?;
        writer.write_event(Event::End(BytesEnd::borrowed(b"TEMPO")))?;

        if entry.loudness.is_some() {
            let loudness = entry.loudness.as_ref().unwrap();
            let mut loudness_start_tag = BytesStart::owned("LOUDNESS", "LOUDNESS".len());
            loudness_start_tag.push_attribute(("PEAK_DB", loudness.peak_db.to_string().as_ref()));
            loudness_start_tag
                .push_attribute(("PERCEIVED_DB", loudness.perceived_db.to_string().as_ref()));
            loudness_start_tag
                .push_attribute(("ANALYZED_DB", loudness.analyzed_db.to_string().as_ref()));
            writer.write_event(Event::Start(loudness_start_tag))?;
            writer.write_event(Event::End(BytesEnd::borrowed(b"LOUDNESS")))?;
        }

        if entry.musical_key.is_some() {
            let musical_key = entry.musical_key.as_ref().unwrap();
            let mut musical_key_start_tag = BytesStart::owned("MUSICAL_KEY", "MUSICAL_KEY".len());
            musical_key_start_tag.push_attribute(("VALUE", musical_key.value.as_ref()));
            writer.write_event(Event::Start(musical_key_start_tag))?;
            writer.write_event(Event::End(BytesEnd::borrowed(b"MUSICAL_KEY")))?;
        }

        if entry.cue_v2.is_some() {
            for cue in entry.cue_v2.as_ref().unwrap() {
                let mut cue_start = BytesStart::owned("CUE_V2", "CUE_V2".len());
                cue_start.push_attribute(("NAME", cue.name.as_ref()));
                cue_start.push_attribute(("DISPL_ORDER", cue.display_order.to_string().as_ref()));
                cue_start.push_attribute(("TYPE", cue.cue_type.to_string().as_ref()));
                cue_start.push_attribute(("START", cue.start.to_string().as_ref()));
                cue_start.push_attribute(("LEN", cue.length.to_string().as_ref()));
                cue_start.push_attribute(("REPEATS", cue.repeats.to_string().as_ref()));
                cue_start.push_attribute(("HOTCUE", cue.hotcue.to_string().as_ref()));
                writer.write_event(Event::Start(cue_start))?;
                writer.write_event(Event::End(BytesEnd::borrowed(b"CUE_V2")))?;
            }
        }

        writer.write_event(Event::End(BytesEnd::borrowed(b"ENTRY")))?;
    }

    writer.write_event(Event::End(BytesEnd::borrowed(b"COLLECTION")))?;
    writer.write_event(Event::End(BytesEnd::borrowed(b"NML")))?;

    output_stream.write_all(writer.into_inner().into_inner().as_ref())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::str::from_utf8;

    use tempdir::TempDir;

    use super::*;

    #[test]
    fn serialization_roundtrip() {
        let input_path = "tests/vectors/1_element_collection.nml";
        let output_dir = TempDir::new("tests").unwrap();
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
