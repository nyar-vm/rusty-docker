#![warn(missing_docs)]

//! 控制组 v2 管理
//!
//! 提供 cgroup v2 的完整实现，包括资源限制、进程管理和控制组生命周期管理。
//! 支持以下资源限制：
//! - 内存限制 (memory.max)
//! - CPU 限制 (cpu.max)
//! - 块 IO 限制 (io.max)
//! - 进程数限制 (pids.max)

#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};

use docker_types::{DockerError, ResourceLimits, Result};

/// 控制组版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CgroupVersion {
    /// 控制组 v1（传统版本）
    V1,
    /// 控制组 v2（现代版本）
    V2,
}

/// 控制组管理器
pub struct CgroupManager {
    #[cfg(target_os = "linux")]
    /// 控制组根路径
    cgroup_root: PathBuf,
    #[cfg(target_os = "linux")]
    /// 控制组版本
    version: CgroupVersion,
}

impl CgroupManager {
    /// 创建新的控制组管理器
    ///
    /// 自动检测系统使用的 cgroup 版本并初始化相应的管理器。
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let cgroup_root = PathBuf::from("/sys/fs/cgroup");

            if !cgroup_root.exists() {
                return Err(DockerError::io_error("cgroup_not_found", "Cgroup directory not found"));
            }

            let version = Self::detect_version(&cgroup_root)?;

