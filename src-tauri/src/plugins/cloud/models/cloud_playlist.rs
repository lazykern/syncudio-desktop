use ormlite::model::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "cloud_playlists")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudPlaylist {
    #[ormlite(primary_key)]
    pub id: String,
    pub name: String,
    #[ormlite(json)]
    pub tracks: Vec<String>, // blake3_hash
    pub created_at: i64,
    pub updated_at: i64,
}