#![warn(missing_docs)]

//! Linux 命名空间管理
//!
//! 提供 Linux 命名空间的创建、管理和切换功能，包括 PID、Mount、Network、
//! UTS、IPC 和 User 命名空间。

use std::{
    fs,
    os::unix::io::RawFd,
    path::{Path, PathBuf},
};

use docker_types::{DockerError, Result};

/// Linux 命名空间类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamespaceType {
    /// PID 命名空间
    Pid,
    /// 网络命名空间
    Network,
    /// 挂载命名空间
    Mount,
    /// UTS 命名空间
    Uts,
    /// IPC 命名空间
    Ipc,
    /// 用户命名空间
    User,
}

impl NamespaceType {
    /// 获取命名空间类型对应的文件名称
    pub fn as_str(&self) -> &'static str {
        match self {
            NamespaceType::Pid => "pid",
            NamespaceType::Network => "net",
            NamespaceType::Mount => "mnt",
            NamespaceType::Uts => "uts",
            NamespaceType::Ipc => "ipc",
            NamespaceType::User => "user",
        }
    }

    /// 获取命名空间类型对应的 clone 标志
    #[cfg(target_os = "linux")]
    pub fn clone_flag(&self) -> nix::sched::CloneFlags {
        match self {
            NamespaceType::Pid => nix::sched::CloneFlags::CLONE_NEWPID,
            NamespaceType::Network => nix::sched::CloneFlags::CLONE_NEWNET,
            NamespaceType::Mount => nix::sched::CloneFlags::CLONE_NEWNS,
            NamespaceType::Uts => nix::sched::CloneFlags::CLONE_NEWUTS,
            NamespaceType::Ipc => nix::sched::CloneFlags::CLONE_NEWIPC,
            NamespaceType::User => nix::sched::CloneFlags::CLONE_NEWUSER,
        }
    }
}

/// 命名空间配置
#[derive(Debug, Clone, Default)]
pub struct NamespaceConfig {
    /// 是否创建新的 PID 命名空间
    pub pid: bool,
    /// 是否创建新的网络命名空间
    pub network: bool,
    /// 是否创建新的挂载命名空间
    pub mount: bool,
    /// 是否创建新的 UTS 命名空间
    pub uts: bool,
    /// 是否创建新的 IPC 命名空间
    pub ipc: bool,
    /// 是否创建新的用户命名空间
    pub user: bool,
    /// 用户命名空间的 UID 映射
    pub uid_mapping: Option<Vec<IdMapping>>,
    /// 用户命名空间的 GID 映射
    pub gid_mapping: Option<Vec<IdMapping>>,
}

impl NamespaceConfig {
    /// 创建默认的命名空间配置，启用所有命名空间
    pub fn all() -> Self {
        Self { pid: true, network: true, mount: true, uts: true, ipc: true, user: false, uid_mapping: None, gid_mapping: None }
    }

    /// 获取组合后的 clone 标志
    #[cfg(target_os = "linux")]
    pub fn clone_flags(&self) -> nix::sched::CloneFlags {
        let mut flags = nix::sched::CloneFlags::empty();
        if self.pid {
            flags |= nix::sched::CloneFlags::CLONE_NEWPID;
        }
        if self.network {
            flags |= nix::sched::CloneFlags::CLONE_NEWNET;
        }
        if self.mount {
            flags |= nix::sched::CloneFlags::CLONE_NEWNS;
        }
        if self.uts {
            flags |= nix::sched::CloneFlags::CLONE_NEWUTS;
        }
        if self.ipc {
            flags |= nix::sched::CloneFlags::CLONE_NEWIPC;
        }
        if self.user {
            flags |= nix::sched::CloneFlags::CLONE_NEWUSER;
        }
        flags
    }
}

/// ID 映射（用于用户命名空间）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdMapping {
    /// 容器内的起始 ID
    pub container_id: u32,
    /// 主机上的起始 ID
    pub host_id: u32,
    /// 映射的 ID 数量
    pub size: u32,
}

impl IdMapping {
    /// 创建新的 ID 映射
    pub fn new(container_id: u32, host_id: u32, size: u32) -> Self {
        Self { container_id, host_id, size }
    }

    /// 格式化 ID 映射为字符串
    pub fn to_string(&self) -> String {
        format!("{} {} {}", self.container_id, self.host_id, self.size)
    }
}

