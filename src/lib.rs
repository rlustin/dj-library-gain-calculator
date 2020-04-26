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
use clap::{load_yaml, App};
use log::LevelFilter::Warn;

pub fn cli() {
    log::set_max_level(Warn);
    log::set_logger(&Logger).unwrap();

    let yaml = load_yaml!("cli.yml");
    let app = App::from(yaml);

    match app.get_matches().subcommand() {
        ("collection", Some(arguments)) => match collection::run(arguments) {
            Ok(_) => {}
            Err(error) => exit_with_error(&error.to_string()),
        },
        ("scanner", Some(arguments)) => match scanner::run(arguments) {
            Ok(_) => {}
            Err(error) => exit_with_error(&error.to_string()),
        },
        ("", None) => println!("No subcommand was used"),
        _ => unreachable!(),
    };
}
