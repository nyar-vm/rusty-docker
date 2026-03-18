#![warn(missing_docs)]

//! rusty-docker - Rust 实现的轻量级容器运行时
//!
//! 实现基于 Rust 的容器运行时，提供：
//! - 轻量级容器隔离
//! - 毫秒级启动速度
//! - 资源限制和监控
//! - Docker 镜像兼容
//! - 与 MOS 平台集成

pub mod cgroup;
pub mod error;
pub mod models;
pub mod namespace;
pub mod runtime;
pub mod storage;
pub mod user;

use std::sync::{Arc, Mutex};

use docker_network::{NetworkConfig, NetworkManager, new_network_manager};
use docker_types::{ContainerInfo, ImageInfo, NetworkConfigInfo, Result as DockerResult};
use futures_util::future::TryFutureExt;
use runtime::ContainerRuntime;
use storage::StorageService;
use user::{Permission, Role, UserManager};

/// 堆栈信息
pub struct StackInfo {
    /// 堆栈名称
    pub name: String,
    /// 堆栈状态
    pub status: String,
    /// 服务数量
    pub services: u32,
    /// 容器数量
    pub containers: u32,
    /// 创建时间
    pub created_at: std::time::SystemTime,
}

/// Rusty Docker 服务
pub struct RustyDocker {
    /// 容器运行时
    runtime: Arc<ContainerRuntime>,
    /// 存储服务
    storage: Arc<StorageService>,
    /// 网络管理器
    network_manager: Arc<Mutex<Box<dyn NetworkManager>>>,
    /// 用户管理器
    user_manager: Arc<UserManager>,
}

/// Docker 服务别名
pub type Docker = RustyDocker;

impl RustyDocker {
    /// 创建新的 Rusty Docker 服务
    pub fn new() -> DockerResult<Self> {
        let storage = Arc::new(StorageService::new()?);
        let runtime = Arc::new(ContainerRuntime::new(storage.clone())?);
        let network_manager = Arc::new(Mutex::new(new_network_manager()));
        let user_manager = Arc::new(UserManager::new());

        Ok(Self {
            runtime,
            storage,
            network_manager,
            user_manager,
        })
    }

    /// 启动服务
    pub async fn start(&self) {
        // 直接启动运行时
        self.runtime.start().await;
    }

    /// 获取容器运行时
    pub fn get_runtime(&self) -> Arc<ContainerRuntime> {
        self.runtime.clone()
    }

    /// 获取存储服务
    pub fn get_storage(&self) -> Arc<StorageService> {
        self.storage.clone()
    }

    /// 运行容器
    pub async fn run(
        &mut self,
        image: String,
        name: Option<String>,
        ports: Vec<String>,
        network_name: Option<String>,
        network_mode: Option<String>,
        aliases: Option<Vec<String>>,
        enable_ipv6: bool,
        detach: bool,
    ) -> DockerResult<ContainerInfo> {
        self.runtime
            .run_container(
                image,
                name,
                ports,
                network_name,
                network_mode,
                aliases,
                enable_ipv6,
                detach,
            )
            .await
    }

    /// 列出容器
    pub async fn list_containers(&self, all: bool) -> DockerResult<Vec<ContainerInfo>> {
        self.runtime.list_containers(all).await
    }

    /// 停止容器
    pub async fn stop_container(&mut self, container_id: &str) -> DockerResult<()> {
        self.runtime.stop_container(container_id).await
    }

    /// 删除容器
    pub async fn remove_container(&mut self, container_id: &str) -> DockerResult<()> {
        self.runtime.remove_container(container_id).await
    }

    /// 构建镜像
    pub async fn build_image(
        &mut self,
        path: &str,
        tag: &str,
        pull: bool,
        no_cache: bool,
        force_rm: bool,
    ) -> DockerResult<ImageInfo> {
        self.storage
            .build_image(path, tag, pull, no_cache, force_rm)
            .await
    }

    /// 列出镜像
    pub async fn list_images(&self) -> DockerResult<Vec<ImageInfo>> {
        self.storage.list_images().await
    }

    /// 拉取镜像
    pub async fn pull_image(&self, image: &str, tag: &str) -> DockerResult<ImageInfo> {
        self.storage.pull_image(image, tag).await
    }

    /// 推送镜像
    pub async fn push_image(&self, image: &str, tag: &str) -> DockerResult<ImageInfo> {
        self.storage.push_image(image, tag).await
    }

