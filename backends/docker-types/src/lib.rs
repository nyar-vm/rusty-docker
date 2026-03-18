#![warn(missing_docs)]

//! Docker 共享数据结构
//!
//! 包含 Docker 相关的所有数据结构定义，供其他组件使用。

pub mod errors;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// 错误类型
pub use crate::errors::*;

/// 向后兼容：RustyDockerError 类型别名
#[deprecated(note = "请使用 DockerError 替代")]
pub type RustyDockerError = DockerError;

/// 结果类型别名
pub type Result<T> = std::result::Result<T, DockerError>;

/// 容器状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerStatus {
    /// 创建中
    Creating,
    /// 运行中
    Running,
    /// 已停止
    Stopped,
    /// 已暂停
    Paused,
    /// 错误
    Error(String),
}

/// 资源限制
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU 限制（核心数）
    pub cpu_limit: f64,
    /// 内存限制（MB）
    pub memory_limit: u32,
    /// 存储限制（GB）
    pub storage_limit: u32,
    /// 网络带宽限制（MB/s）
    pub network_limit: u32,
}

/// 重启策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// 无重启策略
    None,
    /// 总是重启
    Always,
    /// 失败时重启
    OnFailure(u32),
    /// 除非被手动停止，否则总是重启
    UnlessStopped,
}

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// 健康检查命令
    pub test: Vec<String>,
    /// 健康检查间隔时间（秒）
    pub interval: u32,
    /// 健康检查超时时间（秒）
    pub timeout: u32,
    /// 健康检查失败重试次数
    pub retries: u32,
    /// 健康检查启动超时时间（秒）
    pub start_period: u32,
}

/// 部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    /// 复制数量
    pub replicas: u32,
    /// 资源限制
    pub resources: ResourceLimits,
    /// 重启策略
    pub restart_policy: RestartPolicy,
    /// 滚动更新配置
    pub update_config: Option<UpdateConfig>,
}

/// 更新配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// 并行更新数量
    pub parallelism: u32,
    /// 失败后是否继续
    pub failure_action: String,
    /// 监控更新的时间（秒）
    pub monitor: u32,
    /// 最大失败比例
    pub max_failure_ratio: f64,
    /// 排序方法
    pub order: String,
}

/// 容器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    /// 容器名称
    pub name: String,
    /// 镜像名称
    pub image: String,
    /// 命令
    pub command: Vec<String>,
    /// 环境变量
    pub environment: HashMap<String, String>,
    /// 端口映射
    pub ports: HashMap<u16, u16>,
    /// 挂载卷
    pub volumes: Vec<VolumeMount>,
    /// 资源限制
    pub resources: ResourceLimits,
    /// 网络配置
    pub network: NetworkConfig,
    /// 重启策略
    pub restart_policy: Option<RestartPolicy>,
    /// 健康检查配置
    pub healthcheck: Option<HealthCheck>,
    /// 部署配置
    pub deploy: Option<DeployConfig>,
}

/// 挂载类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MountType {
    /// 绑定挂载
    Bind,
    /// 卷
    Volume,
    /// tmpfs
    Tmpfs,
}

/// 绑定挂载一致性选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Consistency {
    /// 默认一致性
    Default,
    /// 一致
    Consistent,
    /// 委托
    Delegated,
    /// 缓存
    Cached,
}

/// 卷挂载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// 挂载类型
    pub mount_type: MountType,
    /// 主机路径（对于绑定挂载）
    pub host_path: Option<String>,
    /// 卷名称（对于卷挂载）
    pub volume_name: Option<String>,
    /// 容器路径
    pub container_path: String,
    /// 读写模式
    pub read_only: bool,
    /// 驱动名称（对于卷挂载）
    pub driver: Option<String>,
    /// 卷标签（对于卷挂载）
    pub labels: Option<std::collections::HashMap<String, String>>,
    /// 绑定挂载一致性选项
    pub consistency: Option<Consistency>,
    /// tmpfs 大小（对于 tmpfs 挂载）
    pub tmpfs_size: Option<u64>,
    /// tmpfs 模式（对于 tmpfs 挂载）
    pub tmpfs_mode: Option<u32>,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 网络名称
    pub network_name: String,
    /// 静态 IP
    pub static_ip: Option<String>,
    /// 主机名
    pub hostname: Option<String>,
    /// 网络别名
    pub aliases: Option<Vec<String>>,
    /// 网络模式
    pub network_mode: Option<String>,
    /// 是否启用 IPv6
    pub enable_ipv6: bool,
}

