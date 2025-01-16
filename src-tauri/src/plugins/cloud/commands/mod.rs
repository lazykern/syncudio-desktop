mod database;
mod provider;

pub use database::*;
pub use provider::*;
use tauri::State;

use crate::{libs::error::AnyResult, plugins::db::DBState};

use super::{CloudFolder, CloudState};

#[tauri::command]
pub async fn import_cloud_folder_tracks(
    folder: CloudFolder,
    cloud_state: State<'_, CloudState>,
    db_state: State<'_, DBState>,
) -> AnyResult<()> {
    let mut db = db_state.get_lock().await;
    let cloud_files = cloud_list_files(
        folder.provider_type,
        folder.cloud_folder_id,
        true,
        cloud_state,
    )
    .await?;

    Ok(())
}
