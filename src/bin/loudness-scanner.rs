use clap::{load_yaml, App};
use rayon::prelude::*;

use dj_library_gain_calculator::analysis::*;
use dj_library_gain_calculator::error::*;
use dj_library_gain_calculator::utils::exit_with_error;

fn main() {
    let yaml = load_yaml!("cli-scanner.yml");
    let app = App::from(yaml);

    match process(app) {
        Ok(_) => {}
        Err(error) => exit_with_error(&error.to_string()),
    };
}

fn process(app: App) -> Result<(), AppError> {
    let matches = app.get_matches();

    let input_paths = matches.values_of("input").ok_or("no input provided")?;
    let paths: Vec<&str> = input_paths.collect();

    paths.par_iter().for_each(|path| {
        match scan_loudness(path) {
            Ok(loudness) => {
                println!("{}\n\tIntegrated loudness: {:.2}dB LUFS\n\tTrue peak: {:.2}dB", path, loudness.integrated_loudness, loudness.true_peak);
            },
            Err(e) => {
                eprintln!("{}", e.to_string());
            }
        };
    });

    Ok(())
}
