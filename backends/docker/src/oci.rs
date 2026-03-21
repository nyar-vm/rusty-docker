#![warn(missing_docs)]

//! OCI 运行时规范兼容
//!
//! 实现 OCI (Open Container Initiative) 运行时规范兼容，包括配置文件解析、
//! 生命周期管理等功能。

use std::{fs, path::Path};

use docker_types::{DockerError, Result};

/// OCI 运行时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciConfig {
    /// 版本
    pub version: String,
    /// 进程配置
    pub process: ProcessConfig,
    /// 根文件系统配置
    pub root: RootConfig,
    /// 挂载点配置
    pub mounts: Vec<MountConfig>,
    /// 钩子配置
    pub hooks: Option<HooksConfig>,
    /// Linux 特定配置
    pub linux: Option<LinuxConfig>,
}

/// 进程配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    /// 命令
    pub args: Vec<String>,
    /// 环境变量
    pub env: Vec<String>,
    /// 工作目录
    pub cwd: String,
    /// 终端配置
    pub terminal: bool,
    /// 用户配置
    pub user: UserConfig,
    /// 资源限制
    pub rlimits: Option<Vec<RlimitConfig>>,
}

/// 用户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// UID
    pub uid: u32,
    /// GID
    pub gid: u32,
    /// 附加 GID
    pub additional_gids: Option<Vec<u32>>,
}

/// 资源限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlimitConfig {
    /// 类型
    pub type_: String,
    /// 软限制
    pub soft: u64,
    /// 硬限制
    pub hard: u64,
}

/// 根文件系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootConfig {
    /// 路径
    pub path: String,
    /// 只读
    pub readonly: bool,
}

/// 挂载点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    /// 源
    pub source: Option<String>,
    /// 目标
    pub destination: String,
    /// 类型
    pub type_: String,
    /// 选项
    pub options: Vec<String>,
}

/// 钩子配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// 创建前钩子
    pub prestart: Option<Vec<HookConfig>>,
    /// 创建后钩子
    pub poststart: Option<Vec<HookConfig>>,
    /// 删除前钩子
    pub poststop: Option<Vec<HookConfig>>,
}

/// 钩子配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// 路径
    pub path: String,
    /// 参数
    pub args: Option<Vec<String>>,
    /// 环境变量
    pub env: Option<Vec<String>>,
}

/// Linux 特定配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxConfig {
    /// 命名空间
    pub namespaces: Option<Vec<NamespaceConfig>>,
    /// 设备
    pub devices: Option<Vec<DeviceConfig>>,
    /// 资源限制
    pub resources: Option<ResourcesConfig>,
}

/// 命名空间配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceConfig {
    /// 类型
    pub type_: String,
    /// 路径
    pub path: Option<String>,
}

/// 设备配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// 路径
    pub path: String,
    /// 类型
    pub type_: String,
    /// 主设备号
    pub major: u64,
    /// 次设备号
    pub minor: u64,
    /// 文件模式
    pub file_mode: Option<u32>,
    /// UID
    pub uid: Option<u32>,
    /// GID
    pub gid: Option<u32>,
}

/// 资源限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesConfig {
    /// CPU 配置
    pub cpu: Option<CpuConfig>,
    /// 内存配置
    pub memory: Option<MemoryConfig>,
    /// 块 IO 配置
    pub block_io: Option<BlockIoConfig>,
    /// PID 配置
    pub pids: Option<PidsConfig>,
}

/// CPU 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuConfig {
    /// 共享池
    pub shares: Option<u64>,
    /// 配额
    pub quota: Option<i64>,
    /// 周期
    pub period: Option<u64>,
    /// 实时运行时
    pub rt_runtime: Option<i64>,
    /// 实时周期
    pub rt_period: Option<u64>,
    /// 亲和性
    pub cpus: Option<String>,
    /// 节点亲和性
    pub mems: Option<String>,
}

/// 内存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// 限制
    pub limit: Option<u64>,
    /// 预留
    pub reservation: Option<u64>,
    /// 交换限制
    pub swap: Option<u64>,
    /// 内核限制
    pub kernel: Option<u64>,
    /// 内核 TCP 限制
    pub kernel_tcp: Option<u64>,
    /// OOM 杀手
    pub oom_kill_disable: Option<bool>,
    /// OOM 分数调整
    pub oom_score_adj: Option<i32>,
    /// 禁用分页
    pub disable_oom_killer: Option<bool>,
}

