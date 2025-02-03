#[tauri::command]
pub async fn check_file_exists(path: String) -> Result<bool, String> {
    match tokio::fs::try_exists(path).await {
        Ok(exists) => Ok(exists),
        Err(e) => {
            log::warn!("Failed to check file existence: {}", e);
            Ok(false) // Return false for any errors (permission, etc)
        }
    }
} 