#![warn(missing_docs)]

//! Swarm 集群管理模块
//!
//! 实现 Docker Swarm 集群的核心功能，包括：
//! - 集群初始化和管理
//! - 节点加入和退出
//! - 集群状态同步
//! - 服务编排和调度

use std::{
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime},
};
use tokio::sync::broadcast;
use uuid::Uuid;

use docker_types::{
    DockerError, NodeAvailability, NodeInfo, NodeRole, NodeStatus, Result as DockerResult, ServiceInfo, ServiceStatus,
    ServiceVersionInfo, SwarmInfo, UpdateConfig,
};

/// Swarm 集群状态
#[derive(Debug, Clone)]
pub struct SwarmState {
    /// 集群 ID
    pub id: String,
    /// 集群名称
    pub name: Option<String>,
    /// 管理节点列表
    pub managers: Vec<NodeInfo>,
    /// 工作节点列表
    pub workers: Vec<NodeInfo>,
    /// 服务列表
    pub services: Vec<ServiceInfo>,
    /// 集群版本
    pub version: String,
    /// 集群创建时间
    pub created_at: SystemTime,
    /// 最后更新时间
    pub updated_at: SystemTime,
    /// 自动锁定
    pub auto_lock: bool,
    /// 默认地址池
    pub default_addr_pool: Option<String>,
    /// 子网大小
    pub subnet_size: u8,
}

/// Swarm 管理器
pub struct SwarmManager {
    /// 集群状态
    state: Arc<RwLock<SwarmState>>,
    /// 事件广播器
    tx: Arc<broadcast::Sender<SwarmEvent>>,
    /// 本地节点信息
    local_node: Arc<Mutex<Option<NodeInfo>>>,
    /// 是否处于 Swarm 模式
    in_swarm: Arc<Mutex<bool>>,
}

/// Swarm 事件类型
#[derive(Debug, Clone)]
pub enum SwarmEvent {
    /// 节点加入
    NodeJoin(NodeInfo),
    /// 节点离开
    NodeLeave(String), // 节点 ID
    /// 节点状态变化
    NodeStatusChange(String, NodeStatus), // 节点 ID, 新状态
    /// 服务创建
    ServiceCreate(ServiceInfo),
    /// 服务更新
    ServiceUpdate(ServiceInfo),
    /// 服务删除
    ServiceDelete(String), // 服务 ID
    /// 集群状态变化
    ClusterStateChange(SwarmState),
}

