use crate::utils::exit_with_error;
use clap::{load_yaml, App};

mod analysis;
mod collection;
mod error;
mod models;
mod scanner;
mod utils;

fn main() {
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
