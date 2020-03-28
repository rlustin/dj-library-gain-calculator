use clap::{load_yaml, App, ArgMatches};
use std::fs::{copy, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tempdir::TempDir;

use dj_library_gain_calculator::analysis::*;
use dj_library_gain_calculator::collection::*;
use dj_library_gain_calculator::error::*;
use dj_library_gain_calculator::utils::exit_with_error;

fn main() {
    let yaml = load_yaml!("cli-collection.yml");
    let app = App::from(yaml);

    match process(app) {
        Ok(_) => {}
        Err(error) => exit_with_error(&error.to_string()),
    };
}

fn process(app: App) -> Result<(), AppError> {
    let matches = app.get_matches();

    let input_path = matches.value_of("input").ok_or("no input provided")?;
    let temp_dir = TempDir::new("traktor")?;
    let output_temp_path = temp_dir.path().join("collection.nml");
    let output_stream = output_stream(&matches, &output_temp_path)?;

    let mut nml = deserialize_collection(input_path)?;

    collection_analysis(&mut nml);

    serialize_collection(nml, output_stream)?;

    if update_in_place(&matches) {
        // Backup the collection.
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .to_string();
        let backup_path = input_path.to_owned() + ".backup-" + &timestamp;
        copy(input_path, backup_path)?;

        // Replace the input collection.
        copy(&output_temp_path, input_path)?;
    } else if matches.value_of("output").is_some() {
        let output_path = matches.value_of("output").ok_or("no output provided")?;

        copy(&output_temp_path, output_path)?;
    }

    Ok(())
}

fn output_stream(
    matches: &ArgMatches,
    output_temp_path: &PathBuf,
) -> Result<Box<dyn Write>, AppError> {
    match matches.value_of("output") {
        Some(output_path) => match output_path {
            "-" => Ok(Box::new(BufWriter::new(std::io::stdout()))),
            _ => Ok(Box::new(BufWriter::new(File::create(output_temp_path)?))),
        },
        None => {
            if update_in_place(&matches) {
                Ok(Box::new(BufWriter::new(File::create(output_temp_path)?)))
            } else {
                Ok(Box::new(BufWriter::new(std::io::stdout())))
            }
        }
    }
}

fn update_in_place(matches: &ArgMatches) -> bool {
    matches.is_present("write")
}
