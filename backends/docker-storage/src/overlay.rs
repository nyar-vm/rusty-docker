#![warn(missing_docs)]

//! 联合文件系统实现
//!
//! 实现 OverlayFS 等联合文件系统的支持，用于镜像和容器的存储管理。

use std::{fs, path::Path};

use docker_types::{DockerError, Result};

/// 存储驱动接口
pub trait StorageDriver {
    /// 创建层
    fn create_layer(&self, layer_id: &str, parent_id: Option<&str>) -> Result<()>;

    /// 挂载层
    fn mount_layer(&self, layer_id: &str, mount_point: &Path) -> Result<()>;

    /// 卸载层
    fn unmount_layer(&self, mount_point: &Path) -> Result<()>;

    /// 删除层
    fn delete_layer(&self, layer_id: &str) -> Result<()>;

    /// 获取层路径
    fn get_layer_path(&self, layer_id: &str) -> String;
}

/// OverlayFS 存储驱动
pub struct OverlayDriver {
    /// 存储根路径
    base_path: String,
}

impl OverlayDriver {
    /// 创建新的 OverlayFS 驱动
    pub fn new(base_path: &str) -> Result<Self> {
        // 创建存储目录
        fs::create_dir_all(base_path).map_err(|e| DockerError::io_error("create_overlay_dir", e.to_string()))?;

        // 创建子目录
        let subdirs = ["lower", "upper", "work", "merged"];
        for subdir in &subdirs {
            let dir_path = format!("{}/{}", base_path, subdir);
            fs::create_dir_all(&dir_path).map_err(|e| DockerError::io_error("create_subdir", e.to_string()))?;
        }

        Ok(Self { base_path: base_path.to_string() })
    }

    /// 从默认路径创建 OverlayFS 驱动
    pub fn default() -> Result<Self> {
        Self::new("/var/lib/rusty-docker/overlay")
    }
}

impl StorageDriver for OverlayDriver {
    /// 创建层
    fn create_layer(&self, layer_id: &str, parent_id: Option<&str>) -> Result<()> {
        let layer_path = self.get_layer_path(layer_id);
        fs::create_dir_all(&layer_path).map_err(|e| DockerError::io_error("create_layer", e.to_string()))?;

        // 创建层的子目录
        let subdirs = ["lower", "upper", "work", "merged"];
        for subdir in &subdirs {
            let dir_path = format!("{}/{}", layer_path, subdir);
            fs::create_dir_all(&dir_path).map_err(|e| DockerError::io_error("create_layer_subdir", e.to_string()))?;
        }

        // 如果有父层，设置 lower 目录
        if let Some(parent_id) = parent_id {
            let parent_path = self.get_layer_path(parent_id);
            let lower_dir = format!("{}/lower", layer_path);
            let parent_merged = format!("{}/merged", parent_path);

            // 创建指向父层 merged 目录的符号链接
            if Path::new(&parent_merged).exists() {
                let link_path = format!("{}/parent", lower_dir);
                if Path::new(&link_path).exists() {
                    fs::remove_file(&link_path).ok();
                }
                std::os::unix::fs::symlink(&parent_merged, &link_path)
                    .map_err(|e| DockerError::io_error("create_symlink", e.to_string()))?;
            }
        }

        Ok(())
    }

    /// 挂载层
    fn mount_layer(&self, layer_id: &str, mount_point: &Path) -> Result<()> {
        let layer_path = self.get_layer_path(layer_id);
        let lowerdir = format!("{}/lower", layer_path);
        let upperdir = format!("{}/upper", layer_path);
        let workdir = format!("{}/work", layer_path);

        // 构建 mount 命令
        let mount_cmd = format!(
            "mount -t overlay overlay -o lowerdir={},upperdir={},workdir={} {}",
            lowerdir,
            upperdir,
            workdir,
            mount_point.to_str().unwrap()
        );

        // 执行 mount 命令
        let output = std::process::Command::new("sh")
            .args(&["-c", &mount_cmd])
            .output()
            .map_err(|e| DockerError::io_error("mount_overlay", e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::io_error("mount_failed", error.to_string()));
        }

        Ok(())
    }

    /// 卸载层
    fn unmount_layer(&self, mount_point: &Path) -> Result<()> {
        // 构建 umount 命令
        let umount_cmd = format!("umount {}", mount_point.to_str().unwrap());

        // 执行 umount 命令
        let output = std::process::Command::new("sh")
            .args(&["-c", &umount_cmd])
            .output()
            .map_err(|e| DockerError::io_error("unmount_overlay", e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::io_error("unmount_failed", error.to_string()));
        }

        Ok(())
    }

    /// 删除层
    fn delete_layer(&self, layer_id: &str) -> Result<()> {
        let layer_path = self.get_layer_path(layer_id);
        if Path::new(&layer_path).exists() {
            fs::remove_dir_all(&layer_path).map_err(|e| DockerError::io_error("delete_layer", e.to_string()))?;
        }
        Ok(())
    }

    /// 获取层路径
    fn get_layer_path(&self, layer_id: &str) -> String {
        format!("{}/{}", self.base_path, layer_id)
    }
}
