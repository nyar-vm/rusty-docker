use super::{Container, ContainerManager, ContainerStatus, Result, RuntimeManager};
use crate::{file_share::FileShareManager, hyperv::HyperVManager, network_manager::NetworkManager, wsl2::Wsl2Manager};
use docker_types::DockerError;
use rand;
use std::{collections::HashMap, sync::RwLock};

pub struct WindowsContainerManager {
    containers: RwLock<HashMap<String, Container>>,
    hyperv_manager: HyperVManager,
    wsl2_manager: Wsl2Manager,
    file_share_manager: FileShareManager,
    network_manager: NetworkManager,
}

impl WindowsContainerManager {
    pub fn new() -> Self {
        Self {
            containers: RwLock::new(HashMap::new()),
            hyperv_manager: HyperVManager::new("rusty-docker"),
            wsl2_manager: Wsl2Manager::new("rusty-docker"),
            file_share_manager: FileShareManager::new(),
            network_manager: NetworkManager::new("rusty-docker-network"),
        }
    }
}

impl ContainerManager for WindowsContainerManager {
    fn create(
        &mut self,
        image: String,
        name: Option<String>,
        ports: Vec<String>,
        environment: Vec<String>,
        volumes: Vec<String>,
        restart_policy: Option<String>,
        healthcheck: Option<String>,
        deploy: Option<String>,
        secrets: Vec<String>,
        cap_add: Vec<String>,
        cap_drop: Vec<String>,
        privileged: bool,
        read_only: bool,
    ) -> Result<Container> {
        // 生成容器 ID
        let container_id: String = (0..32).map(|_| rand::random::<char>()).collect();

        // 检查 WSL 2 是否已安装
        if self.wsl2_manager.is_wsl2_installed() {
            // 使用 WSL 2 模式
            match self.wsl2_manager.create_distro() {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to create WSL distro: {}", e))),
            }

            // 启动 WSL 发行版
            match self.wsl2_manager.start_distro() {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to start WSL distro: {}", e))),
            }

            // 安装 Docker
            match self.wsl2_manager.install_docker() {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to install Docker in WSL: {}", e))),
            }
        }
        else {
            // 使用 Hyper-V 模式
            if !self.hyperv_manager.is_hyperv_enabled() {
                return Err(DockerError::runtime_error(
                    "Hyper-V is not enabled. Please enable Hyper-V in Windows Features or install WSL 2.",
                ));
            }

            // 创建 Hyper-V 虚拟机
            match self.hyperv_manager.create_vm(&container_id, 2048, 20) {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to create Hyper-V VM: {}", e))),
            }
        }

        // 处理卷挂载
        for volume in &volumes {
            // 解析卷挂载格式: host_path:container_path
            if let Some((host_path, container_path)) = volume.split_once(':') {
                match self.file_share_manager.add_mount(host_path, container_path) {
                    Ok(_) => (),
                    Err(e) => return Err(DockerError::runtime_error(&format!("Failed to add mount: {}", e))),
                }
            }
        }

        // 处理端口映射
        for port in &ports {
            // 解析端口映射格式: host_port:container_port
            if let Some((host_port_str, container_port_str)) = port.split_once(':') {
                if let (Ok(host_port), Ok(container_port)) = (host_port_str.parse::<u16>(), container_port_str.parse::<u16>()) {
                    match self.network_manager.configure_port_mapping(host_port, container_port) {
                        Ok(_) => (),
                        Err(e) => return Err(DockerError::runtime_error(&format!("Failed to configure port mapping: {}", e))),
                    }
                }
            }
        }

        // 创建容器
        let container = Container {
            id: container_id.clone(),
            name,
            image,
            status: ContainerStatus::Created,
            ports,
            environment,
            volumes,
            secrets,
            cap_add,
            cap_drop,
            privileged,
            read_only,
        };

        // 存储容器
        self.containers.write().unwrap().insert(container_id, container.clone());

        Ok(container)
    }

    fn start(&mut self, container_id: &str) -> Result<()> {
        // 检查 WSL 2 是否已安装
        if self.wsl2_manager.is_wsl2_installed() {
            // 使用 WSL 2 模式
            match self.wsl2_manager.start_distro() {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to start WSL distro: {}", e))),
            }

            // 处理 WSL 2 文件共享
            for (host_path, container_path) in &self.file_share_manager.mounts {
                let wsl_path = self.file_share_manager.windows_to_wsl_path(host_path.to_str().unwrap());
                // 在 WSL 中创建挂载点
                let command = format!(
                    "mkdir -p {} && mount --bind {} {}",
                    container_path.to_str().unwrap(),
                    wsl_path,
                    container_path.to_str().unwrap()
                );
                match self.wsl2_manager.exec_command(&command) {
                    Ok(_) => (),
                    Err(e) => return Err(DockerError::runtime_error(&format!("Failed to mount volume in WSL: {}", e))),
                }
            }

            // 配置 WSL 网络
            match self.network_manager.configure_wsl_network("rusty-docker") {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to configure WSL network: {}", e))),
            }
        }
        else {
            // 使用 Hyper-V 模式
            match self.hyperv_manager.start_vm(container_id) {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to start Hyper-V VM: {}", e))),
            }

            // 处理 Hyper-V 文件共享（这里需要具体的 Hyper-V 文件共享实现）
            // 暂时跳过，因为需要更复杂的 Hyper-V 集成
        }

        let mut containers = self.containers.write().unwrap();
        if let Some(container) = containers.get_mut(container_id) {
            container.status = ContainerStatus::Running;
            Ok(())
        }
        else {
            Err(DockerError::not_found("container", container_id.to_string()))
        }
    }

