use crate::analysis::ComputedLoudness;
use crate::error::AppError;
use bitflags::*;
use log::{error, info, trace};
use rusqlite::*;
use std::fs::remove_file;
use std::path::Path;

bitflags! {
    #[derive(Default)]
    pub struct CachePolicy: u8 {
        const NO_READ = 0b0000_0001;
        const NO_WRITE = 0b0000_0010;
        const PURGE = 0b0000_0100;
    }
}

pub struct AnalyzedFile {
    pub audio_id: String,
    pub loudness_info: ComputedLoudness,
}

pub struct Cache {
    db: Connection,
    policy: CachePolicy,
}

impl Cache {
    pub fn new(path: &Path, policy: CachePolicy) -> Result<Cache, AppError> {
        if policy.contains(CachePolicy::PURGE) {
            match remove_file(path) {
                Ok(_) => {
                    info!("Removed database file {}", path.display());
                }
                Err(e) => {
                    info!("Error removing database file {} ({})", path.display(), e);
                }
            }
        }
        match Connection::open(&path) {
            Ok(db) => {
                db.execute(
                    "CREATE TABLE IF NOT EXISTS tracks (
                         id INTEGER PRIMARY KEY,
                         audio_id TEXT NOT NULL UNIQUE,
                         analyzed_db REAL,
                         peak_db REAL
                     )",
                     (),
                )?;

                Ok(Cache { db, policy })
            }
            Err(e) => {
                error!("Could not open database at {}", path.display());
                Err(e.into())
            }
        }
    }
    #[allow(clippy::match_wild_err_arm)]
    pub fn get(&self, key: &str) -> Option<AnalyzedFile> {
        if self.policy.contains(CachePolicy::NO_READ) {
            return None;
        }

        let maybe_statement = self
            .db
            .prepare("SELECT audio_id, analyzed_db, peak_db FROM tracks where audio_id = ?1");
        let mut statement = match maybe_statement {
            Ok(s) => s,
            Err(_) => {
                panic!("Error in SQL query in Cache::get, fix this.");
            }
        };
        match statement.query(&[key]) {
            Ok(mut rows) => {
                if let Some(row) = rows.next().unwrap() {
                    let integrated_loudness = row.get::<_, f64>(1).unwrap() as f32;
                    let true_peak = row.get::<_, f64>(2).unwrap() as f32;
                    return Some({
                        AnalyzedFile {
                            audio_id: row.get(0).unwrap(),
                            loudness_info: ComputedLoudness {
                                integrated_loudness,
                                true_peak,
                            },
                        }
                    });
                }
            }
            Err(e) => {
                error!(
                    "{}",
                    format!("Error querying the database !? {}", e.to_string())
                );
            }
        }

        None
    }
    pub fn store(&self, file: AnalyzedFile) {
        if self.policy.contains(CachePolicy::NO_WRITE) {
            return;
        }
        match self.db.execute(
            "INSERT INTO tracks (audio_id, analyzed_db, peak_db) VALUES (?1, ?2, ?3)",
            &[
                &file.audio_id,
                &file.loudness_info.integrated_loudness.to_string(),
                &file.loudness_info.true_peak.to_string(),
            ],
        ) {
            Ok(_) => {
                trace!("Storing a result for {}", file.audio_id);
            }
            Err(e) => {
                trace!("Error storing a result for {} ({})", file.audio_id, e);
            }
        }
    }
}
