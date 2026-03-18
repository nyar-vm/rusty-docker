#![warn(missing_docs)]

//! Kubernetes 数据模型定义

/// 服务类型
#[derive(Clone)]
pub enum ServiceType {
    /// 集群内部访问
    ClusterIP,
    /// 节点端口访问
    NodePort,
    /// 负载均衡器访问
    LoadBalancer,
    /// 外部名称访问
    ExternalName,
}

/// 服务端口
#[derive(Clone)]
pub struct ServicePort {
    /// 端口名称
    pub name: String,
    /// 端口号
    pub port: u16,
    /// 目标端口
    pub target_port: u16,
    /// 协议
    pub protocol: String,
}

/// 部署信息
#[derive(Clone)]
pub struct DeploymentInfo {
    /// 部署名称
    pub name: String,
    /// 命名空间
    pub namespace: String,
    /// 副本数
    pub replicas: u32,
    /// 可用副本数
    pub available_replicas: u32,
    /// 镜像
    pub image: String,
    /// 创建时间
    pub created_at: std::time::SystemTime,
    /// 更新时间
    pub updated_at: std::time::SystemTime,
}

/// 服务信息
#[derive(Clone)]
pub struct ServiceInfo {
    /// 服务名称
    pub name: String,
    /// 命名空间
    pub namespace: String,
    /// 服务类型
    pub service_type: ServiceType,
    /// 集群 IP
    pub cluster_ip: String,
    /// 端口
    pub ports: Vec<ServicePort>,
    /// 选择器
    pub selector: std::collections::HashMap<String, String>,
    /// 创建时间
    pub created_at: std::time::SystemTime,
}

/// 配置映射信息
#[derive(Clone)]
pub struct ConfigMapInfo {
    /// 配置映射名称
    pub name: String,
    /// 命名空间
    pub namespace: String,
    /// 数据
    pub data: std::collections::HashMap<String, String>,
    /// 创建时间
    pub created_at: std::time::SystemTime,
}

/// 秘密信息
#[derive(Clone)]
pub struct SecretInfo {
    /// 秘密名称
    pub name: String,
    /// 命名空间
    pub namespace: String,
    /// 类型
    pub secret_type: String,
    /// 创建时间
    pub created_at: std::time::SystemTime,
}

/// 节点信息
pub struct NodeInfo {
    /// 节点名称
    pub name: String,
    /// 状态
    pub status: String,
    /// 角色
    pub role: String,
    /// IP 地址
    pub ip: String,
    /// 操作系统
    pub os: String,
    /// Kubernetes 版本
    pub kubelet_version: String,
    /// 容器运行时
    pub container_runtime: String,
    /// 创建时间
    pub created_at: std::time::SystemTime,
}

/// 集群信息
pub struct ClusterInfo {
    /// 集群名称
    pub name: String,
    /// Kubernetes 版本
    pub kubernetes_version: String,
    /// 节点数量
    pub node_count: u32,
    /// 控制平面节点数量
    pub control_plane_count: u32,
    /// 工作节点数量
    pub worker_count: u32,
    /// Pod 数量
    pub pod_count: u32,
    /// 服务数量
    pub service_count: u32,
    /// 部署数量
    pub deployment_count: u32,
    /// 创建时间
    pub created_at: std::time::SystemTime,
}
