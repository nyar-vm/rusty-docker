#![warn(missing_docs)]

//! 控制组管理

#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;

use docker_types::{DockerError, ResourceLimits, Result};

/// 控制组管理器
pub struct CgroupManager {
    #[cfg(target_os = "linux")]
    /// 控制组根路径
    cgroup_root: String,
}

impl CgroupManager {
    /// 创建新的控制组管理器
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let cgroup_root = "/sys/fs/cgroup".to_string();

            // 检查控制组目录是否存在
            if !Path::new(&cgroup_root).exists() {
                return Err(DockerError::io_error(
                    "cgroup_not_found",
                    "Cgroup directory not found",
                ));
            }

            Ok(Self { cgroup_root })
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，返回一个空的控制组管理器
            Ok(Self {})
        }
    }

    /// 创建控制组
    pub fn create_cgroup(&self, container_id: &str, limits: &ResourceLimits) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let cgroup_path = format!("{}/rusty-docker/{}", self.cgroup_root, container_id);

            // 创建控制组目录
            fs::create_dir_all(&cgroup_path)
                .map_err(|e| DockerError::io_error("operation", e.to_string()))?;

            // 设置 CPU 限制
            self.set_cpu_limit(&cgroup_path, limits.cpu_limit)?;

            // 设置内存限制
            self.set_memory_limit(&cgroup_path, limits.memory_limit)?;

            // 设置存储限制
            self.set_storage_limit(&cgroup_path, limits.storage_limit)?;

            // 设置网络限制
            self.set_network_limit(&cgroup_path, limits.network_limit)?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，什么都不做
            Ok(())
        }
    }

    /// 设置 CPU 限制
    pub fn set_cpu_limit(&self, cgroup_path: &str, cpu_limit: f64) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let cpu_path = format!("{}/cpu/cpu.cfs_quota_us", cgroup_path);
            let cpu_period = format!("{}/cpu/cpu.cfs_period_us", cgroup_path);

            // 设置周期为 100ms
            fs::write(cpu_period, "100000")
                .map_err(|e| DockerError::io_error("operation", e.to_string()))?;

            // 设置配额
            let quota = (cpu_limit * 100000.0) as i32;
            fs::write(cpu_path, quota.to_string())
                .map_err(|e| DockerError::io_error("operation", e.to_string()))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，什么都不做
            Ok(())
        }
    }

    /// 设置内存限制
    pub fn set_memory_limit(&self, cgroup_path: &str, memory_limit: u32) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let memory_path = format!("{}/memory/memory.limit_in_bytes", cgroup_path);

            // 转换为字节
            let limit_bytes = memory_limit * 1024 * 1024;
            fs::write(memory_path, limit_bytes.to_string())
                .map_err(|e| DockerError::io_error("operation", e.to_string()))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，什么都不做
            Ok(())
        }
    }

    /// 设置存储限制
    pub fn set_storage_limit(&self, cgroup_path: &str, storage_limit: u32) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let blkio_path = format!("{}/blkio/blkio.throttle.read_bps_device", cgroup_path);
            let blkio_write_path = format!("{}/blkio/blkio.throttle.write_bps_device", cgroup_path);

            // 设置存储限制（单位：字节/秒）
            let limit = storage_limit * 1024 * 1024; // 转换为 MB/s
            // 对所有设备设置限制
            std::fs::write(blkio_path, "8:0 " + &limit.to_string())
                .map_err(|e| DockerError::io_error("operation", e.to_string()))?;
            std::fs::write(blkio_write_path, "8:0 " + &limit.to_string())
                .map_err(|e| DockerError::io_error("operation", e.to_string()))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，什么都不做
            Ok(())
        }
    }

    /// 设置网络限制
    pub fn set_network_limit(&self, cgroup_path: &str, network_limit: u32) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            // 网络限制可以通过 net_cls 控制组实现
            // 这里简化实现，实际项目中可能需要更复杂的网络限制
            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，什么都不做
            Ok(())
        }
    }

    /// 将进程添加到控制组
    pub fn add_process(&self, container_id: &str, pid: u32) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let cgroup_path = format!("{}/rusty-docker/{}", self.cgroup_root, container_id);
            let tasks_path = format!("{}/cgroup.procs", cgroup_path);

            fs::write(tasks_path, pid.to_string())
                .map_err(|e| DockerError::io_error("operation", e.to_string()))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，什么都不做
            Ok(())
        }
    }

    /// 删除控制组
    pub fn delete_cgroup(&self, container_id: &str) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let cgroup_path = format!("{}/rusty-docker/{}", self.cgroup_root, container_id);

            fs::remove_dir_all(&cgroup_path)
                .map_err(|e| DockerError::io_error("operation", e.to_string()))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，什么都不做
            Ok(())
        }
    }
}
