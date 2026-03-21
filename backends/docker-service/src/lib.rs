#![warn(missing_docs)]

#![doc = include_str!("../README.md")]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use docker_network::NetworkManager;
use docker_types::DockerError;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
use tokio::sync::RwLock;
use uuid::Uuid;

/// 结果类型
pub type Result<T> = std::result::Result<T, DockerError>;

/// 服务状态
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ServiceStatus {
    /// 运行中
    Running,
    /// 停止
    Stopped,
    /// 不健康
    Unhealthy,
    /// 启动中
    Starting,
    /// 停止中
    Stopping,
}

/// 服务实例
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInstance {
    /// 实例 ID
    pub id: String,
    /// 服务 ID
    pub service_id: String,
    /// 容器 ID
    pub container_id: String,
    /// 容器名称
    pub container_name: String,
    /// 实例地址
    pub address: SocketAddr,
    /// 实例状态
    pub status: ServiceStatus,
    /// 健康检查状态
    pub health_status: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

/// 负载均衡策略
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LoadBalancingStrategy {
    /// 轮询
    RoundRobin,
    /// 随机
    Random,
    /// 最少连接
    LeastConnections,
    /// IP 哈希
    IpHash,
}

/// 服务配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    /// 服务名称
    pub name: String,
    /// 服务端口
    pub port: u16,
    /// 网络 ID
    pub network_id: String,
    /// 负载均衡策略
    pub load_balancing_strategy: LoadBalancingStrategy,
    /// 健康检查路径
    pub health_check_path: Option<String>,
    /// 健康检查间隔
    pub health_check_interval: Option<u64>,
    /// 服务标签
    pub labels: HashMap<String, String>,
}

/// 服务信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInfo {
    /// 服务 ID
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 服务端口
    pub port: u16,
    /// 网络 ID
    pub network_id: String,
    /// 负载均衡策略
    pub load_balancing_strategy: LoadBalancingStrategy,
    /// 健康检查路径
    pub health_check_path: Option<String>,
    /// 健康检查间隔
    pub health_check_interval: Option<u64>,
    /// 服务实例列表
    pub instances: Vec<ServiceInstance>,
    /// 服务标签
    pub labels: HashMap<String, String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

/// 负载均衡器 trait
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    /// 选择服务实例
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    /// * `client_ip` - 客户端 IP 地址（用于 IP 哈希策略）
    ///
    /// # 返回值
    /// * `Ok(ServiceInstance)` - 选择的服务实例
    /// * `Err(DockerError)` - 选择失败的错误信息
    async fn select_instance(&self, service_id: &str, client_ip: Option<IpAddr>) -> Result<ServiceInstance>;

    /// 更新服务实例状态
    ///
    /// # 参数
    /// * `instance` - 服务实例
    ///
    /// # 返回值
    /// * `Ok(())` - 更新成功
    /// * `Err(DockerError)` - 更新失败的错误信息
    async fn update_instance(&self, instance: &ServiceInstance) -> Result<()>;

    /// 添加服务实例
    ///
    /// # 参数
    /// * `instance` - 服务实例
    ///
    /// # 返回值
    /// * `Ok(())` - 添加成功
    /// * `Err(DockerError)` - 添加失败的错误信息
    async fn add_instance(&self, instance: &ServiceInstance) -> Result<()>;

    /// 移除服务实例
    ///
    /// # 参数
    /// * `instance_id` - 实例 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 移除成功
    /// * `Err(DockerError)` - 移除失败的错误信息
    async fn remove_instance(&self, instance_id: &str) -> Result<()>;
}

