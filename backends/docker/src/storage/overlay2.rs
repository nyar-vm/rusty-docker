#![warn(missing_docs)]

//! Overlay2 文件系统驱动
//!
//! 提供容器文件系统的 overlay2 挂载支持，包括：
//! - 创建和管理 overlay2 挂载
//! - 处理 whiteout 文件
//! - 准备容器根文件系统
//! - 清理容器文件系统

use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use docker_types::{DockerError, Result};
use nix::{
    mount::{MsFlags, mount},
    sys::stat::makedev,
};

/// Overlay2 文件系统驱动
pub struct Overlay2Driver {
    /// 存储根路径
    storage_root: PathBuf,
}

impl Overlay2Driver {
    /// 创建新的 Overlay2Driver 实例
    pub fn new(storage_root: impl Into<PathBuf>) -> Self {
        Self { storage_root: storage_root.into() }
    }

    /// 获取容器存储目录
    pub fn get_container_dir(&self, container_id: &str) -> PathBuf {
        self.storage_root.join("containers").join(container_id)
    }

    /// 获取镜像存储目录
    pub fn get_image_dir(&self, image_id: &str) -> PathBuf {
        self.storage_root.join("images").join(image_id)
    }

    /// 获取容器的 lowerdir 路径
    pub fn get_lowerdir(&self, container_id: &str) -> PathBuf {
        self.get_container_dir(container_id).join("lower")
    }

    /// 获取容器的 upperdir 路径
    pub fn get_upperdir(&self, container_id: &str) -> PathBuf {
        self.get_container_dir(container_id).join("upper")
    }

    /// 获取容器的 workdir 路径
    pub fn get_workdir(&self, container_id: &str) -> PathBuf {
        self.get_container_dir(container_id).join("work")
    }

    /// 获取容器的 merged 路径（挂载点）
    pub fn get_merged_dir(&self, container_id: &str) -> PathBuf {
        self.get_container_dir(container_id).join("merged")
    }

    /// 初始化容器的 overlay2 目录结构
    pub fn initialize_container_dirs(&self, container_id: &str) -> Result<()> {
        let container_dir = self.get_container_dir(container_id);
        fs::create_dir_all(&container_dir).map_err(|e| DockerError::io_error("create container dir", e.to_string()))?;

        let dirs = [
            self.get_lowerdir(container_id),
            self.get_upperdir(container_id),
            self.get_workdir(container_id),
            self.get_merged_dir(container_id),
        ];

        for dir in &dirs {
            fs::create_dir_all(dir).map_err(|e| DockerError::io_error("create overlay dir", e.to_string()))?;
        }

        Ok(())
    }

    /// 创建 overlay2 挂载
    pub fn mount_overlay(&self, container_id: &str, image_id: &str) -> Result<()> {
        let image_dir = self.get_image_dir(image_id);
        if !image_dir.exists() {
            return Err(DockerError::not_found("image", &format!("Image not found: {}", image_id)));
        }

        let lowerdir = self.get_lowerdir(container_id);
        let upperdir = self.get_upperdir(container_id);
        let workdir = self.get_workdir(container_id);
        let merged_dir = self.get_merged_dir(container_id);

        let lowerdirs = vec![image_dir.to_str().ok_or_else(|| DockerError::internal("Invalid image path"))?];

        let lowerdir_str = lowerdirs.join(":");

        let options = format!(
            "lowerdir={},upperdir={},workdir={}",
            lowerdir_str,
            upperdir.to_str().ok_or_else(|| DockerError::internal("Invalid upperdir path"))?,
            workdir.to_str().ok_or_else(|| DockerError::internal("Invalid workdir path"))?
        );

        mount(Some("overlay"), merged_dir.as_path(), Some("overlay"), MsFlags::empty(), Some(options.as_str()))
            .map_err(|e| DockerError::internal(format!("Failed to mount overlay: {}", e)))?;

        Ok(())
    }

    /// 卸载 overlay2 挂载
    pub fn umount_overlay(&self, container_id: &str) -> Result<()> {
        let merged_dir = self.get_merged_dir(container_id);
        if merged_dir.exists() {
            nix::mount::umount(merged_dir.as_path())
                .map_err(|e| DockerError::internal(format!("Failed to umount overlay: {}", e)))?;
        }
        Ok(())
    }

    /// 准备容器根文件系统，挂载必要的文件系统
    pub fn prepare_container_rootfs(&self, container_id: &str) -> Result<()> {
        let merged_dir = self.get_merged_dir(container_id);

        let mounts = [
            ("proc", "proc", "/proc", vec!["nosuid", "noexec", "nodev"]),
            ("sysfs", "sysfs", "/sys", vec!["nosuid", "noexec", "nodev"]),
            ("tmpfs", "tmpfs", "/dev", vec!["nosuid", "strictatime", "mode=755", "size=65536k"]),
            ("devpts", "devpts", "/dev/pts", vec!["nosuid", "noexec", "newinstance", "ptmxmode=0666", "mode=0620", "gid=5"]),
            ("tmpfs", "tmpfs", "/dev/shm", vec!["nosuid", "noexec", "nodev", "mode=1777"]),
            ("tmpfs", "tmpfs", "/run", vec!["nosuid", "noexec", "nodev", "mode=755"]),
            ("tmpfs", "tmpfs", "/tmp", vec!["nosuid", "nodev"]),
        ];

        for (source, fstype, mountpoint, options) in mounts {
            let target_path = merged_dir.join(mountpoint.strip_prefix('/').unwrap_or(mountpoint));
            fs::create_dir_all(&target_path).map_err(|e| DockerError::io_error("create mountpoint", e.to_string()))?;

            let options_str = options.join(",");
            mount(
                Some(source),
                target_path.as_path(),
                Some(fstype),
                MsFlags::empty(),
                if options_str.is_empty() { None } else { Some(options_str.as_str()) },
            )
            .map_err(|e| DockerError::internal(format!("Failed to mount {}: {}", fstype, e)))?;
        }

        self.create_dev_nodes(&merged_dir)?;

        Ok(())
    }

