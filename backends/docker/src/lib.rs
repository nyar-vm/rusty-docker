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
pub mod oci;
pub mod runtime;
pub mod storage;
pub mod swarm;
pub mod user;

use std::sync::{Arc, Mutex};

use docker_network::{NetworkConfig, NetworkManager, new_network_manager};
use docker_types::{ContainerInfo, ImageInfo, NetworkConfigInfo, Result as DockerResult};
use futures_util::future::TryFutureExt;
use runtime::ContainerRuntime;
use storage::StorageService;
use swarm::{init_swarm_manager, get_swarm_manager, SwarmManager};
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
    /// Swarm 管理器
    swarm_manager: Arc<SwarmManager>,
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
        
        // 初始化 Swarm 管理器
        init_swarm_manager();
        let swarm_manager = get_swarm_manager();

        Ok(Self { runtime, storage, network_manager, user_manager, swarm_manager })
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
        self.runtime.run_container(image, name, ports, network_name, network_mode, aliases, enable_ipv6, detach).await
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
        self.storage.build_image(path, tag, pull, no_cache, force_rm).await
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
        let mut network_manager =
            self.network_manager.lock().map_err(|e| docker_types::DockerError::internal(e.to_string()))?;

        let networks =
            network_manager.list_networks().await.map_err(|e| docker_types::DockerError::network_error(e.to_string()))?;

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
        let mut network_manager =
            self.network_manager.lock().map_err(|e| docker_types::DockerError::internal(e.to_string()))?;

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
        let mut network_manager =
            self.network_manager.lock().map_err(|e| docker_types::DockerError::internal(e.to_string()))?;

        network_manager.remove_network(network_id).await.map_err(|e| docker_types::DockerError::network_error(e.to_string()))
    }

    /// 查看网络详情
    pub async fn inspect_network(&self, network_id: &str) -> DockerResult<docker_types::NetworkConfigInfo> {
        // 使用网络管理器查看网络详情
        let mut network_manager =
            self.network_manager.lock().map_err(|e| docker_types::DockerError::internal(e.to_string()))?;

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
            swarm_manager: self.swarm_manager.clone(),
        }
    }

    /// 获取系统状态
    pub async fn get_system_status(&self) -> DockerResult<docker_types::SystemStatus> {
        // 获取容器统计信息
        let containers = self.runtime.list_containers(true).await?;
        let running_containers =
            containers.iter().filter(|c| c.status == docker_types::ContainerStatus::Running).count() as u32;
        let stopped_containers = containers.len() as u32 - running_containers;
        let total_containers = containers.len() as u32;

        let container_stats =
            docker_types::ContainerStats { running: running_containers, stopped: stopped_containers, total: total_containers };

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

        Ok(docker_types::SystemStatus { daemon_status, resource_usage, system_info, container_stats })
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
    pub async fn exec_command(&self, container_id: &str, command: &[String]) -> DockerResult<String> {
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
    pub async fn get_container_ports(&self, container_id: &str) -> DockerResult<std::collections::HashMap<u16, u16>> {
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
        self.swarm_manager.init(advertise_addr, auto_lock, default_addr_pool, force_new_cluster, subnet_size).await
    }

    /// 加入 Swarm 集群
    pub async fn swarm_join(
        &mut self,
        token: String,
        advertise_addr: Option<String>,
        listen_addr: Option<String>,
        manager_addr: Option<String>,
    ) -> DockerResult<()> {
        self.swarm_manager.join(token, advertise_addr, listen_addr, manager_addr).await
    }

    /// 离开 Swarm 集群
    pub async fn swarm_leave(&mut self, force: bool) -> DockerResult<()> {
        self.swarm_manager.leave(force).await
    }

    /// 获取 Swarm 集群信息
    pub async fn swarm_info(&self) -> DockerResult<docker_types::SwarmInfo> {
        self.swarm_manager.info().await
    }

    /// 更新 Swarm 集群配置
    pub async fn swarm_update(
        &mut self,
        auto_lock: Option<bool>,
        default_addr_pool: Option<String>,
        subnet_size: Option<u8>,
    ) -> DockerResult<()> {
        self.swarm_manager.update(auto_lock, default_addr_pool, subnet_size).await
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
        self.swarm_manager.create_service(name, image, publish, replicas, env, mount).await
    }

    /// 列出 Swarm 服务
    pub async fn list_services(&self) -> DockerResult<Vec<docker_types::ServiceInfo>> {
        self.swarm_manager.list_services().await
    }

    /// 查看 Swarm 服务详情
    pub async fn inspect_service(&self, service: &str) -> DockerResult<docker_types::ServiceInfo> {
        self.swarm_manager.inspect_service(service).await
    }

    /// 更新 Swarm 服务
    pub async fn update_service(
        &mut self,
        service: &str,
        image: Option<String>,
        replicas: Option<u32>,
    ) -> DockerResult<docker_types::ServiceInfo> {
        self.swarm_manager.update_service(service, image, replicas).await
    }

    /// 删除 Swarm 服务
    pub async fn remove_service(&mut self, service: &str) -> DockerResult<()> {
        self.swarm_manager.remove_service(service).await
    }

    /// 扩缩容 Swarm 服务
    pub async fn scale_service(&mut self, service: &str, replicas: u32) -> DockerResult<docker_types::ServiceInfo> {
        self.swarm_manager.scale_service(service, replicas).await
    }

    /// 列出 Swarm 节点
    pub async fn list_nodes(&self) -> DockerResult<Vec<docker_types::NodeInfo>> {
        self.swarm_manager.list_nodes().await
    }

    /// 查看 Swarm 节点详情
    pub async fn inspect_node(&self, node: &str) -> DockerResult<docker_types::NodeInfo> {
        self.swarm_manager.inspect_node(node).await
    }

    /// 更新 Swarm 节点
    pub async fn update_node(
        &mut self,
        node: &str,
        role: Option<String>,
        availability: Option<String>,
    ) -> DockerResult<docker_types::NodeInfo> {
        self.swarm_manager.update_node(node, role, availability).await
    }

    /// 提升 Swarm 节点为 manager
    pub async fn promote_node(&mut self, node: &str) -> DockerResult<()> {
        self.swarm_manager.promote_node(node).await
    }

    /// 降级 Swarm 节点为 worker
    pub async fn demote_node(&mut self, node: &str) -> DockerResult<()> {
        self.swarm_manager.demote_node(node).await
    }

    /// 删除 Swarm 节点
    pub async fn remove_node(&mut self, node: &str) -> DockerResult<()> {
        self.swarm_manager.remove_node(node).await
    }

    // 堆栈管理相关方法

    /// 创建堆栈
    pub async fn stack_deploy(&mut self, name: String, compose_file: String, prune: bool) -> DockerResult<StackInfo> {
        // 加载 Compose 文件
        let config = docker_types::compose::load_single_compose_file(&compose_file)?;
        
        // 验证配置
        docker_types::compose::validate_compose_config(&config)?;
        
        // 解析服务
        let services = docker_types::compose::parse_services(&config)?;
        
        // 解析网络
        let networks = docker_types::compose::parse_networks(&config);
        
        // 解析卷
        let volumes = docker_types::compose::parse_volumes(&config);
        
        // 创建网络
        for network in &networks {
            let network_name = format!("{}_{}", name, network.name);
            let driver = network.driver.clone();
            let enable_ipv6 = network.enable_ipv6;
            let driver_opts = network.driver_opts.clone();
            
            // 尝试创建网络，如果已存在则忽略
            let _ = self.create_network(network_name, driver, enable_ipv6, driver_opts).await;
        }
        
        // 创建卷
        for volume in &volumes {
            let volume_name = format!("{}_{}", name, volume.name);
            let driver = volume.driver.clone();
            let labels = volume.labels.clone();
            
            if !volume.external {
                // 尝试创建卷，如果已存在则忽略
                let _ = self.create_volume(volume_name, driver, labels).await;
            }
        }
        
        // 创建服务
        let mut service_count = 0;
        let mut container_count = 0;
        
        for service in &services {
            let service_name = format!("{}_{}", name, service.name);
            let image = service.image.clone();
            let ports = service.ports.clone();
            let replicas = service.deploy.as_ref().and_then(|d| d.replicas).unwrap_or(1);
            
            // 构建环境变量
            let mut env = Vec::new();
            if let Some(env_map) = &service.environment_map {
                for (key, value) in env_map {
                    env.push(format!("{}={}", key, value));
                }
            }
            env.extend(service.environment.clone());
            
            // 构建卷挂载
            let mut mounts = Vec::new();
            for mount in &service.volumes {
                let source = if mount.mount_type == "volume" {
                    format!("{}_{}", name, mount.source)
                } else {
                    mount.source.clone()
                };
                mounts.push(format!("{}:{}{}", source, mount.target, if mount.read_only { ":ro" } else { "" }));
            }
            
            // 创建服务
            let _ = self.create_service(service_name, image, ports, replicas, env, mounts).await;
            
            service_count += 1;
            container_count += replicas;
        }
        
        // 如果需要清理未使用的服务
        if prune {
            // 实现清理逻辑
        }
        
        Ok(StackInfo {
            name,
            status: "running".to_string(),
            services: service_count,
            containers: container_count,
            created_at: std::time::SystemTime::now(),
        })
    }

    /// 列出堆栈
    pub async fn stack_list(&self) -> DockerResult<Vec<StackInfo>> {
        // 获取所有服务
        let services = self.list_services().await?;
        
        // 按堆栈名称分组
        let mut stack_services: std::collections::HashMap<String, Vec<docker_types::ServiceInfo>> = std::collections::HashMap::new();
        
        for service in services {
            if let Some(stack_name) = service.name.split('_').next() {
                stack_services.entry(stack_name.to_string()).or_default().push(service);
            }
        }
        
        // 构建堆栈列表
        let mut stacks = Vec::new();
        for (stack_name, stack_service_list) in stack_services {
            let service_count = stack_service_list.len() as u32;
            let container_count = stack_service_list.iter().map(|s| s.replicas).sum();
            
            stacks.push(StackInfo {
                name: stack_name,
                status: "running".to_string(),
                services: service_count,
                containers: container_count,
                created_at: std::time::SystemTime::now(),
            });
        }
        
        Ok(stacks)
    }

    /// 查看堆栈详情
    pub async fn stack_inspect(&self, stack: &str) -> DockerResult<StackInfo> {
        // 获取堆栈中的服务
        let services = self.stack_services(stack).await?;
        
        let service_count = services.len() as u32;
        let container_count = services.iter().map(|s| s.replicas).sum();
        
        Ok(StackInfo {
            name: stack.to_string(),
            status: "running".to_string(),
            services: service_count,
            containers: container_count,
            created_at: std::time::SystemTime::now(),
        })
    }

    /// 删除堆栈
    pub async fn stack_rm(&mut self, stack: &str) -> DockerResult<()> {
        // 获取堆栈中的服务
        let services = self.stack_services(stack).await?;
        
        // 删除服务
        for service in services {
            let _ = self.remove_service(&service.id).await;
        }
        
        // 删除网络和卷（这里简化处理，实际需要更复杂的逻辑）
        
        Ok(())
    }

    /// 列出堆栈中的服务
    pub async fn stack_services(&self, stack: &str) -> DockerResult<Vec<docker_types::ServiceInfo>> {
        // 获取所有服务
        let services = self.list_services().await?;
        
        // 过滤出属于该堆栈的服务
        let stack_services = services
            .into_iter()
            .filter(|s| s.name.starts_with(&format!("{}_", stack)))
            .collect();
        
        Ok(stack_services)
    }

    // 用户管理相关方法

    /// 创建用户
    pub fn create_user(&self, username: String, password: String, role: Role) -> DockerResult<user::User> {
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
        self.user_manager.update_user(user_id, username, password, role)
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
