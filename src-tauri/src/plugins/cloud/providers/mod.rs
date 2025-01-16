mod dropbox;

pub use dropbox::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::libs::{
    error::{AnyResult, SyncudioError},
    track::Track,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub enum CloudProviderType {
    #[serde(rename = "dropbox")]
    Dropbox,
    #[serde(rename = "gdrive")]
    GoogleDrive,
}

impl CloudProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CloudProviderType::Dropbox => "dropbox",
            CloudProviderType::GoogleDrive => "gdrive",
        }
    }

    pub fn from_str(s: &str) -> AnyResult<Self> {
        match s {
            "dropbox" => Ok(CloudProviderType::Dropbox),
            "gdrive" => Ok(CloudProviderType::GoogleDrive),
            _ => Err(SyncudioError::InvalidProviderType),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub struct CloudFile {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub size: u64,
    pub is_folder: bool,
    pub modified_at: i64,
    pub mime_type: Option<String>,
    pub hash: Option<FileHash>,
    pub display_path: Option<String>,
    pub relative_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/generated/typings/index.ts")]
pub enum FileHash {
    Sha1(String),
    Sha256(String),
    ContentHash(String), // For Dropbox
}

use async_trait::async_trait;
use std::path::PathBuf;

use super::CloudTracksMetadata;

#[async_trait]
pub trait CloudProvider {
    async fn is_authorized(&self) -> bool;
    async fn unauthorize(&self);
    async fn list_files(&self, folder_id: &str, recursive: bool) -> AnyResult<Vec<CloudFile>>;
    async fn list_root_files(&self, recursive: bool) -> AnyResult<Vec<CloudFile>>;
    async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> AnyResult<CloudFile>;
    async fn upload_file(&self, local_path: &PathBuf, name: &str, parent_id: Option<&str>) -> AnyResult<CloudFile>;
    async fn download_file(&self, file_id: &str, local_path: &PathBuf) -> AnyResult<()>;
    async fn delete_file(&self, file_id: &str) -> AnyResult<()>;

    // Metadata sync methods
    async fn ensure_metadata_folder(&self) -> AnyResult<String> {
        // Create /Syncudio if it doesn't exist
        let syncudio = match self.list_root_files(false).await?.iter()
            .find(|f| f.is_folder && f.name == "Syncudio") {
                Some(f) => f.id.clone(),
                None => self.create_folder("Syncudio", None).await?.id
            };

        // Create /Syncudio/metadata if it doesn't exist
        match self.list_files(&syncudio, false).await?.iter()
            .find(|f| f.is_folder && f.name == "metadata") {
                Some(f) => Ok(f.id.clone()),
                None => Ok(self.create_folder("metadata", Some(&syncudio)).await?.id)
            }
    }

    async fn get_metadata_file_id(&self) -> AnyResult<Option<String>> {
        let metadata_folder = self.ensure_metadata_folder().await?;
        Ok(self.list_files(&metadata_folder, false).await?
            .iter()
            .find(|f| !f.is_folder && f.name == "tracks.json")
            .map(|f| f.id.clone()))
    }

    async fn upload_metadata(&self, metadata: &CloudTracksMetadata) -> AnyResult<()> {
        let metadata_folder = self.ensure_metadata_folder().await?;
        let json = metadata.to_json()?;
        let temp_path = std::env::temp_dir().join("tracks.json");
        std::fs::write(&temp_path, json)?;
        
        // Delete existing file if it exists
        if let Some(file_id) = self.get_metadata_file_id().await? {
            let _ = self.delete_file(&file_id).await;
        }
        
        self.upload_file(&temp_path, "tracks.json", Some(&metadata_folder))
            .await?;
        
        std::fs::remove_file(temp_path)?;
        Ok(())
    }

    async fn download_metadata(&self) -> AnyResult<Option<CloudTracksMetadata>> {
        let temp_path = std::env::temp_dir().join("tracks.json");
        
        // Try to get the metadata file ID
        if let Some(file_id) = self.get_metadata_file_id().await? {
            match self.download_file(&file_id, &temp_path).await {
                Ok(_) => {
                    let json = std::fs::read_to_string(&temp_path)?;
                    std::fs::remove_file(temp_path)?;
                    Ok(Some(CloudTracksMetadata::from_json(&json)?))
                }
                Err(_) => Ok(None)
            }
        } else {
            Ok(None)
        }
    }
} 
