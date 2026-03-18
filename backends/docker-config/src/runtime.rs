#![warn(missing_docs)]

//! 容器运行时

use std::{sync::Arc, time::SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

use docker_types::{ContainerConfig, ContainerInfo, ContainerStatus, DockerConfig, DockerError};

/// 结果类型
pub type Result<T> = std::result::Result<T, DockerError>;

/// 容器运行时
pub struct ContainerRuntime {
    /// 全局配置
    config: Arc<DockerConfig>,
    /// 容器列表
    containers: Arc<RwLock<Vec<ContainerInfo>>>,
}

impl ContainerRuntime {
    /// 创建新的容器运行时
    pub fn new() -> Result<Self> {
        // 使用默认配置
        let default_config = DockerConfig {
            data_dir: "./data".to_string(),
            image_dir: "./data/images".to_string(),
            container_dir: "./data/containers".to_string(),
            network_dir: "./data/networks".to_string(),
            default_network: "default".to_string(),
            default_resources: docker_types::ResourceLimits {
                cpu_limit: 1.0,
                memory_limit: 512,
                storage_limit: 10,
                network_limit: 10,
            },
            log_config: docker_types::LogConfig {
                log_level: "info".to_string(),
                log_file: "./data/logs/docker.log".to_string(),
                max_log_size: 100,
            },
        };

        Ok(Self { config: Arc::new(default_config), containers: Arc::new(RwLock::new(vec![])) })
    }

    /// 使用配置创建新的容器运行时
    pub fn new_with_config(config: Arc<DockerConfig>) -> Result<Self> {
        Ok(Self { config, containers: Arc::new(RwLock::new(vec![])) })
    }

    /// 获取配置
    pub fn get_config(&self) -> Arc<DockerConfig> {
        self.config.clone()
    }

    /// 启动运行时
    pub async fn start(&self) {
        // 启动清理任务
        tokio::spawn(async move {
            // 定期清理停止的容器
            loop {
                tokio::time::sleep(tokio::time::Duration::from_mins(5)).await;
                // 清理逻辑
            }
        });
    }

    /// 创建容器
    pub async fn create_container(&self, config: ContainerConfig) -> Result<ContainerInfo> {
        // 生成容器 ID
        let container_id = Uuid::new_v4().to_string();

        // 创建容器信息
        let container_info = ContainerInfo {
            id: container_id.clone(),
            name: config.name.clone(),
            image: config.image.clone(),
            status: ContainerStatus::Creating,
            config,
            created_at: SystemTime::now(),
            started_at: None,
            stopped_at: None,
            pid: None,
            network_info: docker_types::NetworkInfo {
                ip_address: None,
                ports: Default::default(),
                network_name: self.config.default_network.clone(),
            },
        };

        // 添加到容器列表
        let mut containers = self.containers.write().await;
        containers.push(container_info.clone());

        Ok(container_info)
    }

    /// 启动容器
    pub async fn start_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;

        if let Some(container) = containers.iter_mut().find(|c| c.id == container_id) {
            // 检查状态
            if container.status != ContainerStatus::Creating && container.status != ContainerStatus::Stopped {
                return Err(DockerError::container_error("Container is not in a state to start".to_string()));
            }

            // 启动容器（这里是简化实现）
            // 实际实现需要：
            // 1. 进入命名空间
            // 2. 挂载文件系统
            // 3. 执行容器命令
            // 4. 将进程添加到控制组

            // 模拟启动
            container.status = ContainerStatus::Running;
            container.started_at = Some(SystemTime::now());
            container.pid = Some(12345); // 模拟 PID

            Ok(())
        }
        else {
            Err(DockerError::not_found("container", format!("Container {} not found", container_id)))
        }
    }

    /// 停止容器
    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;

        if let Some(container) = containers.iter_mut().find(|c| c.id == container_id) {
            // 检查状态
            if container.status != ContainerStatus::Running {
                return Err(DockerError::container_error("Container is not in a running state".to_string()));
            }

            // 模拟停止
            container.status = ContainerStatus::Stopped;
            container.stopped_at = Some(SystemTime::now());
            container.pid = None;

            Ok(())
        }
        else {
            Err(DockerError::not_found("container", format!("Container {} not found", container_id)))
        }
    }

    /// 运行容器
    pub async fn run_container(&self, image: String, name: Option<String>, _ports: Vec<String>) -> Result<ContainerInfo> {
        // 创建容器配置
        let config = ContainerConfig {
            name: name.unwrap_or_else(|| format!("container-{}", Uuid::new_v4().to_string().split('-').next().unwrap())),
            image,
            command: vec!["/bin/sh".to_string()],
            environment: Default::default(),
            ports: Default::default(),
            volumes: Default::default(),
            resources: self.config.default_resources.clone(),
            network: docker_types::NetworkConfig {
                network_name: self.config.default_network.clone(),
                static_ip: None,
                hostname: None,
                aliases: None,
                network_mode: None,
                enable_ipv6: false,
            },
            restart_policy: None,
            healthcheck: None,
            deploy: None,
        };

        // 创建并启动容器
        let container_info = self.create_container(config).await?;
        self.start_container(&container_info.id).await?;

        Ok(container_info)
    }

    /// 列出容器
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>> {
        let containers = self.containers.read().await;

        if all {
            Ok(containers.clone())
        }
        else {
            Ok(containers.iter().filter(|c| c.status == ContainerStatus::Running).cloned().collect())
        }
    }

    /// 删除容器
    pub async fn remove_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;

        if let Some(index) = containers.iter().position(|c| c.id == container_id) {
            // 检查状态
            let container = &containers[index];
            if container.status == ContainerStatus::Running {
                return Err(DockerError::container_error("Cannot remove a running container".to_string()));
            }

            // 移除容器
            containers.remove(index);

            Ok(())
        }
        else {
            Err(DockerError::not_found("container", format!("Container {} not found", container_id)))
        }
    }
}
