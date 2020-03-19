extern crate clap;
extern crate minimp3;
extern crate quick_xml;
extern crate rayon;
extern crate serde;

use clap::{load_yaml, App};
use minimp3::{Decoder, Error, Frame};
use rayon::prelude::*;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use traktor::parse_traktor_collection;

mod error;
mod models;
mod traktor;

struct DecodedFile {
    path: String,
    channels: u32,
    rate: u32,
    data: Vec<f32>, // interleaved
}

fn handle_audrey(path: &str) -> Result<DecodedFile, &str> {
    let maybe_file = audrey::read::open(&path);
    if let Ok(mut file) = maybe_file {
        let desc = file.description();
        let data: Vec<f32> = file.samples().map(Result::unwrap).collect::<Vec<_>>();

        Ok(DecodedFile {
            path: path.to_string(),
            channels: desc.channel_count(),
            rate: desc.sample_rate(),
            data: data,
        })
    } else {
        Err("file not found")
    }
}

fn handle_minimp3(path: &str) -> Result<DecodedFile, &str> {
    match File::open(path) {
        Ok(f) => {
            let mut decoder = Decoder::new(f);
            let mut pcm_data = Vec::<f32>::new();
            let mut sr: i32 = 0;
            let mut ch: u32 = 0;
            let rv = loop {
                match decoder.next_frame() {
                    Ok(Frame {
                        data,
                        sample_rate,
                        channels,
                        ..
                    }) => {
                        if sr != sample_rate && sr != 0 {
                            break Err("inconsistent sample-rate");
                        } else {
                            sr = sample_rate;
                        }
                        if ch != channels.try_into().unwrap() && ch != 0 {
                            break Err("inconsistent channel count");
                        } else {
                            ch = channels.try_into().unwrap();
                        }
                        // i16 -> f32
                        data.iter().for_each(|s| {
                            pcm_data.push((*s as f32) / (2_u32.pow(14) as f32));
                        });
                    }
                    Err(Error::Eof) => break Ok(pcm_data),
                    Err(_) => {
                        break Err("mp3 corrupted");
                    }
                }
            };
            match rv {
                Ok(data) => Ok(DecodedFile {
                    path: path.to_string(),
                    channels: ch,
                    rate: sr as u32,
                    data,
                }),
                Err(e) => Err(e),
            }
        }
        _ => Err("file not found"),
    }
}

fn collection_analysis(collection: &models::Nml) {
    collection.collection.entries.par_iter().for_each(|entry| {
        let mut path = entry.location.directory.clone();
        path.retain(|c| c != ':');
        path.push_str(&entry.location.file);
        // open file
        eprintln!("starting: {}", path);
        let decode_result = match Path::new(&entry.location.file)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("??")
        {
            "wav" | "flac" | "ogg" => handle_audrey(&path),
            "mp3" => handle_minimp3(&path),
            _ => Err("unknown file type"),
        };

        match decode_result {
            Ok(decoded) => {
                eprintln!(
                    "name: {}\nchannels: {} sample-rate: {}, frame count: {}",
                    decoded.path,
                    decoded.channels,
                    decoded.rate,
                    decoded.data.len() / (decoded.channels as usize)
                );
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
        // analyse
        // out integrated lufs
    });
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    match matches.value_of("input") {
        Some(input) => match parse_traktor_collection(input) {
            Ok(nml) => {
                collection_analysis(&nml);

                for entry in nml.collection.entries {
                    println!(
                        "{} — {}",
                        entry.artist.unwrap_or("[none]".to_string()),
                        entry.title.unwrap_or("[none]".to_string())
                    );
                }
            }
            Err(error) => exit_with_error(&error.to_string()),
        },
        None => exit_with_error("no collection input provided"),
    }
}

fn exit_with_error(message: &str) {
    use std::process;

    println!("{}", message);
    println!("aborting…");

    process::exit(1);
}
