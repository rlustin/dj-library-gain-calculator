use crate::models;
use crate::utils::*;
use audrey;
use cfg_if::cfg_if;
use claxon;
use ebur128::{EbuR128, Mode};
use hound;
use rayon::prelude::*;
use rmp3::{Decoder, Frame};
use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct DecodedFile {
    channels: u32,
    rate: u32,
    data: Vec<f32>, // interleaved
}

fn i16_in_i32_to_float(integer: i32) -> f32 {
    (integer as f32) / (2_u32.pow(14) as f32)
}

fn i24_to_float(integer: i32) -> f32 {
    // input is i32, but the most significant byte is all 0s
    (integer as f32) / (2_u32.pow(23) as f32)
}

fn i32_to_float(integer: i32) -> f32 {
    (integer as f32) / (2_u32.pow(31) as f32)
}

fn handle_audrey(path: &str) -> Result<DecodedFile, String> {
    let maybe_file = audrey::read::open(&path);
    if let Ok(mut file) = maybe_file {
        let desc = file.description();
        let data: Vec<f32> = file.samples().map(Result::unwrap).collect::<Vec<_>>();
        Ok(DecodedFile {
            channels: desc.channel_count(),
            rate: desc.sample_rate(),
            data,
        })
    } else {
        Err("file not found".to_string())
    }
}

fn handle_hound(path: &str) -> Result<DecodedFile, String> {
    match hound::WavReader::open(path) {
        Ok(mut reader) => {
            let spec = reader.spec();
            let data: Vec<f32> = match reader.spec().sample_format {
                hound::SampleFormat::Int => {
                    let conversion_function = match reader.spec().bits_per_sample {
                        16 => i16_in_i32_to_float,
                        24 => i24_to_float,
                        32 => i32_to_float,
                        _ => {
                            return Err(format!(
                                "Integer {} bits not supported",
                                reader.spec().bits_per_sample
                            ));
                        }
                    };
                    reader
                        .samples::<i32>()
                        .map(Result::unwrap)
                        .map(conversion_function)
                        .collect()
                }
                hound::SampleFormat::Float => reader.samples::<f32>().map(Result::unwrap).collect(),
            };
            Ok(DecodedFile {
                channels: spec.channels.into(),
                rate: spec.sample_rate,
                data,
            })
        }
        Err(_) => Err("invalid wav".to_string()),
    }
}

fn handle_claxon(path: &str) -> Result<DecodedFile, String> {
    match claxon::FlacReader::open(path) {
        Ok(mut reader) => {
            let conversion_function = match reader.streaminfo().bits_per_sample {
                16 => i16_in_i32_to_float,
                24 => i24_to_float,
                32 => i32_to_float,
                _ => {
                    return Err("flac sample type not supported".to_string());
                }
            };
            let data = reader
                .samples()
                .map(Result::unwrap)
                .map(conversion_function)
                .collect::<Vec<f32>>();
            let spec = reader.streaminfo();
            Ok(DecodedFile {
                channels: spec.channels,
                rate: spec.sample_rate,
                data,
            })
        }
        Err(_) => Err("invalid flac".to_string()),
    }
}

fn handle_minimp3(path: &str) -> Result<DecodedFile, String> {
    match File::open(path) {
        Ok(mut f) => {
            let mut buffer = Vec::new();
            // read the whole file
            f.read_to_end(&mut buffer).unwrap();

            let mut decoder = Decoder::new(&buffer);
            let mut rate: u32 = 0;
            let mut ch: u32 = 0;
            let mut pcm_data = Vec::<f32>::new();
            while let Some(Frame {
                channels,
                sample_rate,
                samples,
                ..
            }) = decoder.next_frame()
            {
                if rate != sample_rate && rate != 0 {
                    return Err("inconsistent sample-rate".to_string());
                } else {
                    rate = sample_rate;
                }
                if ch != channels.try_into().unwrap() && ch != 0 {
                    return Err("inconsistent channel count".to_string());
                } else {
                    ch = channels.try_into().unwrap();
                }
                pcm_data.extend(samples);
            }
            Ok(DecodedFile {
                channels: ch,
                rate: rate as u32,
                data: pcm_data,
            })
        }
        _ => Err("file not found".to_string()),
    }
}

pub struct ComputedLoudness {
    pub integrated_loudness: f32,
    pub true_peak: f32,
}

pub fn scan_loudness(path: &str) -> Result<ComputedLoudness, String> {
    let decode_result = match Path::new(path)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("??")
    {
        "ogg" => handle_audrey(&path),
        "wav" => handle_hound(&path),
        "flac" => handle_claxon(&path),
        "mp3" => handle_minimp3(&path),
        _ => Err("unknown file type".to_string()),
    };

    match decode_result {
        Ok(decoded) => {
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
            Ok(ComputedLoudness {
                integrated_loudness: ebu.loudness_global().unwrap() as f32,
                true_peak: max_peak as f32,
            })
        }
        Err(e) => Err(e),
    }
}

pub fn collection_analysis(collection: &mut models::Nml, target_loudness: f32) {
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

            match scan_loudness(&path) {
                Ok(loudness) => {
                    let peak = linear_to_db(loudness.true_peak);
                    let gain = loudness_to_gain(loudness.integrated_loudness, target_loudness);
                    let peak_after_gain = peak + gain;

                    if peak_after_gain > 0.0 {
                        eprintln!("warning: {} clipping at {}", &path, peak_after_gain);
                    }

                    entry.loudness.as_mut().unwrap().analyzed_db = gain as f64;
                    entry.loudness.as_mut().unwrap().perceived_db = gain as f64;
                    entry.loudness.as_mut().unwrap().peak_db = peak as f64;
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        });
}