/// 容器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    /// 容器 ID
    pub id: String,
    /// 容器名称
    pub name: String,
    /// 镜像名称
    pub image: String,
    /// 状态
    pub status: ContainerStatus,
    /// 配置
    pub config: ContainerConfig,
    /// 创建时间
    pub created_at: SystemTime,
    /// 启动时间
    pub started_at: Option<SystemTime>,
    /// 停止时间
    pub stopped_at: Option<SystemTime>,
    /// 进程 ID
    pub pid: Option<u32>,
    /// 网络信息
    pub network_info: NetworkInfo,
}

/// 网络信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// IP 地址
    pub ip_address: Option<String>,
    /// 端口映射
    pub ports: HashMap<u16, u16>,
    /// 网络名称
    pub network_name: String,
}

/// 镜像信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// 镜像 ID
    pub id: String,
    /// 镜像名称
    pub name: String,
    /// 标签
    pub tags: Vec<String>,
    /// 大小
    pub size: u64,
    /// 创建时间
    pub created_at: SystemTime,
    /// 架构
    pub architecture: String,
    /// 操作系统
    pub os: String,
}

/// 网络配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfigInfo {
    /// 网络名称
    pub name: String,
    /// 网络类型
    pub network_type: String,
    /// 子网
    pub subnet: String,
    /// 网关
    pub gateway: String,
    /// 容器列表
    pub containers: Vec<String>,
}

/// 全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    /// 数据目录
    pub data_dir: String,
    /// 镜像存储目录
    pub image_dir: String,
    /// 容器存储目录
    pub container_dir: String,
    /// 网络配置目录
    pub network_dir: String,
    /// 默认网络名称
    pub default_network: String,
    /// 默认资源限制
    pub default_resources: ResourceLimits,
    /// 日志配置
    pub log_config: LogConfig,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志级别
    pub log_level: String,
    /// 日志文件路径
    pub log_file: String,
    /// 日志大小限制（MB）
    pub max_log_size: u32,
}

/// 卷信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    /// 卷 ID
    pub id: String,
    /// 卷名称
    pub name: String,
    /// 卷大小（字节）
    pub size: u64,
    /// 创建时间
    pub created_at: SystemTime,
    /// 挂载点
    pub mount_point: String,
    /// 驱动
    pub driver: String,
    /// 标签
    pub labels: HashMap<String, String>,
    /// 被使用的容器
    pub used_by: Vec<String>,
}

/// 系统资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceUsage {
    /// CPU 使用率（百分比）
    pub cpu_usage: f64,
    /// 内存使用量（MB）
    pub memory_used: u32,
    /// 内存总量（MB）
    pub memory_total: u32,
    /// 存储使用量（GB）
    pub storage_used: u32,
    /// 存储总量（GB）
    pub storage_total: u32,
    /// 网络发送量（MB）
    pub network_sent: u32,
    /// 网络接收量（MB）
    pub network_received: u32,
}

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 操作系统类型
    pub os_type: String,
    /// 操作系统版本
    pub os_version: String,
    /// 内核版本
    pub kernel_version: String,
    /// 架构
    pub architecture: String,
    /// 主机名
    pub hostname: String,
    /// 处理器核心数
    pub cpu_cores: u32,
    /// 总内存（MB）
    pub total_memory: u32,
}

/// Docker 守护进程状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockerDaemonStatus {
    /// 运行中
    Running,
    /// 停止
    Stopped,
    /// 错误
    Error(String),
}

/// 系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Docker 守护进程状态
    pub daemon_status: DockerDaemonStatus,
    /// 系统资源使用情况
    pub resource_usage: SystemResourceUsage,
    /// 系统信息
    pub system_info: SystemInfo,
    /// 容器统计信息
    pub container_stats: ContainerStats,
}

