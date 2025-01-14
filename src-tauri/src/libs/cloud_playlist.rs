
use ormlite::model::Model;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "cloud_playlists")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudPlaylist {
    #[ormlite(primary_key)]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[ormlite(json)]
    pub tracks: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl CloudPlaylist {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            tracks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_track(&mut self, index_hash: String) {
        let now = chrono::Utc::now().timestamp();
        self.tracks.push(index_hash);
        self.updated_at = now;
    }

    pub fn remove_track(&mut self, index_hash: &str) {
        self.tracks.retain(|t| t != index_hash);
        self.updated_at = chrono::Utc::now().timestamp();
    }
} 