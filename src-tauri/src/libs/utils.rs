/**
 * Small utility to display time metrics with a log message
 */
use log::info;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{ffi::OsStr, time::Instant};
use tauri::Theme;
use walkdir::WalkDir;

use crate::plugins::config::SYSTEM_THEME;

/**
 * Small helper to compute the execution time of some code
 */
pub struct TimeLogger {
    start_time: Instant,
    message: String,
}

impl TimeLogger {
    pub fn new(message: String) -> Self {
        TimeLogger {
            start_time: Instant::now(),
            message,
        }
    }

    pub fn complete(&self) {
        let duration = self.start_time.elapsed();
        info!("{} in {:.2?}", self.message, duration);
    }
}

/**
 * Check if a directory or a file is visible or not, by checking if it start
 * with a dot
 */
fn is_dir_visible(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| !s.starts_with("."))
        .unwrap_or(false)
}

/**
 * Take an entry and filter out non-allowed extensions
 */
pub fn is_file_valid(path: &Path, allowed_extensions: &[&str]) -> bool {
    let extension = path.extension().and_then(OsStr::to_str).unwrap_or("");
    allowed_extensions.contains(&extension)
}

/**
 * Scan multiple directories and filter files by extension
 */
pub fn scan_dirs(paths: &[PathBuf], allowed_extensions: &[&str]) -> Vec<PathBuf> {
    paths
        .iter()
        .flat_map(|path| scan_dir(path, allowed_extensions))
        .collect()
}

/**
 * Scan directory and filter files by extension
 */
pub fn scan_dir(path: &PathBuf, allowed_extensions: &[&str]) -> Vec<PathBuf> {
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_entry(is_dir_visible)
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| is_file_valid(path, allowed_extensions))
        .collect()
}

/**
 * Give an arbitrary string (usually the theme value from the config), returns
 * a Tauri theme
 */
pub fn get_theme_from_name(theme_name: &str) -> Option<Theme> {
    match theme_name {
        "light" => Some(Theme::Light),
        "dark" => Some(Theme::Dark),
        SYSTEM_THEME => None,
        _ => None, // ? :]
    }
}

const CONTENT_HASH_BLOCK_SIZE: usize = 4 * 1024 * 1024; // 4MB

/**
 * Compute the content hash of a file
 */
pub fn compute_content_hash<R: Read>(mut reader: R) -> String {
    let mut blocks = Vec::new();
    let mut buffer = vec![0u8; CONTENT_HASH_BLOCK_SIZE];

    while let Ok(bytes_read) = reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        blocks.push(buffer[..bytes_read].to_vec());
        if bytes_read < CONTENT_HASH_BLOCK_SIZE {
            break;
        }
    }

    let block_hashes: Vec<_> = blocks
        .par_iter()
        .map(|block| {
            let mut block_hasher = Sha256::new();
            block_hasher.update(block);
            block_hasher.finalize().to_vec()
        })
        .collect();

    let mut overall_hasher = Sha256::new();
    for block_hash in block_hashes {
        overall_hasher.update(block_hash);
    }

    hex::encode(overall_hasher.finalize())
}
