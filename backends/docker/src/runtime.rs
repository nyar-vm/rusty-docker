#![warn(missing_docs)]

//! 容器运行时

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::cgroup::CgroupManager;
use crate::namespace::{NamespaceManager, NamespaceType};
use crate::storage::StorageService;

use docker_types::{
    ContainerConfig, ContainerInfo, ContainerStatus, DockerError, NetworkConfig, NetworkInfo,
    ResourceLimits, Result,
};
#[cfg(target_os = "linux")]
use nix::sys::signal;
#[cfg(target_os = "linux")]
use nix::unistd;

/// 容器运行时
pub struct ContainerRuntime {
    /// 命名空间管理器
    namespace_manager: NamespaceManager,
    /// 控制组管理器
    cgroup_manager: CgroupManager,
    /// 存储服务
    storage: Arc<StorageService>,
    /// 容器列表
    containers: Arc<RwLock<Vec<ContainerInfo>>>,
}

impl ContainerRuntime {
    /// 创建新的容器运行时
    pub fn new(storage: Arc<StorageService>) -> Result<Self> {
        let namespace_manager = NamespaceManager::new()?;
        let cgroup_manager = CgroupManager::new()?;

        // 加载容器状态
        let containers = Arc::new(RwLock::new(Self::load_containers()?));

        Ok(Self {
            namespace_manager,
            cgroup_manager,
            storage,
            containers,
        })
    }

    /// 加载容器状态
    fn load_containers() -> Result<Vec<ContainerInfo>> {
        let container_file = Path::new("containers.json");
        if container_file.exists() {
            let mut file = File::open(container_file)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            serde_json::from_str(&content)
                .map_err(|e| DockerError::io_error("load_containers", e.to_string()))
        } else {
            Ok(vec![])
        }
    }