            Ok(Self { cgroup_root, version })
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(Self {})
        }
    }

    #[cfg(target_os = "linux")]
    /// 检测系统使用的 cgroup 版本
    fn detect_version(cgroup_root: &Path) -> Result<CgroupVersion> {
        let cgroup2_mount = cgroup_root.join("cgroup.controllers");
        if cgroup2_mount.exists() {
            Ok(CgroupVersion::V2)
        } else {
            Ok(CgroupVersion::V1)
        }
    }

    #[cfg(target_os = "linux")]
    /// 获取控制组路径
    fn get_cgroup_path(&self, container_id: &str) -> PathBuf {
        self.cgroup_root.join("rusty-docker").join(container_id)
    }

    #[cfg(target_os = "linux")]
    /// 检查是否使用 cgroup v2
    fn is_v2(&self) -> bool {
        self.version == CgroupVersion::V2
    }

    /// 创建控制组并设置资源限制
    ///
    /// # 参数
    /// * `container_id` - 容器唯一标识符
    /// * `limits` - 资源限制配置
    pub fn create_cgroup(&self, container_id: &str, limits: &ResourceLimits) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let cgroup_path = self.get_cgroup_path(container_id);
            fs::create_dir_all(&cgroup_path).map_err(|e| DockerError::io_error("create_cgroup", e.to_string()))?;

            if self.is_v2() {
                self.setup_v2_cgroup(&cgroup_path, limits)?;
            } else {
                self.setup_v1_cgroup(&cgroup_path, limits)?;
            }

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    #[cfg(target_os = "linux")]
    /// 设置 cgroup v2 资源限制
    fn setup_v2_cgroup(&self, cgroup_path: &Path, limits: &ResourceLimits) -> Result<()> {
        self.set_v2_memory_limit(cgroup_path, limits.memory_limit)?;
        self.set_v2_cpu_limit(cgroup_path, limits.cpu_limit)?;
        self.set_v2_io_limit(cgroup_path, limits.storage_limit)?;
        self.set_v2_pids_limit(cgroup_path)?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    /// 设置 cgroup v1 资源限制（向后兼容）
    fn setup_v1_cgroup(&self, cgroup_path: &Path, limits: &ResourceLimits) -> Result<()> {
        self.set_v1_memory_limit(cgroup_path, limits.memory_limit)?;
        self.set_v1_cpu_limit(cgroup_path, limits.cpu_limit)?;
        self.set_v1_io_limit(cgroup_path, limits.storage_limit)?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    /// 设置 cgroup v2 内存限制
    fn set_v2_memory_limit(&self, cgroup_path: &Path, memory_limit: u32) -> Result<()> {
        let limit_bytes = (memory_limit as u64) * 1024 * 1024;
        let memory_max_path = cgroup_path.join("memory.max");
        fs::write(&memory_max_path, limit_bytes.to_string())
            .map_err(|e| DockerError::io_error("set_memory_limit", e.to_string()))?;

        let memory_high_path = cgroup_path.join("memory.high");
        let high_limit = (limit_bytes as f64 * 0.9) as u64;
        fs::write(&memory_high_path, high_limit.to_string())
            .map_err(|e| DockerError::io_error("set_memory_high", e.to_string()))?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    /// 设置 cgroup v2 CPU 限制
    fn set_v2_cpu_limit(&self, cgroup_path: &Path, cpu_limit: f64) -> Result<()> {
        let period = 100000;
        let quota = (cpu_limit * period as f64) as i64;
        let cpu_max_path = cgroup_path.join("cpu.max");
        let cpu_max_value = format!("{} {}", quota, period);
        fs::write(&cpu_max_path, cpu_max_value)
            .map_err(|e| DockerError::io_error("set_cpu_limit", e.to_string()))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    /// 设置 cgroup v2 块 IO 限制
    fn set_v2_io_limit(&self, cgroup_path: &Path, storage_limit: u32) -> Result<()> {
        let io_max_path = cgroup_path.join("io.max");
        let limit_bps = (storage_limit as u64) * 1024 * 1024;
        let io_max_value = format!("{}:{} rbps={} wbps={}", 8, 0, limit_bps, limit_bps);
        fs::write(&io_max_path, io_max_value)
            .map_err(|e| DockerError::io_error("set_io_limit", e.to_string()))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    /// 设置 cgroup v2 进程数限制
    fn set_v2_pids_limit(&self, cgroup_path: &Path) -> Result<()> {
        let pids_max_path = cgroup_path.join("pids.max");
        fs::write(&pids_max_path, "4096")
            .map_err(|e| DockerError::io_error("set_pids_limit", e.to_string()))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    /// 设置 cgroup v1 内存限制
    fn set_v1_memory_limit(&self, cgroup_path: &Path, memory_limit: u32) -> Result<()> {
        let memory_dir = cgroup_path.join("memory");
        fs::create_dir_all(&memory_dir).map_err(|e| DockerError::io_error("create_memory_dir", e.to_string()))?;

        let limit_bytes = (memory_limit as u64) * 1024 * 1024;
        let memory_limit_path = memory_dir.join("memory.limit_in_bytes");
        fs::write(&memory_limit_path, limit_bytes.to_string())
            .map_err(|e| DockerError::io_error("set_memory_limit_v1", e.to_string()))?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    /// 设置 cgroup v1 CPU 限制
    fn set_v1_cpu_limit(&self, cgroup_path: &Path, cpu_limit: f64) -> Result<()> {
        let cpu_dir = cgroup_path.join("cpu");
        fs::create_dir_all(&cpu_dir).map_err(|e| DockerError::io_error("create_cpu_dir", e.to_string()))?;

        let period = 100000;
        let quota = (cpu_limit * period as f64) as i64;

        let period_path = cpu_dir.join("cpu.cfs_period_us");
        fs::write(&period_path, period.to_string())
            .map_err(|e| DockerError::io_error("set_cpu_period_v1", e.to_string()))?;

        let quota_path = cpu_dir.join("cpu.cfs_quota_us");
        fs::write(&quota_path, quota.to_string())
            .map_err(|e| DockerError::io_error("set_cpu_quota_v1", e.to_string()))?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    /// 设置 cgroup v1 块 IO 限制
    fn set_v1_io_limit(&self, cgroup_path: &Path, storage_limit: u32) -> Result<()> {
        let blkio_dir = cgroup_path.join("blkio");
        fs::create_dir_all(&blkio_dir).map_err(|e| DockerError::io_error("create_blkio_dir", e.to_string()))?;

        let limit_bps = (storage_limit as u64) * 1024 * 1024;
        let io_value = format!("8:0 {}", limit_bps);

        let read_path = blkio_dir.join("blkio.throttle.read_bps_device");
        fs::write(&read_path, &io_value)
            .map_err(|e| DockerError::io_error("set_io_read_v1", e.to_string()))?;

        let write_path = blkio_dir.join("blkio.throttle.write_bps_device");
        fs::write(&write_path, io_value)
            .map_err(|e| DockerError::io_error("set_io_write_v1", e.to_string()))?;

        Ok(())
    }

    /// 将进程添加到控制组
    ///
    /// # 参数
    /// * `container_id` - 容器唯一标识符
    /// * `pid` - 进程 ID
    pub fn add_process(&self, container_id: &str, pid: u32) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let cgroup_path = self.get_cgroup_path(container_id);

            let procs_path = if self.is_v2() {
                cgroup_path.join("cgroup.procs")
            } else {
                cgroup_path.join("cgroup.procs")
            };

            fs::write(&procs_path, pid.to_string())
                .map_err(|e| DockerError::io_error("add_process", e.to_string()))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    /// 删除控制组
    ///
    /// # 参数
    /// * `container_id` - 容器唯一标识符
    pub fn delete_cgroup(&self, container_id: &str) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let cgroup_path = self.get_cgroup_path(container_id);
            fs::remove_dir_all(&cgroup_path)
                .map_err(|e| DockerError::io_error("delete_cgroup", e.to_string()))?;
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    /// 获取控制组版本
    pub fn version(&self) -> Option<CgroupVersion> {
        #[cfg(target_os = "linux")]
        {
            Some(self.version)
        }

        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }
}

impl Default for CgroupManager {
    fn default() -> Self {
        Self::new().expect("Failed to create CgroupManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_cgroup_manager_creation() {
        let manager = CgroupManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_cgroup_version_enum() {
        let v1 = CgroupVersion::V1;
        let v2 = CgroupVersion::V2;
        assert_ne!(v1, v2);
        assert_eq!(v1, CgroupVersion::V1);
        assert_eq!(v2, CgroupVersion::V2);
    }
}
