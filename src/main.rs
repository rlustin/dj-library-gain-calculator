extern crate clap;
extern crate quick_xml;
extern crate serde;

use clap::{load_yaml, App};
use traktor::parse_traktor_collection;

mod error;
mod models;
mod traktor;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    match matches.value_of("input") {
        Some(input) => match parse_traktor_collection(input) {
            Ok(nml) => {
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
