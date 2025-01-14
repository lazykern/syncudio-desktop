use ormlite::model::Model;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model, TS)]
#[ormlite(table = "local_folders")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct LocalFolder {
    #[ormlite(primary_key)]
    pub path: String,  // Absolute path to the music folder
} 