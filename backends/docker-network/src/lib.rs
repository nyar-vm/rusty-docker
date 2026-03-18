#![warn(missing_docs)]


use docker_types::DockerError;
use serde::{Deserialize, Serialize};

/// 结果类型
pub type Result<T> = std::result::Result<T, DockerError>;

/// NetworkConfig 结构体表示网络配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    /// 网络名称
    pub name: String,
    /// 网络驱动
    pub driver: String,
    /// IP 地址管理配置
    pub ipam: Option<IpamConfig>,
    /// 网络选项
    pub options: Option<std::collections::HashMap<String, String>>,
    /// 网络别名
    pub aliases: Option<Vec<String>>,
    /// 网络模式
    pub network_mode: Option<String>,
    /// 是否启用 IPv6
    pub enable_ipv6: bool,
}

/// IpamConfig 结构体表示 IP 地址管理配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpamConfig {
    /// IPAM 驱动
    pub driver: String,
    /// 子网配置列表
    pub config: Vec<IpamSubnetConfig>,
}

/// IpamSubnetConfig 结构体表示子网配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpamSubnetConfig {
    /// 子网 CIDR
    pub subnet: String,
    /// 网关地址
    pub gateway: Option<String>,
    /// IP 范围
    pub ip_range: Option<String>,
}

/// NetworkInfo 结构体表示网络信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    /// 网络 ID
    pub id: String,
    /// 网络名称
    pub name: String,
    /// 网络驱动
    pub driver: String,
    /// 网络作用域
    pub scope: String,
    /// 是否启用 IPv6
    pub enable_ipv6: bool,
    /// 是否内部网络
    pub internal: bool,
    /// 是否可附加
    pub attachable: bool,
    /// 是否入口网络
    pub ingress: bool,
    /// 容器信息映射
    pub containers: std::collections::HashMap<String, ContainerInfo>,
    /// 网络选项
    pub options: std::collections::HashMap<String, String>,
    /// 网络标签
    pub labels: std::collections::HashMap<String, String>,
}

/// ContainerInfo 结构体表示容器网络信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerInfo {
    /// 容器名称
    pub name: String,
    /// 端点 ID
    pub endpoint_id: String,
    /// MAC 地址
    pub mac_address: String,
    /// IPv4 地址
    pub ipv4_address: String,
    /// IPv6 地址
    pub ipv6_address: String,
}

/// NetworkManager trait 定义了网络管理的核心方法
pub trait NetworkManager: Send + Sync {
    /// 创建网络
    ///
    /// # 参数
    /// * `config` - 网络配置
    ///
    /// # 返回值
    /// * `Ok(NetworkInfo)` - 创建成功的网络信息
    /// * `Err(DockerError)` - 创建失败的错误信息
    fn create_network(
        &mut self,
        config: &NetworkConfig,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<NetworkInfo>> + Send + '_>>;

    /// 连接容器到网络
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    /// * `container_id` - 容器 ID
    /// * `aliases` - 网络别名列表
    ///
    /// # 返回值
    /// * `Ok(())` - 连接成功
    /// * `Err(DockerError)` - 连接失败的错误信息
    fn connect_container(
        &mut self,
        network_id: &str,
        container_id: &str,
        aliases: Option<Vec<String>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>;

    /// 断开容器与网络的连接
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    /// * `container_id` - 容器 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 断开连接成功
    /// * `Err(DockerError)` - 断开连接失败的错误信息
    fn disconnect_container(
        &mut self,
        network_id: &str,
        container_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>;

    /// 删除网络
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 删除成功
    /// * `Err(DockerError)` - 删除失败的错误信息
    fn remove_network(
        &mut self,
        network_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>;

    /// 列出所有网络
    ///
    /// # 返回值
    /// * `Ok(Vec<NetworkInfo>)` - 网络列表
    /// * `Err(DockerError)` - 列出失败的错误信息
    fn list_networks(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<NetworkInfo>>> + Send + '_>>;

    /// 查看网络详细信息
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    ///
    /// # 返回值
    /// * `Ok(NetworkInfo)` - 网络详细信息
    /// * `Err(DockerError)` - 查看失败的错误信息
    fn inspect_network(
        &mut self,
        network_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<NetworkInfo>> + Send + '_>>;
}

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
/// 创建 Linux 平台的网络管理器
pub fn new_network_manager() -> Box<dyn NetworkManager> {
    Box::new(linux::LinuxNetworkManager::new())
}

#[cfg(target_os = "windows")]
/// 创建 Windows 平台的网络管理器
pub fn new_network_manager() -> Box<dyn NetworkManager> {
    Box::new(windows::WindowsNetworkManager::new())
}

#[cfg(target_os = "macos")]
/// 创建 macOS 平台的网络管理器
pub fn new_network_manager() -> Box<dyn NetworkManager> {
    Box::new(macos::MacOSNetworkManager::new())
}
