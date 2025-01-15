use async_trait::async_trait;
use std::path::PathBuf;
use crate::plugins::cloud::models::*;

#[async_trait]
pub trait CloudProvider {
    async fn is_authorized(&self) -> bool;
    async fn unauthorize(&self);
    async fn list_files(&self, folder_id: &str, recursive: bool) -> Result<Vec<CloudFile>, String>;
    async fn list_root_files(&self, recursive: bool) -> Result<Vec<CloudFile>, String>;
    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String>;
    async fn upload_file(&self, local_path: &PathBuf, name: &str, parent_id: Option<&str>) -> Result<CloudFile, String>;
    async fn download_file(&self, file_id: &str, local_path: &PathBuf) -> Result<(), String>;
    async fn delete_file(&self, file_id: &str) -> Result<(), String>;
} 