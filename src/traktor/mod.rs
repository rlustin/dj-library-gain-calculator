use self::models::Nml;
use crate::error::AppError;
use quick_xml::de::from_reader;
use std::fs::File;
use std::io::BufReader;

pub mod models;

pub fn deserialize_collection(path: &str) -> Result<Nml, AppError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let nml: Nml = from_reader(buf_reader)?;

    Ok(nml)
}
