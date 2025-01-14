pub mod dropbox;
pub mod models;

pub use models::{CloudFile, CloudFolder, CloudSync, FileHash};

use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait CloudProvider {
    fn provider_type(&self) -> &'static str;
    async fn is_authorized(&self) -> bool;
    async fn unauthorize(&self);
    async fn list_files(&self, folder_id: &str) -> Result<Vec<CloudFile>, String>;
    async fn list_files_recursive(&self, folder_id: &str) -> Result<Vec<CloudFile>, String>;
    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String>;
    async fn upload_file(&self, local_path: &PathBuf, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String>;
    async fn download_file(&self, file_id: &str, local_path: &PathBuf) -> Result<(), String>;
    async fn delete_file(&self, file_id: &str) -> Result<(), String>;
}

