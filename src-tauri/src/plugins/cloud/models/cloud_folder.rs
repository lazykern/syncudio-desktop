use ormlite::model::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "cloud_folders")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudFolder {
    #[ormlite(primary_key)]
    pub id: String,
    pub provider_type: String,
    pub cloud_folder_id: String,
    pub cloud_folder_path: String,
    pub local_folder_path: String,
}

impl CloudFolder {
    pub fn new(
        provider_type: String,
        cloud_folder_id: String,
        cloud_folder_path: String,
        local_folder_path: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            provider_type,
            cloud_folder_id,
            cloud_folder_path,
            local_folder_path,
        }
    }
}
