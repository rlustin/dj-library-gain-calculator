use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Location {
    #[serde(rename = "DIR")]
    pub directory: String,
    #[serde(rename = "FILE")]
    pub file: String,
    #[serde(rename = "VOLUME")]
    pub volume: String,
    #[serde(rename = "VOLUMEID")]
    pub volume_id: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Loudness {
    #[serde(rename = "ANALYZED_DB")]
    pub analyzed_db: f64,
    #[serde(rename = "PERCEIVED_DB")]
    pub perceived_db: f64,
    #[serde(rename = "PEAK_DB")]
    pub peak_db: f64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Entry {
    #[serde(rename = "ARTIST")]
    pub artist: Option<String>,
    #[serde(rename = "LOCATION")]
    pub location: Location,
    #[serde(rename = "LOUDNESS")]
    pub loudness: Option<Loudness>,
    #[serde(rename = "TITLE")]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Collection {
    #[serde(rename = "ENTRY")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "NML")]
pub struct Nml {
    #[serde(rename = "COLLECTION")]
    pub collection: Collection,
}
