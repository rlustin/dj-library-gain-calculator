use crate::traktor::models;
use audrey;
use cfg_if::cfg_if;
use claxon;
use ebur128::{EbuR128, Mode};
use hound;
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

fn i16_to_float(integer: i16) -> f32 {
    (integer as f32) / (2_u32.pow(14) as f32)
}

fn i16_in_i32_to_float(integer: i32) -> f32 {
    (integer as f32) / (2_u32.pow(14) as f32)
}

fn i24_to_float(integer: i32) -> f32 {
    // most significant byte is 0
    (integer as f32) / (2_u32.pow(22) as f32)
}

fn i32_to_float(integer: i32) -> f32 {
    (integer as f32) / (2_u32.pow(30) as f32)
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

fn handle_hound(path: &str) -> Result<DecodedFile, &str> {
    match hound::WavReader::open(path) {
        Ok(mut reader) => {
            let mut data = Vec::<f32>::new();
            let spec = reader.spec();
            reader
                .samples::<f32>()
                .map(Result::unwrap)
                .for_each(|s| data.push(s));
            return Ok(DecodedFile {
                path: path.to_string(),
                channels: spec.channels.into(),
                rate: spec.sample_rate,
                data,
            });
        }
        Err(_) => {
            return Err("invalid wav");
        }
    }
}

fn handle_claxon(path: &str) -> Result<DecodedFile, &str> {
    match claxon::FlacReader::open(path) {
        Ok(mut reader) => {
            let conversion_function = match reader.streaminfo().bits_per_sample {
                16 => i16_in_i32_to_float,
                24 => i24_to_float,
                32 => i32_to_float,
                _ => {
                    return Err("flac sample type not supported");
                }
            };
            let data = reader
                .samples()
                .map(Result::unwrap)
                .map(conversion_function)
                .collect::<Vec<f32>>();
            let spec = reader.streaminfo();
            return Ok(DecodedFile {
                path: path.to_string(),
                channels: spec.channels,
                rate: spec.sample_rate,
                data,
            });
        }
        Err(_) => {
            return Err("invalid flac");
        }
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
                            pcm_data.push(i16_to_float(*s));
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

pub fn collection_analysis(collection: &mut models::Nml) {
    collection
        .collection
        .entries
        .par_iter_mut()
        .for_each(|entry_ref| {
            let mut entry = entry_ref.lock();

            cfg_if! {
              if #[cfg(target_os = "macos")] {
                let mut path = "/Volumes/".to_string();
              } else if #[cfg(target_os = "windows")] {
                let mut path = "".to_string();
              } else {
                let mut path = "/".to_string();
              }
            }

            path.push_str(&entry.location.volume);
            path.push_str(&entry.location.directory);
            path.retain(|c| c != ':');
            path.push_str(&entry.location.file);
            // open file and decode
            let decode_result = match Path::new(&entry.location.file)
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("??")
            {
                "ogg" => handle_audrey(&path),
                "wav" => handle_hound(&path),
                "flac" => handle_claxon(&path),
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

            let mut ebu =
                EbuR128::new(decoded.channels, decoded.rate, Mode::I | Mode::TRUE_PEAK).unwrap();
            ebu.add_frames_f32(&decoded.data).unwrap();

            // find max peak of all channels: the model has a single value for the peak
            let mut max_peak = 0.0;
            for i in 0..decoded.channels {
                if max_peak < ebu.true_peak(i).unwrap() {
                    max_peak = ebu.true_peak(i).unwrap();
                }
            }

            entry.loudness.as_mut().unwrap().analyzed_db = ebu.loudness_global().unwrap();
            entry.loudness.as_mut().unwrap().perceived_db = ebu.loudness_global().unwrap();
            entry.loudness.as_mut().unwrap().peak_db = max_peak;
        });
}