/// 块 IO 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockIoConfig {
    /// 权重
    pub weight: Option<u16>,
    /// 权重设备
    pub weight_device: Option<Vec<WeightDeviceConfig>>,
    /// 读取限制
    pub throttle_read_bps_device: Option<Vec<ThrottleDeviceConfig>>,
    /// 写入限制
    pub throttle_write_bps_device: Option<Vec<ThrottleDeviceConfig>>,
    /// 读取 IOPS 限制
    pub throttle_read_iops_device: Option<Vec<ThrottleDeviceConfig>>,
    /// 写入 IOPS 限制
    pub throttle_write_iops_device: Option<Vec<ThrottleDeviceConfig>>,
}

/// 权重设备配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightDeviceConfig {
    /// 主要设备号
    pub major: u64,
    /// 次要设备号
    pub minor: u64,
    /// 权重
    pub weight: u16,
    /// 权重设备
    pub weight_device: Option<u16>,
}

/// 限流设备配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleDeviceConfig {
    /// 主要设备号
    pub major: u64,
    /// 次要设备号
    pub minor: u64,
    /// 限制
    pub rate: u64,
}

/// PID 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidsConfig {
    /// 限制
    pub limit: u64,
}

/// OCI 运行时
pub struct OciRuntime {
    /// 运行时路径
    runtime_path: String,
}

impl OciRuntime {
    /// 创建新的 OCI 运行时
    pub fn new(runtime_path: &str) -> Self {
        Self { runtime_path: runtime_path.to_string() }
    }

    /// 从默认路径创建 OCI 运行时
    pub fn default() -> Self {
        Self::new("runc")
    }

    /// 加载 OCI 配置文件
    pub fn load_config<P: AsRef<Path>>(&self, path: P) -> Result<OciConfig> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| DockerError::io_error("read_oci_config", e.to_string()))?;

        let config: OciConfig = serde_json::from_str(&content).map_err(|e| DockerError::json_error(e.to_string()))?;

        Ok(config)
    }

    /// 保存 OCI 配置文件
    pub fn save_config<P: AsRef<Path>>(&self, config: &OciConfig, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = serde_json::to_string_pretty(config).map_err(|e| DockerError::json_error(e.to_string()))?;

        fs::write(path, content).map_err(|e| DockerError::io_error("write_oci_config", e.to_string()))?;

        Ok(())
    }

    /// 创建容器
    pub fn create(&self, container_id: &str, config_path: &str, bundle_path: &str) -> Result<()> {
        let output = std::process::Command::new(&self.runtime_path)
            .args(&["create", container_id, bundle_path])
            .output()
            .map_err(|e| DockerError::io_error("oci_create", e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::container_error(format!("OCI create failed: {}", error)));
        }

        Ok(())
    }

    /// 启动容器
    pub fn start(&self, container_id: &str) -> Result<()> {
        let output = std::process::Command::new(&self.runtime_path)
            .args(&["start", container_id])
            .output()
            .map_err(|e| DockerError::io_error("oci_start", e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::container_error(format!("OCI start failed: {}", error)));
        }

        Ok(())
    }

    /// 停止容器
    pub fn stop(&self, container_id: &str, timeout: Option<u32>) -> Result<()> {
        let mut args = vec!["stop"];
        if let Some(timeout) = timeout {
            args.extend(&["--timeout", &timeout.to_string()]);
        }
        args.push(container_id);

        let output = std::process::Command::new(&self.runtime_path)
            .args(&args)
            .output()
            .map_err(|e| DockerError::io_error("oci_stop", e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::container_error(format!("OCI stop failed: {}", error)));
        }

        Ok(())
    }

    /// 删除容器
    pub fn delete(&self, container_id: &str) -> Result<()> {
        let output = std::process::Command::new(&self.runtime_path)
            .args(&["delete", container_id])
            .output()
            .map_err(|e| DockerError::io_error("oci_delete", e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::container_error(format!("OCI delete failed: {}", error)));
        }

        Ok(())
    }

    /// 状态容器
    pub fn state(&self, container_id: &str) -> Result<String> {
        let output = std::process::Command::new(&self.runtime_path)
            .args(&["state", container_id])
            .output()
            .map_err(|e| DockerError::io_error("oci_state", e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::container_error(format!("OCI state failed: {}", error)));
        }

        let state = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(state)
    }
}

