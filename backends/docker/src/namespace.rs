#![warn(missing_docs)]

//! 命名空间管理

use std::fs;

use docker_types::{DockerError, Result};

/// 命名空间类型
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// 命名空间管理
pub struct NamespaceManager {
    /// 命名空间路径
    namespace_path: String,
}

impl NamespaceManager {
    /// 创建新的命名空间管理器
    pub fn new() -> Result<Self> {
        let namespace_path = "./rusty-docker/namespaces".to_string();

        // 创建命名空间目录
        fs::create_dir_all(&namespace_path)
            .map_err(|e| DockerError::io_error("operation", e.to_string()))?;

        Ok(Self { namespace_path })
    }

    /// 创建新的命名空间
    pub fn create_namespace(&self, ns_type: NamespaceType) -> Result<i32> {
        #[cfg(target_os = "linux")]
        {
            use nix::sched;
            use nix::unistd;

            // 创建子进程
            match unistd::fork() {
                Ok(unistd::ForkResult::Parent { child }) => {
                    // 在父进程中返回子进程 PID
                    Ok(child.as_raw())
                }
                Ok(unistd::ForkResult::Child) => {
                    // 在子进程中创建命名空间
                    let flags = match ns_type {
                        NamespaceType::Pid => sched::CloneFlags::CLONE_NEWPID,
                        NamespaceType::Network => sched::CloneFlags::CLONE_NEWNET,
                        NamespaceType::Mount => sched::CloneFlags::CLONE_NEWNS,
                        NamespaceType::Uts => sched::CloneFlags::CLONE_NEWUTS,
                        NamespaceType::Ipc => sched::CloneFlags::CLONE_NEWIPC,
                        NamespaceType::User => sched::CloneFlags::CLONE_NEWUSER,
                    };

                    // 执行命名空间切换
                    if let Err(e) = sched::unshare(flags) {
                        eprintln!("Failed to create namespace: {}", e);
                        std::process::exit(1);
                    }

                    // 子进程继续运行
                    Ok(0)
                }
                Err(e) => Err(DockerError::io_error(
                    "fork_failed",
                    format!("Fork failed: {}", e),
                )),
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，返回模拟的 PID
            Ok(12345)
        }
    }

    /// 进入命名空间
    pub fn enter_namespace(&self, ns_path: &str) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use nix::fcntl;
            use nix::sched;
            use nix::unistd;

            // 打开命名空间文件
            let fd = fcntl::open(
                ns_path,
                fcntl::OFlag::O_RDONLY,
                nix::sys::stat::Mode::empty(),
            )
            .map_err(|e| {
                DockerError::io_error(
                    "open_ns_file",
                    format!("Failed to open namespace file: {}", e),
                )
            })?;

            // 进入命名空间
            let ns_type = if ns_path.ends_with("-pid") {
                sched::CloneFlags::CLONE_NEWPID
            } else if ns_path.ends_with("-net") {
                sched::CloneFlags::CLONE_NEWNET
            } else if ns_path.ends_with("-mnt") {
                sched::CloneFlags::CLONE_NEWNS
            } else if ns_path.ends_with("-uts") {
                sched::CloneFlags::CLONE_NEWUTS
            } else if ns_path.ends_with("-ipc") {
                sched::CloneFlags::CLONE_NEWIPC
            } else if ns_path.ends_with("-user") {
                sched::CloneFlags::CLONE_NEWUSER
            } else {
                return Err(DockerError::io_error(
                    "unknown_ns_type",
                    "Unknown namespace type",
                ));
            };

            if let Err(e) = sched::setns(fd, ns_type) {
                return Err(DockerError::io_error(
                    "enter_ns",
                    format!("Failed to enter namespace: {}", e),
                ));
            }

            // 关闭文件描述符
            unistd::close(fd).ok();

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 在非 Linux 平台上，直接返回成功
            Ok(())
        }
    }

    /// 获取命名空间路径
    pub fn get_namespace_path(&self, container_id: &str, ns_type: NamespaceType) -> String {
        let ns_name = match ns_type {
            NamespaceType::Pid => "pid",
            NamespaceType::Network => "net",
            NamespaceType::Mount => "mnt",
            NamespaceType::Uts => "uts",
            NamespaceType::Ipc => "ipc",
            NamespaceType::User => "user",
        };

        format!("{}/{}-{}", self.namespace_path, container_id, ns_name)
    }

    /// 保存命名空间
    pub fn save_namespace(
        &self,
        container_id: &str,
        ns_type: NamespaceType,
        pid: i32,
    ) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let ns_path = self.get_namespace_path(container_id, ns_type);
            let src_path = format!(
                "/proc/{}/ns/{}",
                pid,
                match ns_type {
                    NamespaceType::Pid => "pid",
                    NamespaceType::Network => "net",
                    NamespaceType::Mount => "mnt",
                    NamespaceType::Uts => "uts",
                    NamespaceType::Ipc => "ipc",
                    NamespaceType::User => "user",
                }
            );

            // 创建符号链接
            if std::path::Path::new(&ns_path).exists() {
                std::fs::remove_file(&ns_path).ok();
            }
            std::os::unix::fs::symlink(src_path, ns_path)
                .map_err(|e| DockerError::io_error("operation", e.to_string()))?;

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }
}
