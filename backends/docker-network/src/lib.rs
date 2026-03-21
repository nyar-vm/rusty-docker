#![warn(missing_docs)]

use docker_types::{DockerError, DockerErrorKind};
use rand;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

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

/// 端口映射配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortMapping {
    /// 主机端口
    pub host_port: u16,
    /// 容器端口
    pub container_port: u16,
    /// 主机地址
    pub host_address: Option<String>,
}

/// NetworkDriver trait 定义了网络驱动的核心方法
#[async_trait]
pub trait NetworkDriver: Send + Sync {
    /// 创建网络
    ///
    /// # 参数
    /// * `config` - 网络配置
    ///
    /// # 返回值
    /// * `Ok(NetworkInfo)` - 创建成功的网络信息
    /// * `Err(DockerError)` - 创建失败的错误信息
    async fn create_network(
        &self,
        config: &NetworkConfig,
    ) -> Result<NetworkInfo>;

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
    async fn connect_container(
        &self,
        network_id: &str,
        container_id: &str,
        aliases: Option<Vec<String>>,
    ) -> Result<()>;

    /// 断开容器与网络的连接
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    /// * `container_id` - 容器 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 断开连接成功
    /// * `Err(DockerError)` - 断开连接失败的错误信息
    async fn disconnect_container(
        &self,
        network_id: &str,
        container_id: &str,
    ) -> Result<()>;

    /// 删除网络
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 删除成功
    /// * `Err(DockerError)` - 删除失败的错误信息
    async fn remove_network(
        &self,
        network_id: &str,
    ) -> Result<()>;

    /// 查看网络详细信息
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    ///
    /// # 返回值
    /// * `Ok(NetworkInfo)` - 网络详细信息
    /// * `Err(DockerError)` - 查看失败的错误信息
    async fn inspect_network(
        &self,
        network_id: &str,
    ) -> Result<NetworkInfo>;

    /// 添加端口映射
    ///
    /// # 参数
    /// * `container_id` - 容器 ID
    /// * `container_ip` - 容器 IP 地址
    /// * `port_mapping` - 端口映射配置
    ///
    /// # 返回值
    /// * `Ok(())` - 添加成功
    /// * `Err(DockerError)` - 添加失败的错误信息
    async fn add_port_mapping(
        &self,
        container_id: &str,
        container_ip: &str,
        port_mapping: &PortMapping,
    ) -> Result<()>;

    /// 删除端口映射
    ///
    /// # 参数
    /// * `port_mapping` - 端口映射配置
    ///
    /// # 返回值
    /// * `Ok(())` - 删除成功
    /// * `Err(DockerError)` - 删除失败的错误信息
    async fn remove_port_mapping(
        &self,
        port_mapping: &PortMapping,
    ) -> Result<()>;
}

/// NetworkManager trait 定义了网络管理的核心方法
#[async_trait]
pub trait NetworkManager: Send + Sync {
    /// 创建网络
    ///
    /// # 参数
    /// * `config` - 网络配置
    ///
    /// # 返回值
    /// * `Ok(NetworkInfo)` - 创建成功的网络信息
    /// * `Err(DockerError)` - 创建失败的错误信息
    async fn create_network(
        &self,
        config: &NetworkConfig,
    ) -> Result<NetworkInfo>;

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
    async fn connect_container(
        &self,
        network_id: &str,
        container_id: &str,
        aliases: Option<Vec<String>>,
    ) -> Result<()>;

    /// 断开容器与网络的连接
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    /// * `container_id` - 容器 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 断开连接成功
    /// * `Err(DockerError)` - 断开连接失败的错误信息
    async fn disconnect_container(
        &self,
        network_id: &str,
        container_id: &str,
    ) -> Result<()>;

    /// 删除网络
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 删除成功
    /// * `Err(DockerError)` - 删除失败的错误信息
    async fn remove_network(
        &self,
        network_id: &str,
    ) -> Result<()>;

    /// 列出所有网络
    ///
    /// # 返回值
    /// * `Ok(Vec<NetworkInfo>)` - 网络列表
    /// * `Err(DockerError)` - 列出失败的错误信息
    async fn list_networks(&self) -> Result<Vec<NetworkInfo>>;

