use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Album {
    #[serde(rename = "TITLE")]
    pub title: Option<String>,
    #[serde(rename = "TRACK")]
    pub track: Option<i64>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Collection {
    #[serde(rename = "ENTRIES")]
    pub entries_count: i64,
    #[serde(rename = "ENTRY")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Entry {
    #[serde(rename = "ALBUM")]
    pub struc: Option<Album>,
    #[serde(rename = "ARTIST")]
    pub artist: Option<String>,
    #[serde(rename = "AUDIO_ID")]
    pub audio_id: Option<String>,
    #[serde(rename = "CUE_V2")]
    pub cue_v2: Option<Vec<CueV2>>,
    #[serde(rename = "INFO")]
    pub info: Info,
    #[serde(rename = "LOCATION")]
    pub location: Location,
    #[serde(rename = "LOUDNESS")]
    pub loudness: Option<Loudness>,
    #[serde(rename = "MODIFICATION_INFO")]
    pub modification_info: ModificationInfo,
    #[serde(rename = "MODIFIED_DATE")]
    pub modified_date: String,
    #[serde(rename = "MODIFIED_TIME")]
    pub modified_time: i64,
    #[serde(rename = "MUSICAL_KEY")]
    pub musical_key: Option<MusicalKey>,
    #[serde(rename = "TITLE")]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Head {
    #[serde(rename = "COMPANY")]
    pub company: String,
    #[serde(rename = "PROGRAM")]
    pub program: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Info {
    #[serde(rename = "BITRATE")]
    pub bitrate: Option<i64>,
    #[serde(rename = "COVERARTID")]
    pub cover_art_id: Option<String>,
    #[serde(rename = "FILESIZE")]
    pub file_size: i64,
    #[serde(rename = "FLAGS")]
    pub flags: i64,
    #[serde(rename = "GENRE")]
    pub genre: Option<String>,
    #[serde(rename = "IMPORT_DATE")]
    pub import_date: String,
    #[serde(rename = "KEY")]
    pub key: Option<String>,
    #[serde(rename = "LABEL")]
    pub label: Option<String>,
    #[serde(rename = "LAST_PLAYED")]
    pub last_played: Option<String>,
    #[serde(rename = "PLAYCOUNT")]
    pub play_count: Option<i64>,
    #[serde(rename = "PLAYTIME")]
    pub play_time: Option<i64>,
    #[serde(rename = "PLAYTIME_FLOAT")]
    pub play_time_float: Option<f64>,
    #[serde(rename = "RELEASE_DATE")]
    pub release_date: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CueV2 {
    #[serde(rename = "TYPE")]
    pub cue_type: i64,
    #[serde(rename = "DISPL_ORDER")]
    pub display_order: i64,
    #[serde(rename = "HOTCUE")]
    pub hotcue: i64,
    #[serde(rename = "LEN")]
    pub length: f64,
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "REPEATS")]
    pub repeats: i64,
    #[serde(rename = "START")]
    pub start: f64,
}

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
pub struct ModificationInfo {
    #[serde(rename = "AUTHOR_TYPE")]
    pub author_type: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MusicalKey {
    #[serde(rename = "VALUE")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "NML")]
pub struct Nml {
    #[serde(rename = "HEAD")]
    pub head: Head,
    #[serde(rename = "COLLECTION")]
    pub collection: Collection,
    #[serde(rename = "VERSION")]
    pub version: i64,
}
