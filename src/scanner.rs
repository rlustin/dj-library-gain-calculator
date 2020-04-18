use clap::ArgMatches;
use log::error;
use rayon::prelude::*;

use crate::analysis::scan_loudness;
use crate::error::AppError;
use crate::utils::linear_to_db;

pub fn run(matches: &ArgMatches) -> Result<(), AppError> {
    let input_paths = matches.values_of("input").ok_or("no input provided")?;
    let paths: Vec<&str> = input_paths.collect();

    paths.par_iter().for_each(|path| {
        match scan_loudness(path) {
            Ok(loudness) => {
                println!(
                    "{}\n\tIntegrated loudness: {:.2}dB LUFS\n\tTrue peak: {:.2} ({:.2}dB)",
                    path,
                    loudness.integrated_loudness,
                    loudness.true_peak,
                    linear_to_db(loudness.true_peak)
                );
            }
            Err(e) => {
                error!("{}", e);
            }
        };
    });

    Ok(())
}