    /// 删除镜像
    pub async fn delete_image(&self, image_id: &str) -> DockerResult<()> {
        self.storage.delete_image(image_id).await
    }

    /// 列出网络
    pub async fn list_networks(&self) -> DockerResult<Vec<docker_types::NetworkConfigInfo>> {
        // 使用网络管理器列出网络
        let mut network_manager = self
            .network_manager
            .lock()
            .map_err(|e| docker_types::DockerError::internal(e.to_string()))?;

        let networks = network_manager
            .list_networks()
            .await
            .map_err(|e| docker_types::DockerError::network_error(e.to_string()))?;

        // 转换为 docker_types::NetworkConfigInfo
        let network_configs = networks
            .into_iter()
            .map(|network| {
                docker_types::NetworkConfigInfo {
                    name: network.name,
                    network_type: network.driver,
                    subnet: "".to_string(),  // 需要从 network.options 中解析
                    gateway: "".to_string(), // 需要从 network.options 中解析
                    containers: vec![],      // 需要从 network.containers 中解析
                }
            })
            .collect();

        Ok(network_configs)
    }

    /// 创建网络
    pub async fn create_network(
        &self,
        name: String,
        driver: String,
        enable_ipv6: bool,
        options: Option<std::collections::HashMap<String, String>>,
    ) -> DockerResult<docker_types::NetworkConfigInfo> {
        // 使用网络管理器创建网络
        let mut network_manager = self
            .network_manager
            .lock()
            .map_err(|e| docker_types::DockerError::internal(e.to_string()))?;

        let network_config = NetworkConfig {
            name: name.clone(),
            driver: driver.clone(),
            ipam: None,
            options,
            aliases: None,
            network_mode: None,
            enable_ipv6,
        };

        let network = network_manager
            .create_network(&network_config)
            .await
            .map_err(|e| docker_types::DockerError::network_error(e.to_string()))?;

        // 转换为 docker_types::NetworkConfigInfo
        Ok(docker_types::NetworkConfigInfo {
            name: network.name,
            network_type: network.driver,
            subnet: "".to_string(),  // 需要从 network.options 中解析
            gateway: "".to_string(), // 需要从 network.options 中解析
            containers: vec![],      // 需要从 network.containers 中解析
        })
    }

    /// 删除网络
    pub async fn delete_network(&self, network_id: &str) -> DockerResult<()> {
        // 使用网络管理器删除网络
        let mut network_manager = self
            .network_manager
            .lock()
            .map_err(|e| docker_types::DockerError::internal(e.to_string()))?;

        network_manager
            .remove_network(network_id)
            .await
            .map_err(|e| docker_types::DockerError::network_error(e.to_string()))
    }

    /// 查看网络详情
    pub async fn inspect_network(
        &self,
        network_id: &str,
    ) -> DockerResult<docker_types::NetworkConfigInfo> {
        // 使用网络管理器查看网络详情
        let mut network_manager = self
            .network_manager
            .lock()
            .map_err(|e| docker_types::DockerError::internal(e.to_string()))?;

        let network = network_manager
            .inspect_network(network_id)
            .await
            .map_err(|e| docker_types::DockerError::network_error(e.to_string()))?;

        // 转换为 docker_types::NetworkConfigInfo
        Ok(docker_types::NetworkConfigInfo {
            name: network.name,
            network_type: network.driver,
            subnet: "".to_string(),  // 需要从 network.options 中解析
            gateway: "".to_string(), // 需要从 network.options 中解析
            containers: vec![],      // 需要从 network.containers 中解析
        })
    }

    /// 列出卷
    pub async fn list_volumes(&self) -> DockerResult<Vec<docker_types::VolumeInfo>> {
        self.storage.list_volumes().await
    }

    /// 创建卷
    pub async fn create_volume(
        &mut self,
        name: String,
        driver: String,
        labels: Option<std::collections::HashMap<String, String>>,
    ) -> DockerResult<docker_types::VolumeInfo> {
        self.storage.create_volume(&name, &driver, labels).await
    }

    /// 删除卷
    pub async fn delete_volume(&mut self, volume_id: &str) -> DockerResult<()> {
        self.storage.delete_volume(volume_id).await
    }

    /// 获取卷详情
    pub async fn get_volume(&self, volume_id: &str) -> DockerResult<docker_types::VolumeInfo> {
        self.storage.get_volume(volume_id).await
    }