    /// 查看网络详细信息
    ///
    /// # 参数
    /// * `network_id` - 网络 ID
    ///
    /// # 返回值
    /// * `Ok(NetworkInfo)` - 网络详细信息
    /// * `Err(DockerError)` - 查看失败的错误信息
    async fn inspect_network(
        &self,
        network_id: &str,
    ) -> Result<NetworkInfo>;

    /// 注册网络驱动
    ///
    /// # 参数
    /// * `name` - 驱动名称
    /// * `driver` - 网络驱动实例
    ///
    /// # 返回值
    /// * `Ok(())` - 注册成功
    /// * `Err(DockerError)` - 注册失败的错误信息
    fn register_driver(
        &self,
        name: &str,
        driver: Box<dyn NetworkDriver>,
    ) -> Result<()>;

    /// 获取网络驱动
    ///
    /// # 参数
    /// * `name` - 驱动名称
    ///
    /// # 返回值
    /// * `Some(Arc<Mutex<Box<dyn NetworkDriver>>>)` - 网络驱动实例
    /// * `None` - 驱动不存在
    fn get_driver(&self, name: &str) -> Option<Arc<Mutex<Box<dyn NetworkDriver>>>>;

    /// 添加端口映射
    ///
    /// # 参数
    /// * `container_id` - 容器 ID
    /// * `container_ip` - 容器 IP 地址
    /// * `port_mapping` - 端口映射配置
    ///
    /// # 返回值
    /// * `Ok(())` - 添加成功
    /// * `Err(DockerError)` - 添加失败的错误信息
    async fn add_port_mapping(
        &self,
        container_id: &str,
        container_ip: &str,
        port_mapping: &PortMapping,
    ) -> Result<()>;

    /// 删除端口映射
    ///
    /// # 参数
    /// * `port_mapping` - 端口映射配置
    ///
    /// # 返回值
    /// * `Ok(())` - 删除成功
    /// * `Err(DockerError)` - 删除失败的错误信息
    async fn remove_port_mapping(
        &self,
        port_mapping: &PortMapping,
    ) -> Result<()>;
}

use std::sync::Arc;
use tokio::sync::Mutex;

use std::process::Command;

/// IP 地址管理
#[derive(Debug)]
pub struct IpManager {
    subnet: String,
    gateway: String,
    used_ips: std::collections::HashSet<String>,
}

impl IpManager {
    /// 创建新的 IP 管理器
    pub fn new(subnet: &str, gateway: &str) -> Self {
        let mut used_ips = std::collections::HashSet::new();
        used_ips.insert(gateway.to_string());
        Self {
            subnet: subnet.to_string(),
            gateway: gateway.to_string(),
            used_ips,
        }
    }

    /// 分配 IP 地址
    pub fn allocate_ip(&mut self) -> Result<String> {
        // 简单的 IP 分配算法
        // 实际实现中应该解析子网并分配可用 IP
        for i in 2..255 {
            let ip = format!("172.17.0.{}", i);
            if !self.used_ips.contains(&ip) {
                self.used_ips.insert(ip.clone());
                return Ok(ip);
            }
        }
        Err(DockerError::new(DockerErrorKind::NetworkError { reason: "No available IP addresses".to_string() }))
    }

    /// 释放 IP 地址
    pub fn release_ip(&mut self, ip: &str) {
        self.used_ips.remove(ip);
    }
}

/// BridgeNetworkDriver 实现了 bridge 网络驱动
pub struct BridgeNetworkDriver {
    networks: Arc<Mutex<std::collections::HashMap<String, (NetworkInfo, Box<IpManager>)>>>,
}

