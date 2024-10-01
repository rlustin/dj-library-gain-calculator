pub mod analysis;
mod cache;
mod collection;
mod error;
mod logging;
mod models;
mod progress;
mod scanner;
mod utils;

use crate::logging::Logger;
use crate::utils::exit_with_error;
use clap::{command, Arg, Command};
use log::LevelFilter::Warn;

pub fn cli() {
    log::set_max_level(Warn);
    log::set_logger(&Logger).unwrap();

    let command = Command::new("dj-library-gain-calculator")
        .about("Analyses all tracks in a Traktor DJ collection to have constant loudness.")
        .version("0.1.0")
        .subcommand_required(true)
        .subcommand(
            command!("collection")
            .about("Analyses all tracks in a Traktor DJ collection to have constant loudness.")
            .arg(
                Arg::new("input")
                .help("The input Traktor collection file to use.")
                .short('i')
                .long("input")
                .required(true)
            )
            .arg(
                Arg::new("output")
                .help("The output Traktor collection file to write or - for stdout.")
                .short('o')
                .long("output")
            )
            .arg(
                Arg::new("target")
                .help("Target loudness in dB LUFS (negative value).")
                .short('t')
                .long("target")
                .allow_hyphen_values(true)
                .default_value("-14.0")
            )
            .arg(
                Arg::new("write")
                .help("Updates the Traktor collection in place.")
                .short('w')
                .long("write")
                .global(true)
                .conflicts_with("output")
            )
            .arg(
                Arg::new("no-cache-read")
                .help("Don't read from cache.")
                .long("no-cache-read")
                .global(true)
            )
            .arg(
                Arg::new("no-cache-write")
                .help("Don't write to cache.")
                .long("no-cache-write")
                .global(true)
            )
            .arg(
                Arg::new("purge-cache")
                .help("Purge the track cache.")
                .short('p')
                .long("purge-cache")
                .global(true)
            )
            .arg(
                Arg::new("cache-file")
                .help("Override the default cache file location.")
                .short('c')
                .long("cache-file")
                .global(true)
            )
            .arg(
                Arg::new("difference-report")
                .help("Output the gain difference.")
                .short('d')
                .long("difference-report")
                .global(true)
            )
        )
        .subcommand(
            command!("scanner")
            .about("Analyses a track or set of tracks and output loudness and peak info.")
            .arg(
                Arg::new("input")
                .help("One or more files to analyse.")
                .required(true)
                .index(1)
            )
        );

    match command.get_matches().subcommand() {
        Some(("collection", matches)) => match collection::run(matches) {
            Ok(_) => {}
            Err(error) => exit_with_error(&error.to_string()),
        },
        Some(("scanner", matches)) => match scanner::run(matches) {
            Ok(_) => {}
            Err(error) => exit_with_error(&error.to_string()),
        },
        None => println!("No subcommand was used"),
        _ => unreachable!(),
    };
}
