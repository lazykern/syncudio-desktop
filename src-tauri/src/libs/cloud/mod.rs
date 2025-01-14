pub mod dropbox;
pub mod models;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use async_trait::async_trait;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudFile {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub size: u64,
    pub is_folder: bool,
    pub modified_at: i64,
    pub created_at: i64,
    pub mime_type: Option<String>,
    pub hash: Option<FileHash>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub enum FileHash {
    Sha1(String),
    Sha256(String),
    ContentHash(String), // For Dropbox
}

#[async_trait]
pub trait CloudProvider {
    async fn is_authorized(&self) -> bool;
    async fn unauthorize(&self);
    async fn list_files(&self, folder_id: &str, recursive: bool) -> Result<Vec<CloudFile>, String>;
    async fn list_root_files(&self, recursive: bool) -> Result<Vec<CloudFile>, String>;
    async fn create_folder(&self, name: &str, parent_id: Option<&str>)
        -> Result<CloudFile, String>;
    async fn upload_file(
        &self,
        local_path: &PathBuf,
        name: &str,
        parent_id: Option<&str>,
    ) -> Result<CloudFile, String>;
    async fn download_file(&self, file_id: &str, local_path: &PathBuf) -> Result<(), String>;
    async fn delete_file(&self, file_id: &str) -> Result<(), String>;
}
