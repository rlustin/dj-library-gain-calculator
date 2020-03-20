use crate::models;
use ebur128::{EbuR128, Mode};
use minimp3::{Decoder, Error, Frame};
use rayon::prelude::*;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

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
            data,
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
            let mut rate: i32 = 0;
            let mut ch: u32 = 0;
            let rv = loop {
                match decoder.next_frame() {
                    Ok(Frame {
                        data,
                        sample_rate,
                        channels,
                        ..
                    }) => {
                        if rate != sample_rate && rate != 0 {
                            break Err("inconsistent sample-rate");
                        } else {
                            rate = sample_rate;
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
                    rate: rate as u32,
                    data,
                }),
                Err(e) => Err(e),
            }
        }
        _ => Err("file not found"),
    }
}

pub fn collection_analysis(collection: &models::Nml) {
    collection.collection.entries.par_iter().for_each(|entry| {
        let mut path = entry.location.directory.clone();
        path.retain(|c| c != ':');
        path.push_str(&entry.location.file);
        // open file and decode
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

        let decoded = match decode_result {
            Ok(decoded) => {
                eprintln!(
                    "name: {}\nchannels: {} sample-rate: {}, frame count: {}",
                    decoded.path,
                    decoded.channels,
                    decoded.rate,
                    decoded.data.len() / (decoded.channels as usize)
                );
                decoded
            }
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };
        // analyse
        let mut ebu =
            EbuR128::new(decoded.channels, decoded.rate, Mode::I | Mode::TRUE_PEAK).unwrap();
        ebu.add_frames_f32(&decoded.data).unwrap();

        // out integrated lufs
        eprintln!("Global loundness: {}", ebu.loudness_global().unwrap());
        for i in 0..decoded.channels {
            eprintln!("True peak (channel {}) {}", i, ebu.true_peak(i).unwrap());
        }
    });
}