/// 服务发现管理器 trait
#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// 注册服务
    ///
    /// # 参数
    /// * `config` - 服务配置
    ///
    /// # 返回值
    /// * `Ok(ServiceInfo)` - 注册成功的服务信息
    /// * `Err(DockerError)` - 注册失败的错误信息
    async fn register_service(&self, config: &ServiceConfig) -> Result<ServiceInfo>;

    /// 注销服务
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 注销成功
    /// * `Err(DockerError)` - 注销失败的错误信息
    async fn deregister_service(&self, service_id: &str) -> Result<()>;

    /// 注册服务实例
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    /// * `instance` - 服务实例
    ///
    /// # 返回值
    /// * `Ok(ServiceInstance)` - 注册成功的服务实例
    /// * `Err(DockerError)` - 注册失败的错误信息
    async fn register_instance(&self, service_id: &str, instance: &ServiceInstance) -> Result<ServiceInstance>;

    /// 注销服务实例
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    /// * `instance_id` - 实例 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 注销成功
    /// * `Err(DockerError)` - 注销失败的错误信息
    async fn deregister_instance(&self, service_id: &str, instance_id: &str) -> Result<()>;

    /// 发现服务
    ///
    /// # 参数
    /// * `service_name` - 服务名称
    ///
    /// # 返回值
    /// * `Ok(ServiceInfo)` - 服务信息
    /// * `Err(DockerError)` - 发现失败的错误信息
    async fn discover_service(&self, service_name: &str) -> Result<ServiceInfo>;

    /// 列出所有服务
    ///
    /// # 返回值
    /// * `Ok(Vec<ServiceInfo>)` - 服务列表
    /// * `Err(DockerError)` - 列出失败的错误信息
    async fn list_services(&self) -> Result<Vec<ServiceInfo>>;

    /// 健康检查
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 健康检查成功
    /// * `Err(DockerError)` - 健康检查失败的错误信息
    async fn health_check(&self, service_id: &str) -> Result<()>;

    /// 负载均衡
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    /// * `client_ip` - 客户端 IP 地址
    ///
    /// # 返回值
    /// * `Ok(ServiceInstance)` - 选择的服务实例
    /// * `Err(DockerError)` - 负载均衡失败的错误信息
    async fn load_balance(&self, service_id: &str, client_ip: Option<IpAddr>) -> Result<ServiceInstance>;
}

/// 服务管理器 trait
#[async_trait]
pub trait ServiceManager: Send + Sync {
    /// 创建服务
    ///
    /// # 参数
    /// * `config` - 服务配置
    ///
    /// # 返回值
    /// * `Ok(ServiceInfo)` - 创建成功的服务信息
    /// * `Err(DockerError)` - 创建失败的错误信息
    async fn create_service(&self, config: &ServiceConfig) -> Result<ServiceInfo>;

    /// 删除服务
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 删除成功
    /// * `Err(DockerError)` - 删除失败的错误信息
    async fn delete_service(&self, service_id: &str) -> Result<()>;

    /// 添加服务实例
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    /// * `container_id` - 容器 ID
    /// * `container_name` - 容器名称
    /// * `address` - 实例地址
    ///
    /// # 返回值
    /// * `Ok(ServiceInstance)` - 添加成功的服务实例
    /// * `Err(DockerError)` - 添加失败的错误信息
    async fn add_service_instance(
        &self,
        service_id: &str,
        container_id: &str,
        container_name: &str,
        address: SocketAddr,
    ) -> Result<ServiceInstance>;

    /// 移除服务实例
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    /// * `instance_id` - 实例 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 移除成功
    /// * `Err(DockerError)` - 移除失败的错误信息
    async fn remove_service_instance(&self, service_id: &str, instance_id: &str) -> Result<()>;

    /// 列出所有服务
    ///
    /// # 返回值
    /// * `Ok(Vec<ServiceInfo>)` - 服务列表
    /// * `Err(DockerError)` - 列出失败的错误信息
    async fn list_services(&self) -> Result<Vec<ServiceInfo>>;

    /// 查看服务详细信息
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    ///
    /// # 返回值
    /// * `Ok(ServiceInfo)` - 服务详细信息
    /// * `Err(DockerError)` - 查看失败的错误信息
    async fn inspect_service(&self, service_id: &str) -> Result<ServiceInfo>;

    /// 服务间通信
    ///
    /// # 参数
    /// * `source_service_id` - 源服务 ID
    /// * `target_service_name` - 目标服务名称
    /// * `request` - 请求数据
    ///
    /// # 返回值
    /// * `Ok(Vec<u8>)` - 响应数据
    /// * `Err(DockerError)` - 通信失败的错误信息
    async fn service_to_service_call(
        &self,
        source_service_id: &str,
        target_service_name: &str,
        request: Vec<u8>,
    ) -> Result<Vec<u8>>;

