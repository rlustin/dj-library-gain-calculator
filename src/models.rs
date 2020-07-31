use parking_lot::Mutex;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct Album {
    #[serde(rename = "TITLE")]
    pub title: Option<String>,
    #[serde(rename = "TRACK")]
    pub track: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Collection {
    #[serde(rename = "ENTRIES")]
    pub entries_count: i64,
    #[serde(rename = "ENTRY")]
    pub entries: Vec<Arc<Mutex<Entry>>>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    #[serde(rename = "ALBUM")]
    pub album: Option<Album>,
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
    pub modified_date: Option<String>,
    #[serde(rename = "MODIFIED_TIME")]
    pub modified_time: Option<i64>,
    #[serde(rename = "MUSICAL_KEY")]
    pub musical_key: Option<MusicalKey>,
    #[serde(rename = "TEMPO")]
    pub tempo: Option<Tempo>,
    #[serde(rename = "TITLE")]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Head {
    #[serde(rename = "COMPANY")]
    pub company: String,
    #[serde(rename = "PROGRAM")]
    pub program: String,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    #[serde(rename = "BITRATE")]
    pub bitrate: Option<i64>,
    #[serde(rename = "COVERARTID")]
    pub cover_art_id: Option<String>,
    #[serde(rename = "FILESIZE")]
    pub file_size: Option<i64>,
    #[serde(rename = "FLAGS")]
    pub flags: Option<i64>,
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
    pub play_time: Option<String>,
    #[serde(rename = "PLAYTIME_FLOAT")]
    pub play_time_float: Option<String>,
    #[serde(rename = "RELEASE_DATE")]
    pub release_date: Option<String>,
    #[serde(rename = "RATING")]
    pub rating: Option<String>,
    #[serde(rename = "COMMENT")]
    pub comment: Option<String>,
    #[serde(rename = "RANKING")]
    pub ranking: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CueV2 {
    #[serde(rename = "TYPE")]
    pub cue_type: i64,
    #[serde(rename = "DISPL_ORDER")]
    pub display_order: i64,
    #[serde(rename = "HOTCUE")]
    pub hotcue: i64,
    #[serde(rename = "LEN")]
    pub length: String,
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "REPEATS")]
    pub repeats: i64,
    #[serde(rename = "START")]
    pub start: String,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Loudness {
    #[serde(rename = "ANALYZED_DB")]
    pub analyzed_db: f64,
    #[serde(rename = "PERCEIVED_DB")]
    pub perceived_db: f64,
    #[serde(rename = "PEAK_DB")]
    pub peak_db: f64,
}

#[derive(Debug, Deserialize)]
pub struct ModificationInfo {
    #[serde(rename = "AUTHOR_TYPE")]
    pub author_type: String,
}

#[derive(Debug, Deserialize)]
pub struct MusicalKey {
    #[serde(rename = "VALUE")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "NML")]
pub struct Nml {
    #[serde(rename = "HEAD")]
    pub head: Head,
    #[serde(rename = "COLLECTION")]
    pub collection: Collection,
    #[serde(rename = "PLAYLISTS")]
    pub playlists: Option<Playlists>,
    #[serde(rename = "SETS")]
    pub sets: Option<Sets>,
    #[serde(rename = "SORTING_ORDER")]
    pub sorting_orders: Option<Vec<SortingOrder>>,
    #[serde(rename = "VERSION")]
    pub version: i64,
}

impl Nml {
    pub fn track_count(&self) -> u64 {
        self.collection.entries.len() as u64
    }
}

#[derive(Debug, Deserialize)]
pub struct Node {
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "TYPE")]
    pub node_type: String,
    #[serde(rename = "PLAYLIST")]
    pub playlist: Option<Playlist>,
    #[serde(rename = "SUBNODES")]
    pub subnodes: Option<SubNodes>,
}

#[derive(Debug, Deserialize)]
pub struct Playlist {
    #[serde(rename = "ENTRY")]
    pub entries: Option<Vec<PlayListEntry>>,
    #[serde(rename = "ENTRIES")]
    pub entries_count: i64,
    #[serde(rename = "TYPE")]
    pub playlist_type: String,
    #[serde(rename = "UUID")]
    pub uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct PlayListEntry {
    #[serde(rename = "PRIMARYKEY")]
    pub primary_key: PrimaryKey,
}

#[derive(Debug, Deserialize)]
pub struct Playlists {
    #[serde(rename = "NODE")]
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct PrimaryKey {
    #[serde(rename = "TYPE")]
    pub primary_key_type: String,
    #[serde(rename = "KEY")]
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct Sets {
    #[serde(rename = "ENTRIES")]
    pub entries: i64,
}

#[derive(Debug, Deserialize)]
pub struct SortingData {
    #[serde(rename = "IDX")]
    pub idx: String,
    #[serde(rename = "ORD")]
    pub ord: String,
}

#[derive(Debug, Deserialize)]
pub struct SortingOrder {
    #[serde(rename = "PATH")]
    pub path: String,
    #[serde(rename = "SORTING_DATA")]
    pub sorting_data: Option<SortingData>,
}

#[derive(Debug, Deserialize)]
pub struct SubNodes {
    #[serde(rename = "COUNT")]
    pub count: i64,
    #[serde(rename = "NODE")]
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Tempo {
    #[serde(rename = "BPM")]
    pub bpm: Option<String>,
    #[serde(rename = "BPM_QUALITY")]
    pub bpm_quality: String,
}