impl SwarmManager {
    /// 创建新的 Swarm 管理器
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);

        Self {
            state: Arc::new(RwLock::new(SwarmState {
                id: String::new(),
                name: None,
                managers: Vec::new(),
                workers: Vec::new(),
                services: Vec::new(),
                version: "1.29.0".to_string(),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                auto_lock: false,
                default_addr_pool: None,
                subnet_size: 24,
            })),
            tx: Arc::new(tx),
            local_node: Arc::new(Mutex::new(None)),
            in_swarm: Arc::new(Mutex::new(false)),
        }
    }

    /// 初始化 Swarm 集群
    pub async fn init(
        &self,
        advertise_addr: Option<String>,
        auto_lock: bool,
        default_addr_pool: Option<String>,
        force_new_cluster: bool,
        subnet_size: u8,
    ) -> DockerResult<()> {
        // 检查是否已经在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if in_swarm && !force_new_cluster {
            return Err(DockerError::internal("Node is already in a swarm"));
        }

        // 生成集群 ID
        let swarm_id = format!("swarm-{}", Uuid::new_v4());

        // 创建本地节点信息
        let local_node = NodeInfo {
            id: format!("node-{}", Uuid::new_v4()),
            name: hostname::get().unwrap_or_else(|_| "localhost".into()).to_string_lossy().to_string(),
            role: NodeRole::Manager,
            availability: NodeAvailability::Active,
            status: NodeStatus::Ready,
            address: advertise_addr.unwrap_or_else(|| "127.0.0.1:2377".to_string()),
            version: "1.29.0".to_string(),
            containers_running: 0,
            labels: std::collections::HashMap::new(),
        };

        // 更新集群状态
        let mut state = self.state.write().unwrap();
        state.id = swarm_id;
        state.name = None;
        state.managers = vec![local_node.clone()];
        state.workers = Vec::new();
        state.services = Vec::new();
        state.created_at = SystemTime::now();
        state.updated_at = SystemTime::now();
        state.auto_lock = auto_lock;
        state.default_addr_pool = default_addr_pool;
        state.subnet_size = subnet_size;

        // 更新本地节点信息
        *self.local_node.lock().await = Some(local_node.clone());
        *self.in_swarm.lock().await = true;

        // 广播集群状态变化事件
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();
        self.tx.send(SwarmEvent::NodeJoin(local_node)).unwrap();

        Ok(())
    }

    /// 加入 Swarm 集群
    pub async fn join(
        &self,
        token: String,
        advertise_addr: Option<String>,
        listen_addr: Option<String>,
        manager_addr: Option<String>,
    ) -> DockerResult<()> {
        // 检查是否已经在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if in_swarm {
            return Err(DockerError::internal("Node is already in a swarm"));
        }

        // 验证 token
        if token.is_empty() {
            return Err(DockerError::internal("Invalid join token"));
        }

        // 确定节点角色（基于 token 类型）
        let role = if token.starts_with("SWMTKN-1-0-") { NodeRole::Manager } else { NodeRole::Worker };

        // 创建本地节点信息
        let local_node = NodeInfo {
            id: format!("node-{}", Uuid::new_v4()),
            name: hostname::get().unwrap_or_else(|_| "localhost".into()).to_string_lossy().to_string(),
            role,
            availability: NodeAvailability::Active,
            status: NodeStatus::Ready,
            address: advertise_addr.unwrap_or_else(|| "127.0.0.1:2377".to_string()),
            version: "1.29.0".to_string(),
            containers_running: 0,
            labels: std::collections::HashMap::new(),
        };

        // 模拟连接到管理节点并获取集群状态
        // 实际实现中，这里应该通过网络连接到管理节点，验证 token，获取集群状态
        let swarm_id = format!("swarm-{}", Uuid::new_v4());

        // 更新集群状态
        let mut state = self.state.write().unwrap();
        state.id = swarm_id;
        state.name = None;
        state.managers = if role == NodeRole::Manager {
            vec![local_node.clone()]
        }
        else {
            Vec::new() // 实际实现中应该从管理节点获取
        };
        state.workers = if role == NodeRole::Worker {
            vec![local_node.clone()]
        }
        else {
            Vec::new() // 实际实现中应该从管理节点获取
        };
        state.services = Vec::new(); // 实际实现中应该从管理节点获取
        state.created_at = SystemTime::now();
        state.updated_at = SystemTime::now();

        // 更新本地节点信息
        *self.local_node.lock().unwrap() = Some(local_node.clone());
        *self.in_swarm.lock().unwrap() = true;

        // 广播集群状态变化事件
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();
        self.tx.send(SwarmEvent::NodeJoin(local_node)).unwrap();

        Ok(())
    }

    /// 离开 Swarm 集群
    pub async fn leave(&self, force: bool) -> DockerResult<()> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        let local_node = self.local_node.lock().unwrap();
        let node_id = local_node.as_ref().map(|n| n.id.clone()).unwrap_or_default();

        // 检查是否是最后一个管理节点
        let state = self.state.read().unwrap();
        if state.managers.len() == 1 && state.managers[0].id == node_id && !force {
            return Err(DockerError::internal(
                "This is the last manager in the swarm. Use --force to remove it",
            ));
        }

        // 重置集群状态
        let mut state = self.state.write().unwrap();
        state.id = String::new();
        state.name = None;
        state.managers = Vec::new();
        state.workers = Vec::new();
        state.services = Vec::new();
        state.created_at = SystemTime::now();
        state.updated_at = SystemTime::now();

        // 重置本地节点信息
        *self.local_node.lock().unwrap() = None;
        *self.in_swarm.lock().unwrap() = false;

        // 广播节点离开事件
        self.tx.send(SwarmEvent::NodeLeave(node_id)).unwrap();
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();

        Ok(())
    }

    /// 获取 Swarm 集群信息
    pub async fn info(&self) -> DockerResult<SwarmInfo> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        let state = self.state.read().unwrap();
        Ok(SwarmInfo {
            id: state.id.clone(),
            name: state.name.clone(),
            managers: state.managers.len() as u32,
            workers: state.workers.len() as u32,
            services: state.services.len() as u32,
            tasks: state.services.iter().map(|s| s.replicas).sum(),
            version: state.version.clone(),
            created_at: state.created_at,
        })
    }

    /// 更新 Swarm 集群配置
    pub async fn update(
        &self,
        auto_lock: Option<bool>,
        default_addr_pool: Option<String>,
        subnet_size: Option<u8>,
    ) -> DockerResult<()> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can update the swarm"));
        }

        // 更新集群状态
        let mut state = self.state.write().unwrap();
        if let Some(auto_lock) = auto_lock {
            state.auto_lock = auto_lock;
        }
        if let Some(default_addr_pool) = default_addr_pool {
            state.default_addr_pool = Some(default_addr_pool);
        }
        if let Some(subnet_size) = subnet_size {
            state.subnet_size = subnet_size;
        }
        state.updated_at = SystemTime::now();

        // 广播集群状态变化事件
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();

        Ok(())
    }

    /// 列出 Swarm 节点
    pub async fn list_nodes(&self) -> DockerResult<Vec<NodeInfo>> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        let state = self.state.read().unwrap();
        let mut nodes = state.managers.clone();
        nodes.extend(state.workers.clone());
        Ok(nodes)
    }

    /// 查看 Swarm 节点详情
    pub async fn inspect_node(&self, node_id: &str) -> DockerResult<NodeInfo> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        let state = self.state.read().unwrap();
        if let Some(node) = state.managers.iter().find(|n| n.id == node_id) {
            Ok(node.clone())
        }
        else if let Some(node) = state.workers.iter().find(|n| n.id == node_id) {
            Ok(node.clone())
        }
        else {
            Err(DockerError::not_found("node", node_id.to_string()))
        }
    }

    /// 更新 Swarm 节点
    pub async fn update_node(
        &self,
        node_id: &str,
        role: Option<String>,
        availability: Option<String>,
    ) -> DockerResult<NodeInfo> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can update nodes"));
        }

        let mut state = self.state.write().unwrap();
        let mut updated_node = None;

        // 查找并更新节点
        if let Some(index) = state.managers.iter().position(|n| n.id == node_id) {
            let mut node = state.managers[index].clone();
            if let Some(role) = role {
                node.role = match role.as_str() {
                    "manager" => NodeRole::Manager,
                    "worker" => NodeRole::Worker,
                    _ => return Err(DockerError::swarm_error(format!("Invalid role: {}", role))),
                };
            }
            if let Some(availability) = availability {
                node.availability = match availability.as_str() {
                    "active" => NodeAvailability::Active,
                    "pause" => NodeAvailability::Pause,
                    "drain" => NodeAvailability::Drain,
                    _ => return Err(DockerError::swarm_error(format!("Invalid availability: {}", availability))),
                };
            }
            state.managers[index] = node.clone();
            updated_node = Some(node);
        }
        else if let Some(index) = state.workers.iter().position(|n| n.id == node_id) {
            let mut node = state.workers[index].clone();
            if let Some(role) = role {
                node.role = match role.as_str() {
                    "manager" => NodeRole::Manager,
                    "worker" => NodeRole::Worker,
                    _ => return Err(DockerError::swarm_error(format!("Invalid role: {}", role))),
                };
            }
            if let Some(availability) = availability {
                node.availability = match availability.as_str() {
                    "active" => NodeAvailability::Active,
                    "pause" => NodeAvailability::Pause,
                    "drain" => NodeAvailability::Drain,
                    _ => return Err(DockerError::swarm_error(format!("Invalid availability: {}", availability))),
                };
            }
            state.workers[index] = node.clone();
            updated_node = Some(node);
        }
        else {
            return Err(DockerError::not_found("node", node_id.to_string()));
        }

        state.updated_at = SystemTime::now();

        // 广播集群状态变化事件
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();

        Ok(updated_node.unwrap())
    }

    /// 提升节点为 manager
    pub async fn promote_node(&self, node_id: &str) -> DockerResult<()> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can promote nodes"));
        }

        let mut state = self.state.write().unwrap();

        // 查找并提升节点
        if let Some(index) = state.workers.iter().position(|n| n.id == node_id) {
            let mut node = state.workers.remove(index);
            node.role = NodeRole::Manager;
            state.managers.push(node.clone());
            state.updated_at = SystemTime::now();

            // 广播集群状态变化事件
            self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();
            self.tx.send(SwarmEvent::NodeStatusChange(node.id, node.status)).unwrap();

            Ok(())
        }
        else {
            return Err(DockerError::not_found("node", node_id.to_string()));
        }
    }

    /// 降级节点为 worker
    pub async fn demote_node(&self, node_id: &str) -> DockerResult<()> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can demote nodes"));
        }

        let mut state = self.state.write().unwrap();

        // 检查是否是最后一个管理节点
        if state.managers.len() == 1 && state.managers[0].id == node_id {
            return Err(DockerError::internal("Cannot demote the last manager in the swarm"));
        }

        // 查找并降级节点
        if let Some(index) = state.managers.iter().position(|n| n.id == node_id) {
            let mut node = state.managers.remove(index);
            node.role = NodeRole::Worker;
            state.workers.push(node.clone());
            state.updated_at = SystemTime::now();

            // 广播集群状态变化事件
            self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();
            self.tx.send(SwarmEvent::NodeStatusChange(node.id, node.status)).unwrap();

            Ok(())
        }
        else {
            return Err(DockerError::not_found("node", node_id.to_string()));
        }
    }

    /// 删除 Swarm 节点
    pub async fn remove_node(&self, node_id: &str) -> DockerResult<()> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can remove nodes"));
        }

        let mut state = self.state.write().unwrap();
        let mut node_removed = false;

        // 查找并删除节点
        if let Some(index) = state.managers.iter().position(|n| n.id == node_id) {
            // 检查是否是最后一个管理节点
            if state.managers.len() == 1 {
                return Err(DockerError::internal("Cannot remove the last manager in the swarm"));
            }
            state.managers.remove(index);
            node_removed = true;
        }
        else if let Some(index) = state.workers.iter().position(|n| n.id == node_id) {
            state.workers.remove(index);
            node_removed = true;
        }

        if !node_removed {
            return Err(DockerError::not_found("node", node_id.to_string()));
        }

        state.updated_at = SystemTime::now();

        // 广播节点离开事件
        self.tx.send(SwarmEvent::NodeLeave(node_id.to_string())).unwrap();
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();

        Ok(())
    }

    /// 创建 Swarm 服务
    pub async fn create_service(
        &self,
        name: String,
        image: String,
        publish: Vec<String>,
        replicas: u32,
        env: Vec<String>,
        mount: Vec<String>,
    ) -> DockerResult<ServiceInfo> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can create services"));
        }

        // 解析端口映射
        let mut ports = std::collections::HashMap::new();
        for port in publish {
            if let Some((host, container)) = port.split_once(":") {
                if let (Ok(host_port), Ok(container_port)) = (host.parse::<u16>(), container.parse::<u16>()) {
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

        // 创建服务
        let service = ServiceInfo {
            id: format!("service-{}", Uuid::new_v4()),
            name,
            status: ServiceStatus::Running,
            image,
            replicas,
            ports,
            environment,
            volumes: vec![], // 实际实现中应该解析 mount 参数
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            version: 1,
            update_config: None,
            history: vec![],
        };

        // 更新集群状态
        let mut state = self.state.write().unwrap();
        state.services.push(service.clone());
        state.updated_at = SystemTime::now();

        // 广播服务创建事件
        self.tx.send(SwarmEvent::ServiceCreate(service.clone())).unwrap();
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();

        Ok(service)
    }

    /// 列出 Swarm 服务
    pub async fn list_services(&self) -> DockerResult<Vec<ServiceInfo>> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        let state = self.state.read().unwrap();
        Ok(state.services.clone())
    }

    /// 查看 Swarm 服务详情
    pub async fn inspect_service(&self, service_id: &str) -> DockerResult<ServiceInfo> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        let state = self.state.read().unwrap();
        if let Some(service) = state.services.iter().find(|s| s.id == service_id || s.name == service_id) {
            Ok(service.clone())
        }
        else {
            Err(DockerError::not_found("service", service_id.to_string()))
        }
    }

    /// 更新 Swarm 服务
    pub async fn update_service(
        &self,
        service_id: &str,
        image: Option<String>,
        replicas: Option<u32>,
        update_config: Option<UpdateConfig>,
    ) -> DockerResult<ServiceInfo> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can update services"));
        }

        let mut state = self.state.write().unwrap();
        let mut updated_service = None;

        // 查找并更新服务
        if let Some(index) = state.services.iter().position(|s| s.id == service_id || s.name == service_id) {
            let mut service = state.services[index].clone();

            // 保存历史版本
            let version_info = ServiceVersionInfo {
                version: service.version,
                image: service.image.clone(),
                replicas: service.replicas,
                updated_at: service.updated_at,
            };
            service.history.push(version_info);

            // 更新服务信息
            if let Some(image) = image {
                service.image = image;
            }
            if let Some(replicas) = replicas {
                service.replicas = replicas;
            }
            if let Some(update_config) = update_config {
                service.update_config = Some(update_config);
            }

            // 执行滚动更新
            service.status = ServiceStatus::Updating;
            service.version += 1;
            service.updated_at = SystemTime::now();

            // 模拟滚动更新过程
            // 实际实现中，这里应该根据 update_config 进行滚动更新
            tokio::time::sleep(Duration::from_secs(2)).await;

            service.status = ServiceStatus::Running;
            state.services[index] = service.clone();
            updated_service = Some(service);
        }
        else {
            return Err(DockerError::not_found("service", service_id.to_string()));
        }

        state.updated_at = SystemTime::now();

        // 广播服务更新事件
        self.tx.send(SwarmEvent::ServiceUpdate(updated_service.clone().unwrap())).unwrap();
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();

        Ok(updated_service.unwrap())
    }

    /// 删除 Swarm 服务
    pub async fn remove_service(&self, service_id: &str) -> DockerResult<()> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can remove services"));
        }

        let mut state = self.state.write().unwrap();
        let mut service_removed = false;
        let mut removed_service_id = String::new();

        // 查找并删除服务
        if let Some(index) = state.services.iter().position(|s| s.id == service_id || s.name == service_id) {
            removed_service_id = state.services[index].id.clone();
            state.services.remove(index);
            service_removed = true;
        }

        if !service_removed {
            return Err(DockerError::not_found("service", service_id.to_string()));
        }

        state.updated_at = SystemTime::now();

        // 广播服务删除事件
        self.tx.send(SwarmEvent::ServiceDelete(removed_service_id)).unwrap();
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();

        Ok(())
    }

    /// 扩缩容 Swarm 服务
    pub async fn scale_service(&self, service_id: &str, replicas: u32) -> DockerResult<ServiceInfo> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can scale services"));
        }

        let mut state = self.state.write().unwrap();
        let mut updated_service = None;

        // 查找并更新服务
        if let Some(index) = state.services.iter().position(|s| s.id == service_id || s.name == service_id) {
            let mut service = state.services[index].clone();
            service.replicas = replicas;
            service.status = ServiceStatus::Updating;
            service.updated_at = SystemTime::now();
            state.services[index] = service.clone();
            updated_service = Some(service);
        }
        else {
            return Err(DockerError::not_found("service", service_id.to_string()));
        }

        state.updated_at = SystemTime::now();

        // 广播服务更新事件
        self.tx.send(SwarmEvent::ServiceUpdate(updated_service.clone().unwrap())).unwrap();
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();

        Ok(updated_service.unwrap())
    }

    /// 回滚 Swarm 服务到之前的版本
    pub async fn rollback_service(&self, service_id: &str) -> DockerResult<ServiceInfo> {
        // 检查是否在 Swarm 集群中
        let in_swarm = *self.in_swarm.lock().unwrap();
        if !in_swarm {
            return Err(DockerError::internal("Node is not in a swarm"));
        }

        // 检查是否是管理节点
        let local_node = self.local_node.lock().unwrap();
        let is_manager = local_node.as_ref().map(|n| n.role == NodeRole::Manager).unwrap_or(false);
        if !is_manager {
            return Err(DockerError::internal("Only managers can rollback services"));
        }

        let mut state = self.state.write().unwrap();
        let mut updated_service = None;

        // 查找并回滚服务
        if let Some(index) = state.services.iter().position(|s| s.id == service_id || s.name == service_id) {
            let mut service = state.services[index].clone();

            // 检查是否有历史版本
            if service.history.is_empty() {
                return Err(DockerError::internal("No history available for rollback"));
            }

            // 获取上一个版本
            let prev_version = service.history.pop().unwrap();

            // 保存当前版本到历史记录
            let current_version = ServiceVersionInfo {
                version: service.version,
                image: service.image.clone(),
                replicas: service.replicas,
                updated_at: service.updated_at,
            };
            service.history.push(current_version);

            // 回滚到上一个版本
            service.image = prev_version.image;
            service.replicas = prev_version.replicas;
            service.status = ServiceStatus::Updating;
            service.version += 1;
            service.updated_at = SystemTime::now();

            // 模拟回滚过程
            // 实际实现中，这里应该根据 update_config 进行滚动更新
            tokio::time::sleep(Duration::from_secs(2)).await;

            service.status = ServiceStatus::Running;
            state.services[index] = service.clone();
            updated_service = Some(service);
        }
        else {
            return Err(DockerError::not_found("service", service_id.to_string()));
        }

        state.updated_at = SystemTime::now();

        // 广播服务更新事件
        self.tx.send(SwarmEvent::ServiceUpdate(updated_service.clone().unwrap())).unwrap();
        self.tx.send(SwarmEvent::ClusterStateChange(state.clone())).unwrap();

        Ok(updated_service.unwrap())
    }

    /// 订阅 Swarm 事件
    pub fn subscribe(&self) -> broadcast::Receiver<SwarmEvent> {
        self.tx.subscribe()
    }

    /// 检查是否在 Swarm 集群中
    pub async fn is_in_swarm(&self) -> bool {
        *self.in_swarm.lock().unwrap()
    }

    /// 获取本地节点信息
    pub async fn get_local_node(&self) -> Option<NodeInfo> {
        self.local_node.lock().unwrap().clone()
    }
}

/// 全局 Swarm 管理器实例
pub static mut SWARM_MANAGER: Option<Arc<SwarmManager>> = None;

/// 初始化 Swarm 管理器
pub fn init_swarm_manager() {
    unsafe {
        if SWARM_MANAGER.is_none() {
            SWARM_MANAGER = Some(Arc::new(SwarmManager::new()));
        }
    }
}

/// 获取 Swarm 管理器
pub fn get_swarm_manager() -> Arc<SwarmManager> {
    unsafe { SWARM_MANAGER.as_ref().unwrap().clone() }
}