    /// 创建必要的设备节点
    fn create_dev_nodes(&self, merged_dir: &Path) -> Result<()> {
        let dev_dir = merged_dir.join("dev");

        let nodes = [
            ("null", 1, 3, 0o666),
            ("zero", 1, 5, 0o666),
            ("full", 1, 7, 0o666),
            ("random", 1, 8, 0o666),
            ("urandom", 1, 9, 0o666),
            ("tty", 5, 0, 0o666),
            ("console", 5, 1, 0o600),
            ("ptmx", 5, 2, 0o666),
        ];

        for (name, major, minor, mode) in nodes {
            let node_path = dev_dir.join(name);
            if !node_path.exists() {
                #[cfg(target_os = "linux")]
                {
                    use std::os::unix::fs::{FileTypeExt, mknod};
                    let file_type = fs::FileType::from_raw(libc::S_IFCHR);
                    let dev = makedev(major, minor);
                    mknod(&node_path, file_type, dev).map_err(|e| DockerError::io_error("mknod", e.to_string()))?;

                    use std::os::unix::fs::PermissionsExt;
                    let mut perms =
                        fs::metadata(&node_path).map_err(|e| DockerError::io_error("metadata", e.to_string()))?.permissions();
                    perms.set_mode(mode);
                    fs::set_permissions(&node_path, perms).map_err(|e| DockerError::io_error("chmod", e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    /// 清理容器根文件系统，卸载挂载的文件系统
    pub fn cleanup_container_rootfs(&self, container_id: &str) -> Result<()> {
        let merged_dir = self.get_merged_dir(container_id);

        let mountpoints = ["/dev/pts", "/dev/shm", "/dev", "/proc", "/sys", "/run", "/tmp"];

        for mountpoint in mountpoints {
            let target_path = merged_dir.join(mountpoint.strip_prefix('/').unwrap_or(mountpoint));
            if target_path.exists() {
                let _ = nix::mount::umount(target_path.as_path());
            }
        }

        Ok(())
    }

    /// 清理 whiteout 文件
    pub fn cleanup_whiteouts(&self, container_id: &str) -> Result<()> {
        let upperdir = self.get_upperdir(container_id);
        self.cleanup_whiteouts_in_dir(&upperdir)?;
        Ok(())
    }

    /// 递归清理目录中的 whiteout 文件
    fn cleanup_whiteouts_in_dir(&self, dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(dir).map_err(|e| DockerError::io_error("read_dir", e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| DockerError::io_error("read entry", e.to_string()))?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.starts_with(".wh.") {
                fs::remove_file(&path).map_err(|e| DockerError::io_error("remove whiteout", e.to_string()))?;
            }
            else if path.is_dir() {
                self.cleanup_whiteouts_in_dir(&path)?;
            }
        }

        Ok(())
    }

    /// 删除容器的所有文件和目录
    pub fn delete_container(&self, container_id: &str) -> Result<()> {
        let container_dir = self.get_container_dir(container_id);
        if container_dir.exists() {
            fs::remove_dir_all(&container_dir).map_err(|e| DockerError::io_error("remove container dir", e.to_string()))?;
        }
        Ok(())
    }

    /// 获取容器的差异（upperdir 中的内容）
    pub fn get_container_diff(&self, container_id: &str) -> Result<HashSet<PathBuf>> {
        let upperdir = self.get_upperdir(container_id);
        let mut diff = HashSet::new();
        self.collect_diff_files(&upperdir, &upperdir, &mut diff)?;
        Ok(diff)
    }

    /// 递归收集差异文件
    fn collect_diff_files(&self, base: &Path, current: &Path, diff: &mut HashSet<PathBuf>) -> Result<()> {
        if !current.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(current).map_err(|e| DockerError::io_error("read_dir", e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| DockerError::io_error("read entry", e.to_string()))?;
            let path = entry.path();
            let rel_path = path.strip_prefix(base).map_err(|e| DockerError::internal(format!("Strip prefix error: {}", e)))?;

            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if !file_name_str.starts_with(".wh.") {
                diff.insert(rel_path.to_path_buf());
                if path.is_dir() {
                    self.collect_diff_files(base, &path, diff)?;
                }
            }
        }

        Ok(())
    }
}

impl Drop for Overlay2Driver {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_overlay2_driver_creation() {
        let temp_dir = tempdir().unwrap();
        let driver = Overlay2Driver::new(temp_dir.path());
        assert_eq!(driver.storage_root, temp_dir.path());
    }

    #[test]
    fn test_get_container_dir() {
        let temp_dir = tempdir().unwrap();
        let driver = Overlay2Driver::new(temp_dir.path());
        let container_dir = driver.get_container_dir("test-container");
        assert_eq!(container_dir, temp_dir.path().join("containers").join("test-container"));
    }
}
