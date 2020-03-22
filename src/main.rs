use crate::traktor::*;
use std::fs::File;
use std::io::BufWriter;
use clap::{load_yaml, App};
use std::any::Any;
use std::io::Write;

mod analysis;
mod error;
mod traktor;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let maybe_output_file: Result<Box<dyn Write>, _> = match matches.value_of("output") {
        Some(output_path) => {
            match output_path {
                "-" => {
                    Ok(Box::new(BufWriter::new(std::io::stdout())))
                }
                _ => {
                    match File::create(output_path) {
                        Ok(f) => {
                            Ok(Box::new(BufWriter::new(f)))
                        }
                        Err(e) => {
                            Err(e)
                        }
                    }
                }
            }
        }
        None => {
            Ok(Box::new(BufWriter::new(std::io::stdout())))
        }
    };

    let mut output_file = match maybe_output_file {
        Ok(output_file) => {
            output_file
        },
        Err(e) => {
            exit_with_error(&e.to_string());
            return;
        }
    };

    match matches.value_of("input") {
        Some(input) => match deserialize_collection(input) {
            Ok(mut nml) => {
                analysis::collection_analysis(&mut nml);
                eprintln!("{:?}", nml);

                let none = "[none]".to_string();

                for entry in nml.collection.entries {
                    let locked_entry = entry.lock();
                    println!(
                        "{} — {}",
                        locked_entry.artist.as_ref().unwrap_or_else(|| &none),
                        locked_entry.title.as_ref().unwrap_or_else(|| &none)
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