    /// 克隆 Docker 实例
    pub fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            storage: self.storage.clone(),
            network_manager: self.network_manager.clone(),
            user_manager: self.user_manager.clone(),
        }
    }

    /// 获取系统状态
    pub async fn get_system_status(&self) -> DockerResult<docker_types::SystemStatus> {
        // 获取容器统计信息
        let containers = self.runtime.list_containers(true).await?;
        let running_containers = containers
            .iter()
            .filter(|c| c.status == docker_types::ContainerStatus::Running)
            .count() as u32;
        let stopped_containers = containers.len() as u32 - running_containers;
        let total_containers = containers.len() as u32;

        let container_stats = docker_types::ContainerStats {
            running: running_containers,
            stopped: stopped_containers,
            total: total_containers,
        };

        // 模拟系统资源使用情况
        let resource_usage = docker_types::SystemResourceUsage {
            cpu_usage: 45.5,
            memory_used: 2048,
            memory_total: 8192,
            storage_used: 50,
            storage_total: 256,
            network_sent: 100,
            network_received: 200,
        };

        // 模拟系统信息
        let system_info = docker_types::SystemInfo {
            os_type: "Windows".to_string(),
            os_version: "10.0.19045".to_string(),
            kernel_version: "10.0.19045.3930".to_string(),
            architecture: "x86_64".to_string(),
            hostname: "DESKTOP-123456".to_string(),
            cpu_cores: 8,
            total_memory: 8192,
        };

        // 模拟 Docker 守护进程状态
        let daemon_status = docker_types::DockerDaemonStatus::Running;

        Ok(docker_types::SystemStatus {
            daemon_status,
            resource_usage,
            system_info,
            container_stats,
        })
    }

    /// 启动容器
    pub async fn start_container(&self, container_id: &str) -> DockerResult<()> {
        self.runtime.start_container(container_id).await
    }

    /// 获取容器日志
    pub async fn get_container_logs(&self, container_id: &str) -> DockerResult<Vec<String>> {
        self.runtime.get_container_logs(container_id).await
    }

    /// 在容器中执行命令
    pub async fn exec_command(
        &self,
        container_id: &str,
        command: &[String],
    ) -> DockerResult<String> {
        self.runtime.exec_command(container_id, command).await
    }

    /// 暂停容器
    pub async fn pause_container(&mut self, container_id: &str) -> DockerResult<()> {
        self.runtime.pause_container(container_id).await
    }

    /// 恢复容器
    pub async fn unpause_container(&mut self, container_id: &str) -> DockerResult<()> {
        self.runtime.unpause_container(container_id).await
    }

    /// 获取容器进程信息
    pub async fn get_container_processes(&self, container_id: &str) -> DockerResult<Vec<String>> {
        self.runtime.get_container_processes(container_id).await
    }

    /// 获取容器端口映射
    pub async fn get_container_ports(
        &self,
        container_id: &str,
    ) -> DockerResult<std::collections::HashMap<u16, u16>> {
        self.runtime.get_container_ports(container_id).await
    }

    /// 获取容器事件
    pub async fn get_container_events(&self) -> DockerResult<Vec<String>> {
        self.runtime.get_container_events().await
    }

    // Swarm 相关方法

    /// 初始化 Swarm 集群
    pub async fn swarm_init(
        &mut self,
        advertise_addr: Option<String>,
        auto_lock: bool,
        default_addr_pool: Option<String>,
        force_new_cluster: bool,
        subnet_size: u8,
    ) -> DockerResult<()> {
        // 模拟 Swarm 初始化
        Ok(())
    }

    /// 加入 Swarm 集群
    pub async fn swarm_join(
        &mut self,
        token: String,
        advertise_addr: Option<String>,
        listen_addr: Option<String>,
        manager_addr: Option<String>,
    ) -> DockerResult<()> {
        // 模拟 Swarm 加入
        Ok(())
    }

    /// 离开 Swarm 集群
    pub async fn swarm_leave(&mut self, force: bool) -> DockerResult<()> {
        // 模拟 Swarm 离开
        Ok(())
    }

    /// 获取 Swarm 集群信息
    pub async fn swarm_info(&self) -> DockerResult<docker_types::SwarmInfo> {
        // 模拟 Swarm 集群信息
        Ok(docker_types::SwarmInfo {
            id: "swarm-1234567890".to_string(),
            name: Some("my-swarm".to_string()),
            managers: 1,
            workers: 2,
            services: 3,
            tasks: 9,
            version: "1.29.0".to_string(),
            created_at: std::time::SystemTime::now(),
        })
    }

    /// 更新 Swarm 集群配置
    pub async fn swarm_update(
        &mut self,
        auto_lock: Option<bool>,
        default_addr_pool: Option<String>,
        subnet_size: Option<u8>,
    ) -> DockerResult<()> {
        // 模拟 Swarm 更新
        Ok(())
    }

    /// 创建 Swarm 服务
    pub async fn create_service(
        &mut self,
        name: String,
        image: String,
        publish: Vec<String>,
        replicas: u32,
        env: Vec<String>,
        mount: Vec<String>,
    ) -> DockerResult<docker_types::ServiceInfo> {
        // 解析端口映射
        let mut ports = std::collections::HashMap::new();
        for port in publish {
            if let Some((host, container)) = port.split_once(":") {
                if let (Ok(host_port), Ok(container_port)) =
                    (host.parse::<u16>(), container.parse::<u16>())
                {
                    ports.insert(host_port, container_port);
                }
            }
        }

        // 解析环境变量
        let mut environment = std::collections::HashMap::new();
        for env_var in env {
            if let Some((key, value)) = env_var.split_once("=") {
                environment.insert(key.to_string(), value.to_string());
            }
        }

        // 模拟创建服务
        Ok(docker_types::ServiceInfo {
            id: format!("service-{}", uuid::Uuid::new_v4()),
            name,
            status: docker_types::ServiceStatus::Running,
            image,
            replicas,
            ports,
            environment,
            volumes: vec![],
            created_at: std::time::SystemTime::now(),
            updated_at: std::time::SystemTime::now(),
        })
    }

    /// 列出 Swarm 服务
    pub async fn list_services(&self) -> DockerResult<Vec<docker_types::ServiceInfo>> {
        // 模拟服务列表
        Ok(vec![
            docker_types::ServiceInfo {
                id: "service-1".to_string(),
                name: "web".to_string(),
                status: docker_types::ServiceStatus::Running,
                image: "nginx:latest".to_string(),
                replicas: 3,
                ports: std::collections::HashMap::from([(80, 80)]),
                environment: std::collections::HashMap::new(),
                volumes: vec![],
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
            docker_types::ServiceInfo {
                id: "service-2".to_string(),
                name: "db".to_string(),
                status: docker_types::ServiceStatus::Running,
                image: "postgres:latest".to_string(),
                replicas: 1,
                ports: std::collections::HashMap::from([(5432, 5432)]),
                environment: std::collections::HashMap::from([(
                    "POSTGRES_PASSWORD".to_string(),
                    "secret".to_string(),
                )]),
                volumes: vec![],
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
        ])
    }

    /// 查看 Swarm 服务详情
    pub async fn inspect_service(&self, service: &str) -> DockerResult<docker_types::ServiceInfo> {
        // 模拟服务详情
        Ok(docker_types::ServiceInfo {
            id: service.to_string(),
            name: "web".to_string(),
            status: docker_types::ServiceStatus::Running,
            image: "nginx:latest".to_string(),
            replicas: 3,
            ports: std::collections::HashMap::from([(80, 80)]),
            environment: std::collections::HashMap::new(),
            volumes: vec![],
            created_at: std::time::SystemTime::now(),
            updated_at: std::time::SystemTime::now(),
        })
    }

    /// 更新 Swarm 服务
    pub async fn update_service(
        &mut self,
        service: &str,
        image: Option<String>,
        replicas: Option<u32>,
    ) -> DockerResult<docker_types::ServiceInfo> {
        // 模拟服务更新
        Ok(docker_types::ServiceInfo {
            id: service.to_string(),
            name: "web".to_string(),
            status: docker_types::ServiceStatus::Updating,
            image: image.unwrap_or("nginx:latest".to_string()),
            replicas: replicas.unwrap_or(3),
            ports: std::collections::HashMap::from([(80, 80)]),
            environment: std::collections::HashMap::new(),
            volumes: vec![],
            created_at: std::time::SystemTime::now(),
            updated_at: std::time::SystemTime::now(),
        })
    }

    /// 删除 Swarm 服务
    pub async fn remove_service(&mut self, service: &str) -> DockerResult<()> {
        // 模拟服务删除
        Ok(())
    }

    /// 扩缩容 Swarm 服务
    pub async fn scale_service(
        &mut self,
        service: &str,
        replicas: u32,
    ) -> DockerResult<docker_types::ServiceInfo> {
        // 模拟服务扩缩容
        Ok(docker_types::ServiceInfo {
            id: service.to_string(),
            name: "web".to_string(),
            status: docker_types::ServiceStatus::Updating,
            image: "nginx:latest".to_string(),
            replicas,
            ports: std::collections::HashMap::from([(80, 80)]),
            environment: std::collections::HashMap::new(),
            volumes: vec![],
            created_at: std::time::SystemTime::now(),
            updated_at: std::time::SystemTime::now(),
        })
    }

    /// 列出 Swarm 节点
    pub async fn list_nodes(&self) -> DockerResult<Vec<docker_types::NodeInfo>> {
        // 模拟节点列表
        Ok(vec![
            docker_types::NodeInfo {
                id: "node-1".to_string(),
                name: "manager1".to_string(),
                role: docker_types::NodeRole::Manager,
                availability: docker_types::NodeAvailability::Active,
                status: docker_types::NodeStatus::Ready,
                address: "192.168.1.100:2377".to_string(),
                version: "1.29.0".to_string(),
                containers_running: 5,
                labels: std::collections::HashMap::new(),
            },
            docker_types::NodeInfo {
                id: "node-2".to_string(),
                name: "worker1".to_string(),
                role: docker_types::NodeRole::Worker,
                availability: docker_types::NodeAvailability::Active,
                status: docker_types::NodeStatus::Ready,
                address: "192.168.1.101:2377".to_string(),
                version: "1.29.0".to_string(),
                containers_running: 3,
                labels: std::collections::HashMap::new(),
            },
        ])
    }

    /// 查看 Swarm 节点详情
    pub async fn inspect_node(&self, node: &str) -> DockerResult<docker_types::NodeInfo> {
        // 模拟节点详情
        Ok(docker_types::NodeInfo {
            id: node.to_string(),
            name: "manager1".to_string(),
            role: docker_types::NodeRole::Manager,
            availability: docker_types::NodeAvailability::Active,
            status: docker_types::NodeStatus::Ready,
            address: "192.168.1.100:2377".to_string(),
            version: "1.29.0".to_string(),
            containers_running: 5,
            labels: std::collections::HashMap::new(),
        })
    }

    /// 更新 Swarm 节点
    pub async fn update_node(
        &mut self,
        node: &str,
        role: Option<String>,
        availability: Option<String>,
    ) -> DockerResult<docker_types::NodeInfo> {
        // 模拟节点更新
        Ok(docker_types::NodeInfo {
            id: node.to_string(),
            name: "worker1".to_string(),
            role: if role.as_deref() == Some("manager") {
                docker_types::NodeRole::Manager
            } else {
                docker_types::NodeRole::Worker
            },
            availability: match availability.as_deref() {
                Some("active") => docker_types::NodeAvailability::Active,
                Some("pause") => docker_types::NodeAvailability::Pause,
                Some("drain") => docker_types::NodeAvailability::Drain,
                _ => docker_types::NodeAvailability::Active,
            },
            status: docker_types::NodeStatus::Ready,
            address: "192.168.1.101:2377".to_string(),
            version: "1.29.0".to_string(),
            containers_running: 3,
            labels: std::collections::HashMap::new(),
        })
    }

    /// 提升 Swarm 节点为 manager
    pub async fn promote_node(&mut self, node: &str) -> DockerResult<()> {
        // 模拟节点提升
        Ok(())
    }

    /// 降级 Swarm 节点为 worker
    pub async fn demote_node(&mut self, node: &str) -> DockerResult<()> {
        // 模拟节点降级
        Ok(())
    }

    /// 删除 Swarm 节点
    pub async fn remove_node(&mut self, node: &str) -> DockerResult<()> {
        // 模拟节点删除
        Ok(())
    }

    // 堆栈管理相关方法

    /// 创建堆栈
    pub async fn stack_deploy(
        &mut self,
        name: String,
        compose_file: String,
        prune: bool,
    ) -> DockerResult<StackInfo> {
        // 模拟堆栈部署
        Ok(StackInfo {
            name,
            status: "running".to_string(),
            services: 3,
            containers: 3,
            created_at: std::time::SystemTime::now(),
        })
    }

    /// 列出堆栈
    pub async fn stack_list(&self) -> DockerResult<Vec<StackInfo>> {
        // 模拟堆栈列表
        Ok(vec![
            StackInfo {
                name: "my-stack".to_string(),
                status: "running".to_string(),
                services: 3,
                containers: 3,
                created_at: std::time::SystemTime::now(),
            },
            StackInfo {
                name: "test-stack".to_string(),
                status: "exited".to_string(),
                services: 2,
                containers: 0,
                created_at: std::time::SystemTime::now(),
            },
        ])
    }

    /// 查看堆栈详情
    pub async fn stack_inspect(&self, stack: &str) -> DockerResult<StackInfo> {
        // 模拟堆栈详情
        Ok(StackInfo {
            name: stack.to_string(),
            status: "running".to_string(),
            services: 3,
            containers: 3,
            created_at: std::time::SystemTime::now(),
        })
    }

    /// 删除堆栈
    pub async fn stack_rm(&mut self, stack: &str) -> DockerResult<()> {
        // 模拟堆栈删除
        Ok(())
    }

    /// 列出堆栈中的服务
    pub async fn stack_services(
        &self,
        stack: &str,
    ) -> DockerResult<Vec<docker_types::ServiceInfo>> {
        // 模拟堆栈服务列表
        Ok(vec![
            docker_types::ServiceInfo {
                id: "service-1".to_string(),
                name: "web".to_string(),
                status: docker_types::ServiceStatus::Running,
                image: "nginx:latest".to_string(),
                replicas: 1,
                ports: std::collections::HashMap::from([(80, 80)]),
                environment: std::collections::HashMap::new(),
                volumes: vec![],
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
            docker_types::ServiceInfo {
                id: "service-2".to_string(),
                name: "db".to_string(),
                status: docker_types::ServiceStatus::Running,
                image: "postgres:latest".to_string(),
                replicas: 1,
                ports: std::collections::HashMap::from([(5432, 5432)]),
                environment: std::collections::HashMap::from([(
                    "POSTGRES_PASSWORD".to_string(),
                    "secret".to_string(),
                )]),
                volumes: vec![],
                created_at: std::time::SystemTime::now(),
                updated_at: std::time::SystemTime::now(),
            },
        ])
    }

    // 用户管理相关方法

    /// 创建用户
    pub fn create_user(
        &self,
        username: String,
        password: String,
        role: Role,
    ) -> DockerResult<user::User> {
        self.user_manager.create_user(username, password, role)
    }

    /// 获取用户
    pub fn get_user(&self, user_id: &str) -> DockerResult<user::User> {
        self.user_manager.get_user(user_id)
    }

    /// 获取用户通过用户名
    pub fn get_user_by_username(&self, username: &str) -> DockerResult<user::User> {
        self.user_manager.get_user_by_username(username)
    }

    /// 列出所有用户
    pub fn list_users(&self) -> DockerResult<Vec<user::User>> {
        self.user_manager.list_users()
    }

    /// 更新用户
    pub fn update_user(
        &self,
        user_id: &str,
        username: Option<String>,
        password: Option<String>,
        role: Option<Role>,
    ) -> DockerResult<user::User> {
        self.user_manager
            .update_user(user_id, username, password, role)
    }

    /// 删除用户
    pub fn delete_user(&self, user_id: &str) -> DockerResult<()> {
        self.user_manager.delete_user(user_id)
    }

    /// 验证用户凭据
    pub fn authenticate(&self, username: &str, password: &str) -> DockerResult<user::User> {
        self.user_manager.authenticate(username, password)
    }

    /// 检查用户是否有指定权限
    pub fn check_permission(&self, user_id: &str, permission: &Permission) -> DockerResult<bool> {
        self.user_manager.check_permission(user_id, permission)
    }

    /// 获取用户角色
    pub fn get_user_role(&self, user_id: &str) -> DockerResult<Role> {
        self.user_manager.get_user_role(user_id)
    }

    /// 更新用户角色
    pub fn update_user_role(&self, user_id: &str, role: Role) -> DockerResult<user::User> {
        self.user_manager.update_user_role(user_id, role)
    }

    /// 获取用户管理器
    pub fn get_user_manager(&self) -> Arc<UserManager> {
        self.user_manager.clone()
    }
}
