#![warn(missing_docs)]

use docker_types::DockerError;
use std::path::{Path, PathBuf};
use tokio::fs;

/// 结果类型
pub type StorageResult<T> = std::result::Result<T, DockerError>;

pub struct StorageManager {
    base_path: PathBuf,
}

impl StorageManager {
    pub fn new() -> StorageResult<Self> {
        let base_path = Self::get_base_path()?;
        Ok(Self { base_path })
    }

    pub async fn ensure_directories(&self) -> StorageResult<()> {
        let directories = [
            &self.base_path,
            &self.containers_path()?,
            &self.images_path()?,
            &self.volumes_path()?,
            &self.tmp_path()?,
        ];

        for dir in directories {
            if !dir.exists() {
                fs::create_dir_all(dir).await?;
            }
        }

        Ok(())
    }

    pub fn containers_path(&self) -> StorageResult<PathBuf> {
        Ok(self.base_path.join("containers"))
    }

    pub fn container_path(&self, container_id: &str) -> StorageResult<PathBuf> {
        Ok(self.containers_path()?.join(container_id))
    }

    pub fn images_path(&self) -> StorageResult<PathBuf> {
        Ok(self.base_path.join("images"))
    }

    pub fn image_path(&self, image_id: &str) -> StorageResult<PathBuf> {
        Ok(self.images_path()?.join(image_id))
    }

    pub fn volumes_path(&self) -> StorageResult<PathBuf> {
        Ok(self.base_path.join("volumes"))
    }

    pub fn volume_path(&self, volume_id: &str) -> StorageResult<PathBuf> {
        Ok(self.volumes_path()?.join(volume_id))
    }

    pub fn tmp_path(&self) -> StorageResult<PathBuf> {
        Ok(self.base_path.join("tmp"))
    }

    pub fn get_base_path() -> StorageResult<PathBuf> {
        #[cfg(windows)]
        {
            let app_data =
                std::env::var("APPDATA").map_err(|_| DockerError::config_missing("APPDATA"))?;
            Ok(PathBuf::from(app_data).join("DockerCrab"))
        }

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").map_err(|_| DockerError::config_missing("HOME"))?;
            Ok(PathBuf::from(home).join(".docker-crab"))
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").map_err(|_| DockerError::config_missing("HOME"))?;
            Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("DockerCrab"))
        }
    }

    pub async fn create_file(&self, path: &Path, content: &[u8]) -> StorageResult<()> {
        fs::write(path, content).await?;
        Ok(())
    }

    pub async fn read_file(&self, path: &Path) -> StorageResult<Vec<u8>> {
        let content = fs::read(path).await?;
        Ok(content)
    }

    pub async fn remove_file(&self, path: &Path) -> StorageResult<()> {
        if path.exists() {
            fs::remove_file(path).await?;
        }
        Ok(())
    }

    pub async fn remove_directory(&self, path: &Path) -> StorageResult<()> {
        if path.exists() {
            fs::remove_dir_all(path).await?;
        }
        Ok(())
    }
}