    /// 保存容器状态
    fn save_containers(containers: &Vec<ContainerInfo>) -> Result<()> {
        let container_file = Path::new("containers.json");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(container_file)?;
        let content = serde_json::to_string_pretty(containers)
            .map_err(|e| DockerError::io_error("save_containers", e.to_string()))?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// 启动运行时
    pub async fn start(&self) {
        // 克隆容器列表的引用
        let containers = self.containers.clone();

        // 启动清理任务
        tokio::spawn(async move {
            // 定期清理停止的容器
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5 * 60)).await;
                // 清理逻辑
                let mut containers = containers.write().await;
                // 移除已停止超过1小时的容器
                let now = std::time::SystemTime::now();
                containers.retain(|c| {
                    if c.status == ContainerStatus::Stopped {
                        if let Some(stopped_at) = c.stopped_at {
                            if let Ok(duration) = now.duration_since(stopped_at) {
                                return duration.as_secs() < 3600;
                            }
                        }
                    }
                    true
                });
            }
        });
    }

    /// 创建容器
    pub async fn create_container(&self, config: ContainerConfig) -> Result<ContainerInfo> {
        // 使用服务名称作为容器 ID
        let container_id = config.name.clone();

        // 创建容器存储目录
        self.storage.create_container_dir(&container_id)?;

        // 创建控制组
        self.cgroup_manager
            .create_cgroup(&container_id, &config.resources)?;

        // 创建容器信息
        let container_info = ContainerInfo {
            id: container_id.clone(),
            name: config.name.clone(),
            image: config.image.clone(),
            status: ContainerStatus::Creating,
            config: config.clone(),
            created_at: SystemTime::now(),
            started_at: None,
            stopped_at: None,
            pid: None,
            network_info: NetworkInfo {
                ip_address: None,
                ports: Default::default(),
                network_name: "default".to_string(),
            },
        };

        // 保存容器配置
        self.storage
            .write_container_config(&container_id, &config)?;

        // 保存环境变量
        self.storage
            .write_container_env(&container_id, &config.environment)?;

        // 添加到容器列表
        let mut containers = self.containers.write().await;
        // 检查容器是否已存在
        if let Some(index) = containers.iter().position(|c| c.id == container_id) {
            // 替换现有容器
            containers[index] = container_info.clone();
        } else {
            // 添加新容器
            containers.push(container_info.clone());
        }

        // 保存容器状态
        Self::save_containers(&containers)?;

        Ok(container_info)
    }

    /// 运行容器
    pub async fn run_container(
        &self,
        image: String,
        name: Option<String>,
        ports: Vec<String>,
        network_name: Option<String>,
        network_mode: Option<String>,
        aliases: Option<Vec<String>>,
        enable_ipv6: bool,
        detach: bool,
    ) -> Result<ContainerInfo> {
        // 创建容器配置
        let config = ContainerConfig {
            name: name.unwrap_or_else(|| format!("container-{}", Uuid::new_v4())),
            image,
            command: vec!["/bin/sh".to_string()],
            environment: Default::default(),
            ports: ports
                .into_iter()
                .filter_map(|p| {
                    if let Some((host, container)) = p.split_once(":") {
                        if let (Ok(host_port), Ok(container_port)) =
                            (host.parse::<u16>(), container.parse::<u16>())
                        {
                            Some((host_port, container_port))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect(),
            volumes: vec![],
            resources: ResourceLimits {
                cpu_limit: 1.0,
                memory_limit: 512,
                storage_limit: 10,
                network_limit: 10,
            },
            network: NetworkConfig {
                network_name: network_name.unwrap_or_else(|| "default".to_string()),
                static_ip: None,
                hostname: None,
                aliases,
                network_mode,
                enable_ipv6,
            },
            restart_policy: None,
            healthcheck: None,
            deploy: None,
        };

        // 创建容器
        let container = self.create_container(config).await?;

        // 启动容器
        self.start_container(&container.id).await?;

        Ok(container)
    }

    /// 启动容器
    pub async fn start_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;

        if let Some(container) = containers.iter_mut().find(|c| c.id == container_id) {
            // 检查状态
            if container.status != ContainerStatus::Creating
                && container.status != ContainerStatus::Stopped
            {
                return Err(DockerError::container_error(
                    "Container is not in a state to start".to_string(),
                ));
            }

            // 1. 创建命名空间
            let pid_ns = self
                .namespace_manager
                .create_namespace(NamespaceType::Pid)?;
            let net_ns = self
                .namespace_manager
                .create_namespace(NamespaceType::Network)?;
            let mnt_ns = self
                .namespace_manager
                .create_namespace(NamespaceType::Mount)?;
            let uts_ns = self
                .namespace_manager
                .create_namespace(NamespaceType::Uts)?;
            let ipc_ns = self
                .namespace_manager
                .create_namespace(NamespaceType::Ipc)?;
            let user_ns = self
                .namespace_manager
                .create_namespace(NamespaceType::User)?;

            // 2. 保存命名空间
            self.namespace_manager
                .save_namespace(container_id, NamespaceType::Pid, pid_ns)?;
            self.namespace_manager
                .save_namespace(container_id, NamespaceType::Network, net_ns)?;
            self.namespace_manager
                .save_namespace(container_id, NamespaceType::Mount, mnt_ns)?;
            self.namespace_manager
                .save_namespace(container_id, NamespaceType::Uts, uts_ns)?;
            self.namespace_manager
                .save_namespace(container_id, NamespaceType::Ipc, ipc_ns)?;
            self.namespace_manager
                .save_namespace(container_id, NamespaceType::User, user_ns)?;

            // 3. 准备容器文件系统
            self.storage
                .prepare_container_fs(container_id, &container.image)?;

            // 4. 执行容器命令
            // 这里使用 std::process 模拟容器进程
            let command = &container.config.command[0];
            let args = &container.config.command[1..];

            let child = std::process::Command::new(command)
                .args(args)
                .spawn()
                .map_err(|e| DockerError::io_error("spawn_process", e.to_string()))?;

            let pid = child.id() as u32;

            // 4. 将进程添加到控制组
            self.cgroup_manager.add_process(container_id, pid)?;

            // 5. 配置网络
            // 将容器连接到指定的网络
            let network_name = &container.config.network.network_name;
            // 这里可以使用 network_manager 来连接容器到网络
            // 由于我们没有直接访问 network_manager，这里简化实现
            // 实际项目中，应该通过 RustyDocker 结构体传递 network_manager

            // 6. 更新容器状态
            container.status = ContainerStatus::Running;
            container.started_at = Some(SystemTime::now());
            container.pid = Some(pid);

            // 更新网络信息
            container.network_info = NetworkInfo {
                ip_address: Some("172.17.0.2".to_string()), // 模拟 IP 地址
                ports: container.config.ports.clone(),
                network_name: network_name.clone(),
            };

            // 写入启动日志
            self.storage.write_container_log(
                container_id,
                &format!("Container started with PID: {}", pid),
            )?;
            self.storage.write_container_log(
                container_id,
                &format!("Network: {}, IP: {}", network_name, "172.17.0.2"),
            )?;

            // 保存容器状态
            Self::save_containers(&containers)?;

            Ok(())
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 停止容器
    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;

        if let Some(container) = containers.iter_mut().find(|c| c.id == container_id) {
            // 检查状态
            if container.status != ContainerStatus::Running {
                return Err(DockerError::container_error(
                    "Container is not running".to_string(),
                ));
            }

            // 1. 发送信号终止进程
            if let Some(pid) = container.pid {
                #[cfg(target_os = "linux")]
                {
                    // 尝试发送 SIGTERM 信号
                    if let Err(e) = signal::kill(unistd::Pid::from_raw(pid as i32), signal::SIGTERM)
                    {
                        // 如果发送失败，尝试 SIGKILL
                        signal::kill(unistd::Pid::from_raw(pid as i32), signal::SIGKILL).ok();
                    }
                }

                #[cfg(not(target_os = "linux"))]
                {
                    // 在非 Linux 平台上，使用标准库终止进程
                    if let Ok(process) = std::process::Command::new("taskkill")
                        .args(&["/F", "/PID", &pid.to_string()])
                        .output()
                    {
                        if !process.status.success() {
                            // 尝试其他方法
                        }
                    }
                }
            }

            // 2. 清理资源
            // 清理命名空间
            // 清理挂载点

            // 3. 更新容器状态
            container.status = ContainerStatus::Stopped;
            container.stopped_at = Some(SystemTime::now());
            container.pid = None;

            // 写入停止日志
            self.storage
                .write_container_log(container_id, "Container stopped")?;

            // 保存容器状态
            Self::save_containers(&containers)?;

            Ok(())
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 删除容器
    pub async fn delete_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;

        if let Some(index) = containers.iter().position(|c| c.id == container_id) {
            let container = &containers[index];

            // 检查状态
            if container.status == ContainerStatus::Running {
                return Err(DockerError::container_error(
                    "Container is running, please stop it first".to_string(),
                ));
            }

            // 1. 删除控制组
            self.cgroup_manager.delete_cgroup(container_id)?;

            // 2. 删除存储目录
            self.storage.delete_container_dir(container_id)?;

            // 3. 清理命名空间
            // 清理 PID 命名空间
            let pid_ns_path = self
                .namespace_manager
                .get_namespace_path(container_id, NamespaceType::Pid);
            if std::path::Path::new(&pid_ns_path).exists() {
                std::fs::remove_file(&pid_ns_path).ok();
            }
            // 清理网络命名空间
            let net_ns_path = self
                .namespace_manager
                .get_namespace_path(container_id, NamespaceType::Network);
            if std::path::Path::new(&net_ns_path).exists() {
                std::fs::remove_file(&net_ns_path).ok();
            }
            // 清理其他命名空间
            let mnt_ns_path = self
                .namespace_manager
                .get_namespace_path(container_id, NamespaceType::Mount);
            if std::path::Path::new(&mnt_ns_path).exists() {
                std::fs::remove_file(&mnt_ns_path).ok();
            }
            let uts_ns_path = self
                .namespace_manager
                .get_namespace_path(container_id, NamespaceType::Uts);
            if std::path::Path::new(&uts_ns_path).exists() {
                std::fs::remove_file(&uts_ns_path).ok();
            }
            let ipc_ns_path = self
                .namespace_manager
                .get_namespace_path(container_id, NamespaceType::Ipc);
            if std::path::Path::new(&ipc_ns_path).exists() {
                std::fs::remove_file(&ipc_ns_path).ok();
            }
            let user_ns_path = self
                .namespace_manager
                .get_namespace_path(container_id, NamespaceType::User);
            if std::path::Path::new(&user_ns_path).exists() {
                std::fs::remove_file(&user_ns_path).ok();
            }

            // 4. 从列表中移除
            containers.remove(index);

            // 5. 保存容器状态
            Self::save_containers(&containers)?;

            Ok(())
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 列出容器
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>> {
        let containers = self.containers.read().await;
        if all {
            Ok(containers.clone())
        } else {
            Ok(containers
                .iter()
                .filter(|c| c.status == ContainerStatus::Running)
                .cloned()
                .collect())
        }
    }

    /// 获取容器信息
    pub async fn get_container(&self, container_id: &str) -> Result<ContainerInfo> {
        let containers = self.containers.read().await;

        if let Some(container) = containers.iter().find(|c| c.id == container_id) {
            Ok(container.clone())
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 删除容器
    pub async fn remove_container(&self, container_id: &str) -> Result<()> {
        self.delete_container(container_id).await
    }

    /// 获取容器日志
    pub async fn get_container_logs(&self, container_id: &str) -> Result<Vec<String>> {
        // 使用存储服务读取容器日志
        self.storage.read_container_logs(container_id)
    }

    /// 在容器中执行命令
    pub async fn exec_command(&self, container_id: &str, command: &[String]) -> Result<String> {
        // 模拟执行命令
        Ok(format!(
            "Command executed in container {}: {}",
            container_id,
            command.join(" ")
        ))
    }

    /// 暂停容器
    pub async fn pause_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;

        if let Some(container) = containers.iter_mut().find(|c| c.id == container_id) {
            // 检查状态
            if container.status != ContainerStatus::Running {
                return Err(DockerError::container_error(
                    "Container is not running".to_string(),
                ));
            }

            // 模拟暂停
            container.status = ContainerStatus::Paused;

            // 保存容器状态
            Self::save_containers(&containers)?;

            Ok(())
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 恢复容器
    pub async fn unpause_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;

        if let Some(container) = containers.iter_mut().find(|c| c.id == container_id) {
            // 检查状态
            if container.status != ContainerStatus::Paused {
                return Err(DockerError::container_error(
                    "Container is not paused".to_string(),
                ));
            }

            // 模拟恢复
            container.status = ContainerStatus::Running;

            // 保存容器状态
            Self::save_containers(&containers)?;

            Ok(())
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 获取容器进程信息
    pub async fn get_container_processes(&self, container_id: &str) -> Result<Vec<String>> {
        // 模拟获取进程信息
        Ok(vec![
            format!("PID   USER     TIME   COMMAND"),
            format!("1     root     0:00   /bin/sh"),
            format!("10    root     0:01   /usr/sbin/nginx"),
            format!("20    root     0:00   /bin/bash"),
        ])
    }

    /// 获取容器端口映射
    pub async fn get_container_ports(
        &self,
        container_id: &str,
    ) -> Result<std::collections::HashMap<u16, u16>> {
        let containers = self.containers.read().await;

        if let Some(container) = containers.iter().find(|c| c.id == container_id) {
            Ok(container.config.ports.clone())
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 获取容器事件
    pub async fn get_container_events(&self) -> Result<Vec<String>> {
        // 模拟获取容器事件
        Ok(vec![
            format!(
                "{:?} container start test-container-1",
                std::time::SystemTime::now()
            ),
            format!(
                "{:?} container stop test-container-2",
                std::time::SystemTime::now()
            ),
            format!(
                "{:?} network create test-network",
                std::time::SystemTime::now()
            ),
            format!(
                "{:?} volume create test-volume",
                std::time::SystemTime::now()
            ),
            format!(
                "{:?} container start test-container-3",
                std::time::SystemTime::now()
            ),
        ])
    }

    /// 检查容器健康状态
    pub async fn check_container_health(&self, container_id: &str) -> Result<String> {
        let containers = self.containers.read().await;

        if let Some(container) = containers.iter().find(|c| c.id == container_id) {
            // 检查容器状态
            if container.status != ContainerStatus::Running {
                return Ok("unhealthy".to_string());
            }

            // 模拟健康检查
            // 实际实现中，这里应该：
            // 1. 执行容器中定义的健康检查命令
            // 2. 检查容器的网络连接
            // 3. 检查容器的资源使用情况

            Ok("healthy".to_string())
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 获取容器监控信息
    pub async fn get_container_stats(
        &self,
        container_id: &str,
    ) -> Result<docker_types::ContainerStats> {
        let containers = self.containers.read().await;

        if let Some(_container) = containers.iter().find(|c| c.id == container_id) {
            // 模拟容器监控信息
            // 实际实现中，这里应该从控制组获取真实的资源使用情况
            let stats = docker_types::ContainerStats {
                running: 1,
                stopped: 0,
                total: 1,
            };

            Ok(stats)
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 获取容器资源使用情况
    pub async fn get_container_resource_usage(
        &self,
        container_id: &str,
    ) -> Result<docker_types::SystemResourceUsage> {
        let containers = self.containers.read().await;

        if let Some(_container) = containers.iter().find(|c| c.id == container_id) {
            // 模拟资源使用情况
            // 实际实现中，这里应该从控制组获取真实的资源使用情况
            let usage = docker_types::SystemResourceUsage {
                cpu_usage: 25.5,
                memory_used: 128,
                memory_total: 512,
                storage_used: 10,
                storage_total: 100,
                network_sent: 50,
                network_received: 100,
            };

            Ok(usage)
        } else {
            Err(DockerError::not_found("container", container_id))
        }
    }

    /// 获取容器环境变量
    pub async fn get_container_env(
        &self,
        container_id: &str,
    ) -> Result<std::collections::HashMap<String, String>> {
        // 使用存储服务读取容器环境变量
        self.storage.read_container_env(container_id)
    }

    /// 更新容器环境变量
    pub async fn update_container_env(
        &self,
        container_id: &str,
        env: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        // 写入环境变量
        self.storage.write_container_env(container_id, env)?;

        // 更新容器列表中的环境变量
        let mut containers = self.containers.write().await;
        if let Some(container) = containers.iter_mut().find(|c| c.id == container_id) {
            container.config.environment = env.clone();
            // 保存容器状态
            Self::save_containers(&containers)?;
        }

        Ok(())
    }

    /// 获取容器配置
    pub async fn get_container_config(
        &self,
        container_id: &str,
    ) -> Result<docker_types::ContainerConfig> {
        // 使用存储服务读取容器配置
        self.storage.read_container_config(container_id)
    }

    /// 更新容器配置
    pub async fn update_container_config(
        &self,
        container_id: &str,
        config: &docker_types::ContainerConfig,
    ) -> Result<()> {
        // 写入配置
        self.storage.write_container_config(container_id, config)?;

        // 更新容器列表中的配置
        let mut containers = self.containers.write().await;
        if let Some(container) = containers.iter_mut().find(|c| c.id == container_id) {
            container.config = config.clone();
            // 保存容器状态
            Self::save_containers(&containers)?;
        }

        Ok(())
    }
}
