#![warn(missing_docs)]

#[cfg(unix)]
pub mod overlay;

use docker_types::DockerError;
use std::path::{Path, PathBuf};
use tokio::fs;

/// 存储驱动接口
pub trait StorageDriver {
    /// 创建层
    fn create_layer(&self, layer_id: &str, parent_id: Option<&str>) -> StorageResult<()>;

    /// 挂载层
    fn mount_layer(&self, layer_id: &str, mount_point: &Path) -> StorageResult<()>;

    /// 卸载层
    fn unmount_layer(&self, mount_point: &Path) -> StorageResult<()>;

    /// 删除层
    fn delete_layer(&self, layer_id: &str) -> StorageResult<()>;

    /// 获取层路径
    fn get_layer_path(&self, layer_id: &str) -> String;
}

/// 虚拟存储驱动（用于不支持的平台）
pub struct DummyStorageDriver {
    base_path: String,
}

impl DummyStorageDriver {
    pub fn new(base_path: &str) -> StorageResult<Self> {
        Ok(Self { base_path: base_path.to_string() })
    }

    pub fn default() -> StorageResult<Self> {
        Self::new("/var/lib/rusty-docker/overlay")
    }
}

impl StorageDriver for DummyStorageDriver {
    fn create_layer(&self, layer_id: &str, parent_id: Option<&str>) -> StorageResult<()> {
        Ok(())
    }

    fn mount_layer(&self, layer_id: &str, mount_point: &Path) -> StorageResult<()> {
        Ok(())
    }

    fn unmount_layer(&self, mount_point: &Path) -> StorageResult<()> {
        Ok(())
    }

    fn delete_layer(&self, layer_id: &str) -> StorageResult<()> {
        Ok(())
    }

    fn get_layer_path(&self, layer_id: &str) -> String {
        format!("{}/{}", self.base_path, layer_id)
    }
}

#[cfg(unix)]
use overlay::OverlayDriver;

/// 结果类型
pub type StorageResult<T> = std::result::Result<T, DockerError>;

pub struct StorageManager {
    base_path: PathBuf,
    storage_driver: Box<dyn StorageDriver>,
}

impl StorageManager {
    pub fn new() -> StorageResult<Self> {
        let base_path = Self::get_base_path()?;
        let storage_driver: Box<dyn StorageDriver> = {
            #[cfg(unix)]
            {
                Box::new(OverlayDriver::default()?)
            }
            
            #[cfg(not(unix))]
            {
                Box::new(DummyStorageDriver::default()?)
            }
        };
        Ok(Self { base_path, storage_driver })
    }

    pub async fn ensure_directories(&self) -> StorageResult<()> {
        let directories =
            [&self.base_path, &self.containers_path()?, &self.images_path()?, &self.volumes_path()?, &self.tmp_path()?];

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
            let app_data = std::env::var("APPDATA").map_err(|_| DockerError::config_missing("APPDATA"))?;
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
            Ok(PathBuf::from(home).join("Library").join("Application Support").join("DockerCrab"))
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

    /// 创建存储层
    pub fn create_layer(&self, layer_id: &str, parent_id: Option<&str>) -> StorageResult<()> {
        self.storage_driver.create_layer(layer_id, parent_id)
    }

    /// 挂载存储层
    pub fn mount_layer(&self, layer_id: &str, mount_point: &Path) -> StorageResult<()> {
        self.storage_driver.mount_layer(layer_id, mount_point)
    }

    /// 卸载存储层
    pub fn unmount_layer(&self, mount_point: &Path) -> StorageResult<()> {
        self.storage_driver.unmount_layer(mount_point)
    }

    /// 删除存储层
    pub fn delete_layer(&self, layer_id: &str) -> StorageResult<()> {
        self.storage_driver.delete_layer(layer_id)
    }

    /// 获取存储驱动
    pub fn get_storage_driver(&self) -> &dyn StorageDriver {
        &*self.storage_driver
    }
}
