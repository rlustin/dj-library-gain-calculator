use crate::traktor::deserialize_collection;
use clap::{load_yaml, App};

mod analysis;
mod error;
mod traktor;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

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