impl BridgeNetworkDriver {
    /// 创建新的 bridge 网络驱动
    pub fn new() -> Self {
        Self {
            networks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// 执行系统命令
    fn execute_command(cmd: &str) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .map_err(|e| DockerError::new(DockerErrorKind::NetworkError { reason: format!("Failed to execute command: {}", e) }))?;

            if !output.status.success() {
                let error_str = String::from_utf8_lossy(&output.stderr);
                return Err(DockerError::new(DockerErrorKind::NetworkError { reason: format!("Command failed: {}", error_str) }));
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("cmd")
                .arg("/c")
                .arg(cmd)
                .output()
                .map_err(|e| DockerError::new(DockerErrorKind::NetworkError { reason: format!("Failed to execute command: {}", e) }))?;

            if !output.status.success() {
                let error_str = String::from_utf8_lossy(&output.stderr);
                return Err(DockerError::new(DockerErrorKind::NetworkError { reason: format!("Command failed: {}", error_str) }));
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .map_err(|e| DockerError::new(DockerErrorKind::NetworkError { reason: format!("Failed to execute command: {}", e) }))?;

            if !output.status.success() {
                let error_str = String::from_utf8_lossy(&output.stderr);
                return Err(DockerError::new(DockerErrorKind::NetworkError { reason: format!("Command failed: {}", error_str) }));
            }
        }

        Ok(())
    }

    /// 创建网桥
    fn create_bridge(bridge_name: &str) -> Result<()> {
        // 检查网桥是否存在
        let check_cmd = format!("ip link show {} || true", bridge_name);
        let output = Command::new("sh")
            .arg("-c")
            .arg(&check_cmd)
            .output()
            .map_err(|e| DockerError::new(DockerErrorKind::NetworkError { reason: format!("Failed to check bridge: {}", e) }))?;

        if String::from_utf8_lossy(&output.stdout).contains(bridge_name) {
            return Ok(());
        }

        // 创建网桥
        let create_cmd = format!("ip link add {} type bridge", bridge_name);
        Self::execute_command(&create_cmd)?;

        // 启动网桥
        let up_cmd = format!("ip link set {} up", bridge_name);
        Self::execute_command(&up_cmd)?;

        Ok(())
    }

    /// 删除网桥
    fn delete_bridge(bridge_name: &str) -> Result<()> {
        let cmd = format!("ip link delete {}", bridge_name);
        Self::execute_command(&cmd)?;
        Ok(())
    }

    /// 配置网桥 IP
    fn configure_bridge_ip(bridge_name: &str, ip: &str) -> Result<()> {
        let cmd = format!("ip addr add {}/24 dev {}", ip, bridge_name);
        Self::execute_command(&cmd)?;
        Ok(())
    }

    /// 创建 veth 对
    fn create_veth_pair(veth1: &str, veth2: &str) -> Result<()> {
        let cmd = format!("ip link add {} type veth peer name {}", veth1, veth2);
        Self::execute_command(&cmd)?;
        Ok(())
    }

    /// 将 veth 连接到网桥
    fn connect_veth_to_bridge(veth: &str, bridge: &str) -> Result<()> {
        let up_cmd = format!("ip link set {} up", veth);
        Self::execute_command(&up_cmd)?;

        let add_cmd = format!("ip link set {} master {}", veth, bridge);
        Self::execute_command(&add_cmd)?;

        Ok(())
    }

    /// 配置容器网络命名空间
    fn configure_container_network(veth: &str, container_id: &str, ip: &str) -> Result<()> {
        let netns_path = format!("/var/run/netns/{}", container_id);
        
        // 创建网络命名空间
        let create_ns_cmd = format!("mkdir -p /var/run/netns && touch {}", netns_path);
        Self::execute_command(&create_ns_cmd)?;

        // 将 veth 移到容器命名空间
        let move_cmd = format!("ip link set {} netns {}", veth, container_id);
        Self::execute_command(&move_cmd)?;

        // 配置容器内的网络
        let config_cmds = format!(
            "ip netns exec {} ip link set {} up && ip netns exec {} ip addr add {}/24 dev {} && ip netns exec {} ip route add default via 172.17.0.1",
            container_id, veth, container_id, ip, veth, container_id
        );
        Self::execute_command(&config_cmds)?;

        Ok(())
    }
}

#[async_trait]
impl NetworkDriver for BridgeNetworkDriver {
    async fn create_network(
        &self,
        config: &NetworkConfig,
    ) -> Result<NetworkInfo> {
        let config = config.clone();
        let networks = self.networks.clone();
        
        // Generate a random network ID
        let network_id = format!("{:x}", rand::random::<u64>());

        // Create bridge name
        let bridge_name = format!("br-{}", &network_id[0..8]);

        // Create bridge
        BridgeNetworkDriver::create_bridge(&bridge_name)?;

        // Configure bridge IP
        let gateway = config.ipam.as_ref().and_then(|ipam| ipam.config.first().and_then(|c| c.gateway.as_deref())).unwrap_or("172.17.0.1");
        BridgeNetworkDriver::configure_bridge_ip(&bridge_name, gateway)?;

        // Create IP manager
        let subnet = config.ipam.as_ref().and_then(|ipam| ipam.config.first().map(|c| c.subnet.clone())).unwrap_or("172.17.0.0/24".to_string());
        let ip_manager = Box::new(IpManager::new(&subnet, gateway));

        // Create network info
        let network_info = NetworkInfo {
            id: network_id.clone(),
            name: config.name,
            driver: config.driver,
            scope: "local".to_string(),
            enable_ipv6: config.enable_ipv6,
            internal: false,
            attachable: true,
            ingress: false,
            containers: std::collections::HashMap::new(),
            options: config.options.unwrap_or_default(),
            labels: std::collections::HashMap::new(),
        };

        // Store network info
        networks.lock().await.insert(network_id, (network_info.clone(), ip_manager));

        Ok(network_info)
    }

    async fn connect_container(
        &self,
        network_id: &str,
        container_id: &str,
        _aliases: Option<Vec<String>>,
    ) -> Result<()> {
        let network_id = network_id.to_string();
        let container_id = container_id.to_string();
        let networks = self.networks.clone();
        
        let mut networks = networks.lock().await;

        if let Some((network, ip_manager)) = networks.get_mut(&network_id) {
            // Allocate IP address
            let ip = ip_manager.allocate_ip()?;

            // Create veth pair
            let veth_host = format!("veth-{}-{}  ", &network_id[0..6], &container_id[0..6]);
            let veth_container = format!("eth0");
            BridgeNetworkDriver::create_veth_pair(&veth_host, &veth_container)?;

            // Connect veth to bridge
            let bridge_name = format!("br-{}", &network_id[0..8]);
            BridgeNetworkDriver::connect_veth_to_bridge(&veth_host, &bridge_name)?;

            // Configure container network
            BridgeNetworkDriver::configure_container_network(&veth_container, &container_id, &ip)?;

            // Add container to network
            let container_info = ContainerInfo {
                name: container_id.clone(),
                endpoint_id: format!("{:x}", rand::random::<u64>()),
                mac_address: format!(
                    "02:42:{:02x}:{:02x}:{:02x}:{:02x}",
                    rand::random::<u8>(),
                    rand::random::<u8>(),
                    rand::random::<u8>(),
                    rand::random::<u8>()
                ),
                ipv4_address: ip,
                ipv6_address: "".to_string(),
            };

            network.containers.insert(container_id, container_info);
            Ok(())
        }
        else {
            Err(DockerError::not_found("network", network_id))
        }
    }

    async fn disconnect_container(
        &self,
        network_id: &str,
        container_id: &str,
    ) -> Result<()> {
        let network_id = network_id.to_string();
        let container_id = container_id.to_string();
        let networks = self.networks.clone();
        
        let mut networks = networks.lock().await;

        if let Some((network, ip_manager)) = networks.get_mut(&network_id) {
            // Get container info
            if let Some(container_info) = network.containers.get(&container_id) {
                // Release IP address
                ip_manager.release_ip(&container_info.ipv4_address);
            }

            // Remove container from network
            network.containers.remove(&container_id);
            Ok(())
        }
        else {
            Err(DockerError::not_found("network", network_id))
        }
    }

    async fn remove_network(
        &self,
        network_id: &str,
    ) -> Result<()> {
        let network_id = network_id.to_string();
        let networks = self.networks.clone();
        
        let mut networks = networks.lock().await;

        if let Some((_, _)) = networks.remove(&network_id) {
            // Delete bridge
            let bridge_name = format!("br-{}", &network_id[0..8]);
            BridgeNetworkDriver::delete_bridge(&bridge_name)?;
            Ok(())
        } else {
            Err(DockerError::not_found("network", network_id))
        }
    }

    async fn inspect_network(
        &self,
        network_id: &str,
    ) -> Result<NetworkInfo> {
        let network_id = network_id.to_string();
        let networks = self.networks.clone();
        
        let networks = networks.lock().await;

        if let Some((network, _)) = networks.get(&network_id) {
            Ok(network.clone())
        }
        else {
            Err(DockerError::not_found("network", network_id))
        }
    }

    async fn add_port_mapping(
        &self,
        _container_id: &str,
        container_ip: &str,
        port_mapping: &PortMapping,
    ) -> Result<()> {
        let host_address = port_mapping.host_address.as_deref().unwrap_or("0.0.0.0");
        
        #[cfg(target_os = "linux")]
        {
            // Add iptables rule for port forwarding
            let cmd = format!(
                "iptables -t nat -A PREROUTING -p tcp -d {} --dport {} -j DNAT --to-destination {}:{}",
                host_address,
                port_mapping.host_port,
                container_ip,
                port_mapping.container_port
            );
            BridgeNetworkDriver::execute_command(&cmd)?;
            
            // Add iptables rule for masquerading
            let cmd = format!(
                "iptables -t nat -A POSTROUTING -p tcp -s {} --sport {} -j MASQUERADE",
                container_ip,
                port_mapping.container_port
            );
            BridgeNetworkDriver::execute_command(&cmd)?;
        }
        
        #[cfg(target_os = "windows")]
        {
            // Add netsh portproxy rule
            let cmd = format!(
                "netsh interface portproxy add v4tov4 listenaddress={} listenport={} connectaddress={} connectport={}",
                host_address,
                port_mapping.host_port,
                container_ip,
                port_mapping.container_port
            );
            BridgeNetworkDriver::execute_command(&cmd)?;
        }
        
        #[cfg(target_os = "macos")]
        {
            // Add pfctl rule
            // Note: This is a simplified implementation
            // In a real implementation, you would need to manage the pf configuration file
            let cmd = format!(
                "echo 'rdr pass inet proto tcp from any to any port {} -> {} port {}' | sudo pfctl -ef -",
                port_mapping.host_port,
                container_ip,
                port_mapping.container_port
            );
            BridgeNetworkDriver::execute_command(&cmd)?;
        }
        
        Ok(())
    }

    async fn remove_port_mapping(
        &self,
        port_mapping: &PortMapping,
    ) -> Result<()> {
        let host_address = port_mapping.host_address.as_deref().unwrap_or("0.0.0.0");
        
        #[cfg(target_os = "linux")]
        {
            // Remove iptables rule for port forwarding (使用更通用的规则)
            let cmd = format!(
                "iptables -t nat -D PREROUTING -p tcp --dport {} -j DNAT",
                port_mapping.host_port
            );
            // 忽略错误，因为规则可能不存在
            let _ = BridgeNetworkDriver::execute_command(&cmd);
            
            // Remove iptables rule for masquerading (使用更通用的规则)
            let cmd = format!(
                "iptables -t nat -D POSTROUTING -p tcp --sport {} -j MASQUERADE",
                port_mapping.container_port
            );
            // 忽略错误，因为规则可能不存在
            let _ = BridgeNetworkDriver::execute_command(&cmd);
        }
        
        #[cfg(target_os = "windows")]
        {
            // Remove netsh portproxy rule
            let cmd = format!(
                "netsh interface portproxy delete v4tov4 listenaddress={} listenport={}",
                host_address,
                port_mapping.host_port
            );
            // 忽略错误，因为规则可能不存在
            let _ = BridgeNetworkDriver::execute_command(&cmd);
        }
        
        #[cfg(target_os = "macos")]
        {
            // Remove pfctl rule
            // Note: This is a simplified implementation
            // In a real implementation, you would need to manage the pf configuration file
            let cmd = "sudo pfctl -F all -f /etc/pf.conf";
            // 忽略错误，因为规则可能不存在
            let _ = BridgeNetworkDriver::execute_command(&cmd);
        }
        
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

mod config;

pub use config::NetworkConfigManager;

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

/// 创建网络配置管理器
///
/// # 参数
/// * `config_path` - 配置文件路径
///
/// # 返回值
/// * `NetworkConfigManager` - 网络配置管理器实例
pub fn new_config_manager(config_path: &str) -> NetworkConfigManager {
    NetworkConfigManager::new(config_path)
}
