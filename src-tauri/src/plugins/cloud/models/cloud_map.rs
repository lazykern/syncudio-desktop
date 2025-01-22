use ormlite::Model;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "cloud_track_maps")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudTrackMap {
    #[ormlite(primary_key)]
    pub id: String,
    pub cloud_track_id: String,
    pub cloud_music_folder_id: String,
    pub cloud_file_id: Option<String>,
    pub relative_path: String,
}
