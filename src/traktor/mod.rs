use crate::error::AppError;
use crate::models::Nml;
use quick_xml::de::from_reader;
use std::fs::File;
use std::io::BufReader;

pub fn parse_traktor_collection(input: &str) -> Result<Nml, AppError> {
    let file = File::open(input)?;
    let buf_reader = BufReader::new(file);
    let nml: Nml = from_reader(buf_reader)?;

    Ok(nml)
}
