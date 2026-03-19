use std::{process::Command, str};

/// 网络管理器
#[derive(Debug)]
pub struct NetworkManager {
    /// 网络名称
    pub network_name: String,
}

impl NetworkManager {
    /// 创建新的网络管理器
    pub fn new(network_name: &str) -> Self {
        Self { network_name: network_name.to_string() }
    }

    /// 创建网络
    pub fn create_network(&self) -> Result<(), String> {
        // 检查网络是否已存在
        let check_output = Command::new("powershell")
            .args(&["-Command", &format!("Get-NetAdapter | Where-Object {{ $_.Name -eq '{}' }}", self.network_name)])
            .output()
            .expect("Failed to execute PowerShell command");

        let check_output_str = str::from_utf8(&check_output.stdout).unwrap();
        if !check_output_str.is_empty() {
            return Ok(());
        }

        // 创建网络
        let output = Command::new("powershell")
            .args(&["-Command", &format!("New-VMSwitch -Name '{}' -SwitchType Internal", self.network_name)])
            .output()
            .expect("Failed to execute PowerShell command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to create network: {}", error_str));
        }

        Ok(())
    }

    /// 配置网络 IP
    pub fn configure_network(&self, ip_address: &str, subnet_mask: &str) -> Result<(), String> {
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "New-NetIPAddress -InterfaceAlias '{}' -IPAddress {} -PrefixLength {}",
                    self.network_name, ip_address, subnet_mask
                ),
            ])
            .output()
            .expect("Failed to execute PowerShell command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to configure network: {}", error_str));
        }

        Ok(())
    }

    /// 在 WSL 2 中配置网络
    pub fn configure_wsl_network(&self, wsl_distro: &str) -> Result<(), String> {
        // 在 WSL 中配置网络
        let command = format!("sudo ip addr add 192.168.50.2/24 dev eth0 && sudo ip route add default via 192.168.50.1");
        let output = Command::new("wsl")
            .args(&["-d", wsl_distro, "bash", "-c", &command])
            .output()
            .expect("Failed to execute wsl command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to configure WSL network: {}", error_str));
        }

        Ok(())
    }

    /// 配置端口映射
    pub fn configure_port_mapping(&self, host_port: u16, container_port: u16) -> Result<(), String> {
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "netsh interface portproxy add v4tov4 listenport={} listenaddress=0.0.0.0 connectport={} connectaddress=192.168.50.2",
                    host_port,
                    container_port
                ),
            ])
            .output()
            .expect("Failed to execute PowerShell command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to configure port mapping: {}", error_str));
        }

        Ok(())
    }

    /// 删除端口映射
    pub fn remove_port_mapping(&self, host_port: u16) -> Result<(), String> {
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!("netsh interface portproxy delete v4tov4 listenport={} listenaddress=0.0.0.0", host_port),
            ])
            .output()
            .expect("Failed to execute PowerShell command");

        if !output.status.success() {
            let error_str = str::from_utf8(&output.stderr).unwrap();
            return Err(format!("Failed to remove port mapping: {}", error_str));
        }

        Ok(())
    }
}