/// 命名空间管理器
pub struct NamespaceManager {
    /// 命名空间存储路径
    base_path: PathBuf,
}

impl NamespaceManager {
    /// 创建新的命名空间管理器
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path).map_err(|e| DockerError::io_error("create_namespace_dir", e.to_string()))?;
        Ok(Self { base_path })
    }

    /// 使用默认路径创建命名空间管理器
    pub fn default() -> Result<Self> {
        Self::new("./rusty-docker/namespaces")
    }

    /// 创建新的命名空间管理器（无参数版本）
    pub fn new() -> Result<Self> {
        Self::default()
    }

    /// 获取命名空间存储路径
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// 获取容器命名空间的路径
    pub fn get_namespace_path(&self, container_id: &str, ns_type: NamespaceType) -> PathBuf {
        self.base_path.join(format!("{}-{}", container_id, ns_type.as_str()))
    }

    /// 获取进程的命名空间路径
    pub fn get_process_namespace_path(pid: i32, ns_type: NamespaceType) -> PathBuf {
        PathBuf::from(format!("/proc/{}/ns/{}", pid, ns_type.as_str()))
    }

    /// 保存进程的命名空间
    pub fn save_namespace(&self, container_id: &str, ns_type: NamespaceType, pid: i32) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let src_path = self.get_process_namespace_path(pid, ns_type);
            let dest_path = self.get_namespace_path(container_id, ns_type);

            if dest_path.exists() {
                fs::remove_file(&dest_path).ok();
            }

            std::os::unix::fs::symlink(&src_path, &dest_path)
                .map_err(|e| DockerError::io_error("save_namespace", e.to_string()))?;

            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    /// 保存进程的所有命名空间
    pub fn save_all_namespaces(&self, container_id: &str, pid: i32, config: &NamespaceConfig) -> Result<()> {
        if config.pid {
            self.save_namespace(container_id, NamespaceType::Pid, pid)?;
        }
        if config.network {
            self.save_namespace(container_id, NamespaceType::Network, pid)?;
        }
        if config.mount {
            self.save_namespace(container_id, NamespaceType::Mount, pid)?;
        }
        if config.uts {
            self.save_namespace(container_id, NamespaceType::Uts, pid)?;
        }
        if config.ipc {
            self.save_namespace(container_id, NamespaceType::Ipc, pid)?;
        }
        if config.user {
            self.save_namespace(container_id, NamespaceType::User, pid)?;
        }
        Ok(())
    }

    /// 进入指定的命名空间
    pub fn enter_namespace(&self, container_id: &str, ns_type: NamespaceType) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let ns_path = self.get_namespace_path(container_id, ns_type);
            Self::enter_namespace_by_path(&ns_path, ns_type)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    /// 通过路径进入命名空间
    #[cfg(target_os = "linux")]
    pub fn enter_namespace_by_path(ns_path: &Path, ns_type: NamespaceType) -> Result<()> {
        use nix::{
            fcntl::{OFlag, open},
            sys::stat::Mode,
            unistd::close,
        };

        let fd = open(ns_path, OFlag::O_RDONLY, Mode::empty())
            .map_err(|e| DockerError::io_error("open_namespace", e.to_string()))?;

        let result = Self::enter_namespace_by_fd(fd, ns_type);

        close(fd).ok();

        result
    }

    /// 通过文件描述符进入命名空间
    #[cfg(target_os = "linux")]
    pub fn enter_namespace_by_fd(fd: RawFd, ns_type: NamespaceType) -> Result<()> {
        nix::sched::setns(fd, ns_type.clone_flag()).map_err(|e| DockerError::io_error("set_namespace", e.to_string()))?;
        Ok(())
    }

    /// 进入容器的所有命名空间
    pub fn enter_all_namespaces(&self, container_id: &str, config: &NamespaceConfig) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if config.pid {
                self.enter_namespace(container_id, NamespaceType::Pid)?;
            }
            if config.network {
                self.enter_namespace(container_id, NamespaceType::Network)?;
            }
            if config.mount {
                self.enter_namespace(container_id, NamespaceType::Mount)?;
            }
            if config.uts {
                self.enter_namespace(container_id, NamespaceType::Uts)?;
            }
            if config.ipc {
                self.enter_namespace(container_id, NamespaceType::Ipc)?;
            }
            if config.user {
                self.enter_namespace(container_id, NamespaceType::User)?;
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    /// 创建命名空间并执行子进程
    #[cfg(target_os = "linux")]
    pub fn create_namespaces<F>(&self, config: &NamespaceConfig, child_func: F) -> Result<i32>
    where
        F: FnOnce() -> i32 + Send + 'static,
    {
        use nix::{
            sched::{CloneFlags, clone},
            sys::wait::waitpid,
            unistd::Pid,
        };

        const STACK_SIZE: usize = 1024 * 1024;
        let mut stack = vec![0u8; STACK_SIZE];

        let flags = config.clone_flags();

        let child_pid =
            clone(Box::new(child_func), &mut stack, flags, None).map_err(|e| DockerError::io_error("clone", e.to_string()))?;

        Ok(child_pid.as_raw())
    }

    /// 创建单个命名空间
    #[cfg(target_os = "linux")]
    pub fn create_namespace(&self, ns_type: NamespaceType) -> Result<i32> {
        use nix::sched::{CloneFlags, clone};

        const STACK_SIZE: usize = 1024 * 1024;
        let mut stack = vec![0u8; STACK_SIZE];

        let flags = ns_type.clone_flag();

        let child_pid = clone(
            Box::new(|| {
                // 子进程保持运行，直到被终止
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }),
            &mut stack,
            flags,
            None,
        )
        .map_err(|e| DockerError::io_error("clone", e.to_string()))?;

        Ok(child_pid.as_raw())
    }

    /// 创建单个命名空间（非 Linux 平台）
    #[cfg(not(target_os = "linux"))]
    pub fn create_namespace(&self, _ns_type: NamespaceType) -> Result<i32> {
        // 在非 Linux 平台上，返回当前进程 ID 作为模拟
        Ok(std::process::id() as i32)
    }

    /// 在当前进程中创建并进入命名空间
    #[cfg(target_os = "linux")]
    pub fn unshare_namespaces(&self, config: &NamespaceConfig) -> Result<()> {
        let flags = config.clone_flags();
        nix::sched::unshare(flags).map_err(|e| DockerError::io_error("unshare", e.to_string()))?;
        Ok(())
    }

    /// 设置用户命名空间的 UID 映射
    #[cfg(target_os = "linux")]
    pub fn set_uid_mapping(&self, pid: i32, mappings: &[IdMapping]) -> Result<()> {
        use std::{fs::OpenOptions, io::Write};

        let uid_map_path = format!("/proc/{}/uid_map", pid);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&uid_map_path)
            .map_err(|e| DockerError::io_error("open_uid_map", e.to_string()))?;

        let mapping_str = mappings.iter().map(|m| m.to_string()).collect::<Vec<_>>().join("\n");

        file.write_all(mapping_str.as_bytes()).map_err(|e| DockerError::io_error("write_uid_map", e.to_string()))?;

        Ok(())
    }

    /// 设置用户命名空间的 GID 映射
    #[cfg(target_os = "linux")]
    pub fn set_gid_mapping(&self, pid: i32, mappings: &[IdMapping]) -> Result<()> {
        use std::{fs::OpenOptions, io::Write};

        let gid_map_path = format!("/proc/{}/gid_map", pid);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&gid_map_path)
            .map_err(|e| DockerError::io_error("open_gid_map", e.to_string()))?;

        let mapping_str = mappings.iter().map(|m| m.to_string()).collect::<Vec<_>>().join("\n");

        file.write_all(mapping_str.as_bytes()).map_err(|e| DockerError::io_error("write_gid_map", e.to_string()))?;

        Ok(())
    }

    /// 删除容器的命名空间
    pub fn cleanup_namespaces(&self, container_id: &str) -> Result<()> {
        let types = [
            NamespaceType::Pid,
            NamespaceType::Network,
            NamespaceType::Mount,
            NamespaceType::Uts,
            NamespaceType::Ipc,
            NamespaceType::User,
        ];

        for ns_type in types {
            let path = self.get_namespace_path(container_id, ns_type);
            if path.exists() {
                fs::remove_file(&path).map_err(|e| DockerError::io_error("cleanup_namespace", e.to_string()))?;
            }
        }

        Ok(())
    }
}


