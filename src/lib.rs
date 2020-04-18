pub mod analysis;
mod collection;
mod error;
mod models;
mod scanner;
mod utils;

use crate::utils::exit_with_error;
use clap::{load_yaml, App};

pub fn cli() {
    let mut builder = pretty_env_logger::formatted_builder();
    builder.filter(None, log::LevelFilter::Error).init();

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