/// 容器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    /// 运行中的容器数
    pub running: u32,
    /// 已停止的容器数
    pub stopped: u32,
    /// 总容器数
    pub total: u32,
}

/// Swarm 服务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// 已创建
    Created,
    /// 运行中
    Running,
    /// 更新中
    Updating,
    /// 错误
    Error(String),
}

/// Swarm 服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// 服务 ID
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 服务状态
    pub status: ServiceStatus,
    /// 镜像名称
    pub image: String,
    /// 副本数
    pub replicas: u32,
    /// 端口映射
    pub ports: HashMap<u16, u16>,
    /// 环境变量
    pub environment: HashMap<String, String>,
    /// 挂载卷
    pub volumes: Vec<VolumeMount>,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: SystemTime,
}

/// Swarm 节点角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeRole {
    /// 管理节点
    Manager,
    /// 工作节点
    Worker,
}

/// Swarm 节点可用性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeAvailability {
    /// 活跃
    Active,
    /// 暂停
    Pause,
    /// 排空
    Drain,
}

/// Swarm 节点状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    /// 就绪
    Ready,
    /// 不可用
    Down,
    /// 未知
    Unknown,
}

/// Swarm 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// 节点 ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 节点角色
    pub role: NodeRole,
    /// 节点可用性
    pub availability: NodeAvailability,
    /// 节点状态
    pub status: NodeStatus,
    /// 节点地址
    pub address: String,
    /// 节点版本
    pub version: String,
    /// 运行的容器数
    pub containers_running: u32,
    /// 节点标签
    pub labels: HashMap<String, String>,
}

/// Swarm 集群信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmInfo {
    /// 集群 ID
    pub id: String,
    /// 集群名称
    pub name: Option<String>,
    /// 管理节点数量
    pub managers: u32,
    /// 工作节点数量
    pub workers: u32,
    /// 服务数量
    pub services: u32,
    /// 任务数量
    pub tasks: u32,
    /// 集群版本
    pub version: String,
    /// 集群创建时间
    pub created_at: SystemTime,
}

/// 配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInfo {
    /// 配置 ID
    pub id: String,
    /// 配置名称
    pub name: String,
    /// 配置数据
    pub data: String,
    /// 创建时间
    pub created_at: SystemTime,
    /// 标签
    pub labels: HashMap<String, String>,
}

/// 密钥信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretInfo {
    /// 密钥 ID
    pub id: String,
    /// 密钥名称
    pub name: String,
    /// 创建时间
    pub created_at: SystemTime,
    /// 标签
    pub labels: HashMap<String, String>,
    /// 密钥摘要（用于验证密钥完整性）
    pub digest: String,
}

/// 端点类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointType {
    /// 本地 Docker 环境
    Local,
    /// 远程 Docker 环境
    Remote,
    /// 云 Docker 环境
    Cloud,
}

/// 端点状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointStatus {
    /// 已连接
    Connected,
    /// 连接中
    Connecting,
    /// 连接失败
    Failed(String),
    /// 未连接
    Disconnected,
}

/// 端点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    /// 端点 ID
    pub id: String,
    /// 端点名称
    pub name: String,
    /// 端点类型
    pub endpoint_type: EndpointType,
    /// 端点 URL
    pub url: String,
    /// 是否使用 TLS
    pub use_tls: bool,
    /// TLS 证书路径（可选）
    pub tls_cert_path: Option<String>,
    /// TLS 密钥路径（可选）
    pub tls_key_path: Option<String>,
    /// TLS CA 证书路径（可选）
    pub tls_ca_path: Option<String>,
    /// 认证令牌（可选）
    pub auth_token: Option<String>,
    /// 标签
    pub labels: HashMap<String, String>,
}

/// 端点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// 端点配置
    pub config: EndpointConfig,
    /// 端点状态
    pub status: EndpointStatus,
    /// 创建时间
    pub created_at: SystemTime,
    /// 最后连接时间
    pub last_connected_at: Option<SystemTime>,
    /// 连接信息
    pub connection_info: Option<SystemInfo>,
}