    /// 负载均衡
    ///
    /// # 参数
    /// * `service_id` - 服务 ID
    /// * `client_ip` - 客户端 IP 地址
    ///
    /// # 返回值
    /// * `Ok(ServiceInstance)` - 选择的服务实例
    /// * `Err(DockerError)` - 负载均衡失败的错误信息
    async fn load_balance(&self, service_id: &str, client_ip: Option<IpAddr>) -> Result<ServiceInstance>;
}

/// 轮询负载均衡器
pub struct RoundRobinLoadBalancer {
    services: Arc<tokio::sync::RwLock<HashMap<String, (usize, Vec<ServiceInstance>)>>>,
}

impl RoundRobinLoadBalancer {
    /// 创建新的轮询负载均衡器
    pub fn new() -> Self {
        Self { services: Arc::new(RwLock::new(HashMap::new())) }
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    async fn select_instance(&self, service_id: &str, _client_ip: Option<IpAddr>) -> Result<ServiceInstance> {
        let mut services = self.services.write().await;

        if let Some((index, instances)) = services.get_mut(service_id) {
            // 过滤出健康的实例
            let healthy_instances: Vec<&ServiceInstance> =
                instances.iter().filter(|inst| inst.status == ServiceStatus::Running && inst.health_status).collect();

            if healthy_instances.is_empty() {
                return Err(DockerError::internal("No healthy instances available"));
            }

            // 轮询选择实例
            let selected_instance = healthy_instances[*index % healthy_instances.len()].clone();
            *index += 1;

            Ok(selected_instance)
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    async fn update_instance(&self, instance: &ServiceInstance) -> Result<()> {
        let mut services = self.services.write().await;

        if let Some((_, instances)) = services.get_mut(&instance.service_id) {
            if let Some(idx) = instances.iter().position(|inst| inst.id == instance.id) {
                instances[idx] = instance.clone();
                Ok(())
            }
            else {
                Err(DockerError::not_found("instance", instance.id.clone()))
            }
        }
        else {
            Err(DockerError::not_found("service", instance.service_id.clone()))
        }
    }

    async fn add_instance(&self, instance: &ServiceInstance) -> Result<()> {
        let mut services = self.services.write().await;

        services.entry(instance.service_id.clone()).or_insert((0, Vec::new())).1.push(instance.clone());

        Ok(())
    }

    async fn remove_instance(&self, instance_id: &str) -> Result<()> {
        let mut services = self.services.write().await;

        for (_, (_, instances)) in services.iter_mut() {
            if let Some(idx) = instances.iter().position(|inst| inst.id == instance_id) {
                instances.remove(idx);
                return Ok(());
            }
        }

        Err(DockerError::not_found("instance", instance_id.to_string()))
    }
}

/// 随机负载均衡器
pub struct RandomLoadBalancer {
    services: Arc<tokio::sync::RwLock<HashMap<String, Vec<ServiceInstance>>>>,
}

impl RandomLoadBalancer {
    /// 创建新的随机负载均衡器
    pub fn new() -> Self {
        Self { services: Arc::new(tokio::sync::RwLock::new(HashMap::new())) }
    }
}

#[async_trait]
impl LoadBalancer for RandomLoadBalancer {
    async fn select_instance(&self, service_id: &str, _client_ip: Option<IpAddr>) -> Result<ServiceInstance> {
        let services = self.services.read().await;

        if let Some(instances) = services.get(service_id) {
            // 过滤出健康的实例
            let healthy_instances: Vec<&ServiceInstance> =
                instances.iter().filter(|inst| inst.status == ServiceStatus::Running && inst.health_status).collect();

            if healthy_instances.is_empty() {
                return Err(DockerError::internal("No healthy instances available"));
            }

            // 随机选择实例
            let mut rng = thread_rng();
            let index = rng.gen_range(0..healthy_instances.len());
            let selected_instance = healthy_instances[index].clone();

            Ok(selected_instance)
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    async fn update_instance(&self, instance: &ServiceInstance) -> Result<()> {
        let mut services = self.services.write().await;

        if let Some(instances) = services.get_mut(&instance.service_id) {
            if let Some(idx) = instances.iter().position(|inst| inst.id == instance.id) {
                instances[idx] = instance.clone();
                Ok(())
            }
            else {
                Err(DockerError::not_found("instance", instance.id.clone()))
            }
        }
        else {
            Err(DockerError::not_found("service", instance.service_id.clone()))
        }
    }

    async fn add_instance(&self, instance: &ServiceInstance) -> Result<()> {
        let mut services = self.services.write().await;

        services.entry(instance.service_id.clone()).or_insert_with(Vec::new).push(instance.clone());

        Ok(())
    }

    async fn remove_instance(&self, instance_id: &str) -> Result<()> {
        let mut services = self.services.write().await;

        for (_, instances) in services.iter_mut() {
            if let Some(idx) = instances.iter().position(|inst| inst.id == instance_id) {
                instances.remove(idx);
                return Ok(());
            }
        }

        Err(DockerError::not_found("instance", instance_id.to_string()))
    }
}

/// 最少连接负载均衡器
pub struct LeastConnectionsLoadBalancer {
    services: Arc<tokio::sync::RwLock<HashMap<String, Vec<ServiceInstance>>>>,
    connections: Arc<tokio::sync::RwLock<HashMap<String, usize>>>, // 实例 ID 到连接数的映射
}

impl LeastConnectionsLoadBalancer {
    /// 创建新的最少连接负载均衡器
    pub fn new() -> Self {
        Self {
            services: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            connections: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl LoadBalancer for LeastConnectionsLoadBalancer {
    async fn select_instance(&self, service_id: &str, _client_ip: Option<IpAddr>) -> Result<ServiceInstance> {
        let services = self.services.read().await;
        let connections = self.connections.read().await;

        if let Some(instances) = services.get(service_id) {
            // 过滤出健康的实例
            let healthy_instances: Vec<&ServiceInstance> =
                instances.iter().filter(|inst| inst.status == ServiceStatus::Running && inst.health_status).collect();

            if healthy_instances.is_empty() {
                return Err(DockerError::internal("No healthy instances available"));
            }

            // 选择连接数最少的实例
            let mut selected_instance = healthy_instances[0];
            let mut min_connections = *connections.get(&selected_instance.id).unwrap_or(&0);

            for instance in &healthy_instances[1..] {
                let conn_count = *connections.get(&instance.id).unwrap_or(&0);
                if conn_count < min_connections {
                    min_connections = conn_count;
                    selected_instance = instance;
                }
            }

            // 增加连接数
            let mut connections = self.connections.write().await;
            *connections.entry(selected_instance.id.clone()).or_insert(0) += 1;

            Ok(selected_instance.clone())
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    async fn update_instance(&self, instance: &ServiceInstance) -> Result<()> {
        let mut services = self.services.write().await;

        if let Some(instances) = services.get_mut(&instance.service_id) {
            if let Some(idx) = instances.iter().position(|inst| inst.id == instance.id) {
                instances[idx] = instance.clone();
                Ok(())
            }
            else {
                Err(DockerError::not_found("instance", instance.id.clone()))
            }
        }
        else {
            Err(DockerError::not_found("service", instance.service_id.clone()))
        }
    }

    async fn add_instance(&self, instance: &ServiceInstance) -> Result<()> {
        let mut services = self.services.write().await;

        services.entry(instance.service_id.clone()).or_insert_with(Vec::new).push(instance.clone());

        Ok(())
    }

    async fn remove_instance(&self, instance_id: &str) -> Result<()> {
        let mut services = self.services.write().await;
        let mut connections = self.connections.write().await;

        // 从连接数映射中移除
        connections.remove(instance_id);

        // 从服务实例列表中移除
        for (_, instances) in services.iter_mut() {
            if let Some(idx) = instances.iter().position(|inst| inst.id == instance_id) {
                instances.remove(idx);
                return Ok(());
            }
        }

        Err(DockerError::not_found("instance", instance_id.to_string()))
    }
}

/// 减少实例的连接数
///
/// # 参数
/// * `instance_id` - 实例 ID
///
/// # 返回值
/// * `Ok(())` - 减少成功
/// * `Err(DockerError)` - 减少失败的错误信息
pub async fn decrease_connections(load_balancer: &LeastConnectionsLoadBalancer, instance_id: &str) -> Result<()> {
    let mut connections = load_balancer.connections.write().await;

    if let Some(count) = connections.get_mut(instance_id) {
        if *count > 0 {
            *count -= 1;
        }
        Ok(())
    }
    else {
        Err(DockerError::not_found("instance", instance_id.to_string()))
    }
}

/// IP 哈希负载均衡器
pub struct IpHashLoadBalancer {
    services: Arc<tokio::sync::RwLock<HashMap<String, Vec<ServiceInstance>>>>,
}

impl IpHashLoadBalancer {
    /// 创建新的 IP 哈希负载均衡器
    pub fn new() -> Self {
        Self { services: Arc::new(tokio::sync::RwLock::new(HashMap::new())) }
    }

    /// 计算 IP 地址的哈希值
    fn hash_ip(&self, ip: &IpAddr) -> u64 {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };

        let mut hasher = DefaultHasher::new();
        ip.hash(&mut hasher);
        hasher.finish()
    }
}

#[async_trait]
impl LoadBalancer for IpHashLoadBalancer {
    async fn select_instance(&self, service_id: &str, client_ip: Option<IpAddr>) -> Result<ServiceInstance> {
        let services = self.services.read().await;

        if let Some(instances) = services.get(service_id) {
            // 过滤出健康的实例
            let healthy_instances: Vec<&ServiceInstance> =
                instances.iter().filter(|inst| inst.status == ServiceStatus::Running && inst.health_status).collect();

            if healthy_instances.is_empty() {
                return Err(DockerError::internal("No healthy instances available"));
            }

            // 根据客户端 IP 计算哈希值，选择实例
            let index = if let Some(ip) = client_ip {
                let hash = self.hash_ip(&ip);
                (hash % healthy_instances.len() as u64) as usize
            }
            else {
                // 如果没有客户端 IP，使用随机选择
                let mut rng = thread_rng();
                rng.gen_range(0..healthy_instances.len())
            };

            let selected_instance = healthy_instances[index].clone();
            Ok(selected_instance)
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    async fn update_instance(&self, instance: &ServiceInstance) -> Result<()> {
        let mut services = self.services.write().await;

        if let Some(instances) = services.get_mut(&instance.service_id) {
            if let Some(idx) = instances.iter().position(|inst| inst.id == instance.id) {
                instances[idx] = instance.clone();
                Ok(())
            }
            else {
                Err(DockerError::not_found("instance", instance.id.clone()))
            }
        }
        else {
            Err(DockerError::not_found("service", instance.service_id.clone()))
        }
    }

    async fn add_instance(&self, instance: &ServiceInstance) -> Result<()> {
        let mut services = self.services.write().await;

        services.entry(instance.service_id.clone()).or_insert_with(Vec::new).push(instance.clone());

        Ok(())
    }

    async fn remove_instance(&self, instance_id: &str) -> Result<()> {
        let mut services = self.services.write().await;

        for (_, instances) in services.iter_mut() {
            if let Some(idx) = instances.iter().position(|inst| inst.id == instance_id) {
                instances.remove(idx);
                return Ok(());
            }
        }

        Err(DockerError::not_found("instance", instance_id.to_string()))
    }
}

/// 服务发现管理器
pub struct ServiceDiscoveryManager {
    services: Arc<tokio::sync::RwLock<HashMap<String, ServiceInfo>>>,
    service_name_map: Arc<tokio::sync::RwLock<HashMap<String, String>>>, // 服务名称到服务 ID 的映射
    load_balancer: Arc<dyn LoadBalancer>,
    network_manager: Arc<dyn NetworkManager>,
}

impl ServiceDiscoveryManager {
    /// 创建新的服务发现管理器
    pub fn new(network_manager: Arc<dyn NetworkManager>) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_name_map: Arc::new(RwLock::new(HashMap::new())),
            load_balancer: Arc::new(RoundRobinLoadBalancer::new()),
            network_manager,
        }
    }

    /// 根据负载均衡策略创建负载均衡器
    pub fn create_load_balancer(strategy: LoadBalancingStrategy) -> Arc<dyn LoadBalancer> {
        match strategy {
            LoadBalancingStrategy::RoundRobin => Arc::new(RoundRobinLoadBalancer::new()),
            LoadBalancingStrategy::Random => Arc::new(RandomLoadBalancer::new()),
            LoadBalancingStrategy::LeastConnections => Arc::new(LeastConnectionsLoadBalancer::new()),
            LoadBalancingStrategy::IpHash => Arc::new(IpHashLoadBalancer::new()),
        }
    }

    /// 执行健康检查
    async fn perform_health_check(instance: &ServiceInstance, service_info: &ServiceInfo) -> bool {
        // 根据服务配置执行不同类型的健康检查
        if let Some(health_check_path) = &service_info.health_check_path {
            // HTTP 健康检查
            Self::perform_http_health_check(instance, health_check_path).await
        }
        else {
            // TCP 健康检查
            Self::perform_tcp_health_check(instance).await
        }
    }

    /// 执行 HTTP 健康检查
    async fn perform_http_health_check(instance: &ServiceInstance, health_check_path: &str) -> bool {
        let url = format!("http://{}{}", instance.address, health_check_path);

        match wae_request::get(&url).send().await {
            Ok(response) => response.is_success(),
            Err(_) => false,
        }
    }

    /// 执行 TCP 健康检查
    async fn perform_tcp_health_check(instance: &ServiceInstance) -> bool {
        use tokio::net::TcpStream;

        match TcpStream::connect(instance.address).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[async_trait]
impl ServiceDiscovery for ServiceDiscoveryManager {
    async fn register_service(&self, config: &ServiceConfig) -> Result<ServiceInfo> {
        let service_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let service_info = ServiceInfo {
            id: service_id.clone(),
            name: config.name.clone(),
            port: config.port,
            network_id: config.network_id.clone(),
            load_balancing_strategy: config.load_balancing_strategy.clone(),
            health_check_path: config.health_check_path.clone(),
            health_check_interval: config.health_check_interval,
            instances: Vec::new(),
            labels: config.labels.clone(),
            created_at: now,
            updated_at: now,
        };

        let mut services = self.services.write().await;
        let mut service_name_map = self.service_name_map.write().await;

        services.insert(service_id.clone(), service_info.clone());
        service_name_map.insert(config.name.clone(), service_id);

        Ok(service_info)
    }

    async fn deregister_service(&self, service_id: &str) -> Result<()> {
        let mut services = self.services.write().await;
        let mut service_name_map = self.service_name_map.write().await;

        if let Some(service_info) = services.remove(service_id) {
            service_name_map.remove(&service_info.name);
            Ok(())
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    async fn register_instance(&self, service_id: &str, instance: &ServiceInstance) -> Result<ServiceInstance> {
        let mut services = self.services.write().await;

        if let Some(service_info) = services.get_mut(service_id) {
            service_info.instances.push(instance.clone());
            service_info.updated_at = Utc::now();

            // 添加到负载均衡器
            self.load_balancer.add_instance(instance).await?;

            Ok(instance.clone())
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    async fn deregister_instance(&self, service_id: &str, instance_id: &str) -> Result<()> {
        let mut services = self.services.write().await;

        if let Some(service_info) = services.get_mut(service_id) {
            if let Some(idx) = service_info.instances.iter().position(|inst| inst.id == instance_id) {
                service_info.instances.remove(idx);
                service_info.updated_at = Utc::now();

                // 从负载均衡器中移除
                self.load_balancer.remove_instance(instance_id).await?;

                Ok(())
            }
            else {
                Err(DockerError::not_found("instance", instance_id.to_string()))
            }
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    async fn discover_service(&self, service_name: &str) -> Result<ServiceInfo> {
        // 从存储中获取服务信息
        let service_name_map = self.service_name_map.read().await;

        if let Some(service_id) = service_name_map.get(service_name) {
            let services = self.services.read().await;

            if let Some(service_info) = services.get(service_id) {
                Ok(service_info.clone())
            }
            else {
                Err(DockerError::not_found("service", service_id.to_string()))
            }
        }
        else {
            Err(DockerError::not_found("service", service_name.to_string()))
        }
    }

    async fn list_services(&self) -> Result<Vec<ServiceInfo>> {
        let services = self.services.read().await;
        Ok(services.values().cloned().collect())
    }

    async fn health_check(&self, service_id: &str) -> Result<()> {
        let mut services = self.services.write().await;

        if let Some(service_info) = services.get_mut(service_id) {
            // 先获取服务信息的克隆，避免可变借用和不可变借用的冲突
            let service_info_clone = service_info.clone();

            for instance in &mut service_info.instances {
                // 执行健康检查
                let health_status = ServiceDiscoveryManager::perform_health_check(instance, &service_info_clone).await;

                // 更新实例状态
                instance.health_status = health_status;
                instance.updated_at = Utc::now();

                // 更新负载均衡器中的实例状态
                self.load_balancer.update_instance(instance).await?;
            }

            Ok(())
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    async fn load_balance(&self, service_id: &str, client_ip: Option<IpAddr>) -> Result<ServiceInstance> {
        self.load_balancer.select_instance(service_id, client_ip).await
    }
}

/// 服务管理器
///
/// 服务管理器是服务发现和负载均衡功能的主要入口点，提供了创建服务、添加服务实例、
/// 发现服务、负载均衡和服务间通信等功能。
///
/// # 示例
/// ```rust
/// use docker_network::new_network_manager;
/// use docker_service::{LoadBalancingStrategy, ServiceConfig, new_service_manager};
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
///
/// #[tokio::main]
/// async fn main() {
///     // 创建网络管理器
///     let network_manager = new_network_manager();
///
///     // 创建服务管理器
///     let service_manager = new_service_manager(network_manager);
///
///     // 创建服务配置
///     let config = ServiceConfig {
///         name: "my-service".to_string(),
///         port: 8080,
///         network_id: "default".to_string(),
///         load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
///         health_check_path: Some("/health".to_string()),
///         health_check_interval: Some(30),
///         labels: Default::default(),
///     };
///
///     // 创建服务
///     let service = service_manager.create_service(&config).await.unwrap();
///
///     // 添加服务实例
///     let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
///     let instance = service_manager
///         .add_service_instance(&service.id, "container-1", "my-container", address)
///         .await
///         .unwrap();
/// }
/// ```
pub struct ServiceManagerImpl {
    service_discovery: Arc<dyn ServiceDiscovery>,
    network_manager: Arc<dyn NetworkManager>,
}

impl ServiceManagerImpl {
    /// 创建新的服务管理器
    pub fn new(network_manager: Arc<dyn NetworkManager>) -> Self {
        let service_discovery = Arc::new(ServiceDiscoveryManager::new(network_manager.clone()));

        Self { service_discovery, network_manager }
    }

    /// 执行 HTTP 服务调用
    async fn perform_http_service_call(instance: &ServiceInstance, request: Vec<u8>) -> Result<Vec<u8>> {
        let url = format!("http://{}/", instance.address);

        let response = wae_request::post(&url)
            .body(request)
            .send()
            .await
            .map_err(|e| DockerError::internal(format!("HTTP service call failed: {}", e)))?;

        Ok(response.body)
    }

    /// 执行 TCP 服务调用
    async fn perform_tcp_service_call(instance: &ServiceInstance, request: Vec<u8>) -> Result<Vec<u8>> {
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpStream,
        };

        let mut stream = TcpStream::connect(instance.address)
            .await
            .map_err(|e| DockerError::internal(format!("TCP connection failed: {}", e)))?;

        // 发送请求
        stream.write_all(&request).await.map_err(|e| DockerError::internal(format!("Failed to send TCP request: {}", e)))?;

        // 读取响应
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .map_err(|e| DockerError::internal(format!("Failed to read TCP response: {}", e)))?;

        Ok(response)
    }
}

#[async_trait]
impl ServiceManager for ServiceManagerImpl {
    async fn create_service(&self, config: &ServiceConfig) -> Result<ServiceInfo> {
        self.service_discovery.register_service(config).await
    }

    async fn delete_service(&self, service_id: &str) -> Result<()> {
        self.service_discovery.deregister_service(service_id).await
    }

    async fn add_service_instance(
        &self,
        service_id: &str,
        container_id: &str,
        container_name: &str,
        address: SocketAddr,
    ) -> Result<ServiceInstance> {
        let instance = ServiceInstance {
            id: Uuid::new_v4().to_string(),
            service_id: service_id.to_string(),
            container_id: container_id.to_string(),
            container_name: container_name.to_string(),
            address,
            status: ServiceStatus::Running,
            health_status: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.service_discovery.register_instance(service_id, &instance).await
    }

    async fn remove_service_instance(&self, service_id: &str, instance_id: &str) -> Result<()> {
        self.service_discovery.deregister_instance(service_id, instance_id).await
    }

    async fn list_services(&self) -> Result<Vec<ServiceInfo>> {
        self.service_discovery.list_services().await
    }

    async fn inspect_service(&self, service_id: &str) -> Result<ServiceInfo> {
        let services = self.service_discovery.list_services().await?;

        if let Some(service_info) = services.into_iter().find(|s| s.id == service_id) {
            Ok(service_info)
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    async fn service_to_service_call(
        &self,
        _source_service_id: &str,
        target_service_name: &str,
        request: Vec<u8>,
    ) -> Result<Vec<u8>> {
        // 发现目标服务
        let target_service = self.service_discovery.discover_service(target_service_name).await?;

        // 尝试多次调用，支持重试
        let max_retries = 3;
        let mut last_error: Option<DockerError> = None;

        for attempt in 0..max_retries {
            // 负载均衡选择实例
            let instance = match self.service_discovery.load_balance(&target_service.id, None).await {
                Ok(instance) => instance,
                Err(e) => {
                    last_error = Some(e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    continue;
                }
            };

            // 尝试 HTTP 调用
            match Self::perform_http_service_call(&instance, request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    continue;
                }
            };
        }

        Err(last_error.unwrap_or_else(|| DockerError::internal("Service call failed after multiple attempts")))
    }

    async fn load_balance(&self, service_id: &str, client_ip: Option<IpAddr>) -> Result<ServiceInstance> {
        // 直接调用服务发现管理器的负载均衡方法
        self.service_discovery.load_balance(service_id, client_ip).await
    }
}

/// 创建服务管理器
///
/// # 参数
/// * `network_manager` - 网络管理器实例
///
/// # 返回值
/// * `Box<dyn ServiceManager>` - 服务管理器实例
pub fn new_service_manager(network_manager: Arc<dyn NetworkManager>) -> Box<dyn ServiceManager> {
    Box::new(ServiceManagerImpl::new(network_manager))
}

/// 创建服务发现管理器
///
/// # 参数
/// * `network_manager` - 网络管理器实例
///
/// # 返回值
/// * `Box<dyn ServiceDiscovery>` - 服务发现管理器实例
pub fn new_service_discovery(network_manager: Arc<dyn NetworkManager>) -> Box<dyn ServiceDiscovery> {
    Box::new(ServiceDiscoveryManager::new(network_manager))
}

/// 创建轮询负载均衡器
///
/// # 返回值
/// * `Box<dyn LoadBalancer>` - 轮询负载均衡器实例
pub fn new_round_robin_load_balancer() -> Box<dyn LoadBalancer> {
    Box::new(RoundRobinLoadBalancer::new())
}

/// 创建随机负载均衡器
///
/// # 返回值
/// * `Box<dyn LoadBalancer>` - 随机负载均衡器实例
pub fn new_random_load_balancer() -> Box<dyn LoadBalancer> {
    Box::new(RandomLoadBalancer::new())
}