/// 从容器配置转换为 OCI 配置
pub fn container_config_to_oci(container_config: &docker_types::ContainerConfig, rootfs_path: &str) -> OciConfig {
    // 构建环境变量
    let env: Vec<String> = container_config.environment.iter().map(|(k, v)| format!("{}={}", k, v)).collect();

    // 构建挂载点
    let mut mounts = vec![
        MountConfig {
            source: Some("proc".to_string()),
            destination: "/proc".to_string(),
            type_: "proc".to_string(),
            options: vec!["nosuid".to_string(), "noexec".to_string(), "nodev".to_string()],
        },
        MountConfig {
            source: Some("sysfs".to_string()),
            destination: "/sys".to_string(),
            type_: "sysfs".to_string(),
            options: vec!["nosuid".to_string(), "noexec".to_string(), "nodev".to_string()],
        },
        MountConfig {
            source: Some("tmpfs".to_string()),
            destination: "/dev".to_string(),
            type_: "tmpfs".to_string(),
            options: vec!["nosuid".to_string(), "strictatime".to_string(), "mode=755".to_string(), "size=65536k".to_string()],
        },
    ];

    // 添加卷挂载
    for volume in &container_config.volumes {
        let source = match volume.mount_type {
            docker_types::MountType::Bind => volume.host_path.clone(),
            docker_types::MountType::Volume => volume.volume_name.clone(),
            _ => None,
        };

        let mount_type = match volume.mount_type {
            docker_types::MountType::Bind => "bind",
            docker_types::MountType::Volume => "volume",
            docker_types::MountType::Tmpfs => "tmpfs",
        };

        let mut options = vec![];
        if volume.read_only {
            options.push("ro".to_string());
        }
        else {
            options.push("rw".to_string());
        }

        mounts.push(MountConfig { source, destination: volume.container_path.clone(), type_: mount_type.to_string(), options });
    }

    // 构建 Linux 配置
    let linux = Some(LinuxConfig {
        namespaces: Some(vec![
            NamespaceConfig { type_: "pid".to_string(), path: None },
            NamespaceConfig { type_: "network".to_string(), path: None },
            NamespaceConfig { type_: "mount".to_string(), path: None },
            NamespaceConfig { type_: "uts".to_string(), path: None },
            NamespaceConfig { type_: "ipc".to_string(), path: None },
        ]),
        devices: Some(vec![
            DeviceConfig {
                path: "/dev/null".to_string(),
                type_: "c".to_string(),
                major: 1,
                minor: 3,
                file_mode: Some(0o666),
                uid: Some(0),
                gid: Some(0),
            },
            DeviceConfig {
                path: "/dev/zero".to_string(),
                type_: "c".to_string(),
                major: 1,
                minor: 5,
                file_mode: Some(0o666),
                uid: Some(0),
                gid: Some(0),
            },
            DeviceConfig {
                path: "/dev/random".to_string(),
                type_: "c".to_string(),
                major: 1,
                minor: 8,
                file_mode: Some(0o666),
                uid: Some(0),
                gid: Some(0),
            },
            DeviceConfig {
                path: "/dev/urandom".to_string(),
                type_: "c".to_string(),
                major: 1,
                minor: 9,
                file_mode: Some(0o666),
                uid: Some(0),
                gid: Some(0),
            },
        ]),
        resources: Some(ResourcesConfig {
            cpu: Some(CpuConfig {
                shares: Some((container_config.resources.cpu_limit * 1024.0) as u64),
                quota: Some((container_config.resources.cpu_limit * 100000.0) as i64),
                period: Some(100000),
                rt_runtime: None,
                rt_period: None,
                cpus: None,
                mems: None,
            }),
            memory: Some(MemoryConfig {
                limit: Some(container_config.resources.memory_limit as u64 * 1024 * 1024),
                reservation: None,
                swap: None,
                kernel: None,
                kernel_tcp: None,
                oom_kill_disable: None,
                oom_score_adj: None,
                disable_oom_killer: None,
            }),
            block_io: None,
            pids: Some(PidsConfig { limit: 4096 }),
        }),
    });

    OciConfig {
        version: "1.0.2-dev".to_string(),
        process: ProcessConfig {
            args: container_config.command.clone(),
            env,
            cwd: "/".to_string(),
            terminal: false,
            user: UserConfig { uid: 0, gid: 0, additional_gids: Some(vec![]) },
            rlimits: None,
        },
        root: RootConfig { path: rootfs_path.to_string(), readonly: false },
        mounts,
        hooks: None,
        linux,
    }
}