    fn stop(&mut self, container_id: &str) -> Result<()> {
        // 检查 WSL 2 是否已安装
        if self.wsl2_manager.is_wsl2_installed() {
            // 使用 WSL 2 模式
            match self.wsl2_manager.stop_distro() {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to stop WSL distro: {}", e))),
            }
        }
        else {
            // 使用 Hyper-V 模式
            match self.hyperv_manager.stop_vm(container_id) {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to stop Hyper-V VM: {}", e))),
            }
        }

        let mut containers = self.containers.write().unwrap();
        if let Some(container) = containers.get_mut(container_id) {
            container.status = ContainerStatus::Stopped;
            Ok(())
        }
        else {
            Err(DockerError::not_found("container", container_id.to_string()))
        }
    }

    fn delete(&mut self, container_id: &str) -> Result<()> {
        // 检查 WSL 2 是否已安装
        if self.wsl2_manager.is_wsl2_installed() {
            // 使用 WSL 2 模式
            match self.wsl2_manager.stop_distro() {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to stop WSL distro: {}", e))),
            }
        }
        else {
            // 使用 Hyper-V 模式
            match self.hyperv_manager.remove_vm(container_id) {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to remove Hyper-V VM: {}", e))),
            }
        }

        // 移除端口映射
        let containers = self.containers.read().unwrap();
        if let Some(container) = containers.get(container_id) {
            for port in &container.ports {
                // 解析端口映射格式: host_port:container_port
                if let Some((host_port_str, _)) = port.split_once(':') {
                    if let Ok(host_port) = host_port_str.parse::<u16>() {
                        let _ = self.network_manager.remove_port_mapping(host_port);
                    }
                }
            }
        }

        let mut containers = self.containers.write().unwrap();
        if containers.remove(container_id).is_some() {
            Ok(())
        }
        else {
            Err(DockerError::not_found("container", container_id.to_string()))
        }
    }

    fn list(&mut self, all: bool) -> Result<Vec<Container>> {
        let containers = self.containers.read().unwrap();
        if all {
            Ok(containers.values().cloned().collect())
        }
        else {
            Ok(containers.values().filter(|c| c.status == ContainerStatus::Running).cloned().collect())
        }
    }

    fn inspect(&mut self, container_id: &str) -> Result<Container> {
        let containers = self.containers.read().unwrap();
        if let Some(container) = containers.get(container_id) {
            Ok(container.clone())
        }
        else {
            Err(DockerError::not_found("container", container_id.to_string()))
        }
    }

    fn get_logs(&mut self, container_id: &str, lines: Option<u32>, follow: bool) -> Result<String> {
        let containers = self.containers.read().unwrap();
        if containers.contains_key(container_id) {
            Ok("Container logs not implemented in mock".to_string())
        }
        else {
            Err(DockerError::not_found("container", container_id.to_string()))
        }
    }

    fn exec_command(&mut self, container_id: &str, command: &str, shell: bool) -> Result<String> {
        let containers = self.containers.read().unwrap();
        if containers.contains_key(container_id) {
            Ok(format!("Command executed: {}", command))
        }
        else {
            Err(DockerError::not_found("container", container_id.to_string()))
        }
    }
}

impl RuntimeManager for WindowsContainerManager {
    fn initialize(&mut self) -> Result<()> {
        // 检查 WSL 2 是否已安装
        if self.wsl2_manager.is_wsl2_installed() {
            // 使用 WSL 2 模式
            match self.wsl2_manager.create_distro() {
                Ok(_) => (),
                Err(e) => return Err(DockerError::runtime_error(&format!("Failed to create WSL distro: {}", e))),
            }
        }
        else {
            // 使用 Hyper-V 模式
            if !self.hyperv_manager.is_hyperv_enabled() {
                return Err(DockerError::runtime_error(
                    "Hyper-V is not enabled. Please enable Hyper-V in Windows Features or install WSL 2.",
                ));
            }
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        // 模拟关闭
        Ok(())
    }

    fn status(&mut self) -> Result<String> {
        Ok("Windows runtime status: running".to_string())
    }

    fn version(&mut self) -> Result<String> {
        Ok("Windows runtime version: 1.0.0".to_string())
    }
}
