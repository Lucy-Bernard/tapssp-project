/*!
 * STORAGE ADAPTER
 *
 * Secondary adapter for file storage operations.
 * In CLI version, we store files locally instead of using cloud storage.
 */

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct StorageAdapter {
    storage_dir: PathBuf,
}

impl StorageAdapter {
    pub fn new() -> Self {
        let storage_dir = std::env::var("STORAGE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
                dir.push("plant-care");
                dir.push("images");
                dir
            });

        // Create directory if it doesn't exist
        fs::create_dir_all(&storage_dir).ok();

        Self { storage_dir }
    }

    pub async fn upload_image(&self, image_data: &[u8], filename: &str) -> Result<String> {
        let file_path = self.storage_dir.join(filename);
        fs::write(&file_path, image_data)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub async fn delete_image(&self, url: &str) -> Result<()> {
        let path = PathBuf::from(url);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}
