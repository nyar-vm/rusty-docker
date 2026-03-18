use super::{Container, ContainerManager, ContainerStatus, Result, RuntimeManager};
use docker_types::DockerError;
use std::{collections::HashMap, sync::RwLock};
use uuid::Uuid;

pub struct MacOSContainerManager {
    containers: RwLock<HashMap<String, Container>>,
}

impl MacOSContainerManager {
    pub fn new() -> Self {
        Self { containers: RwLock::new(HashMap::new()) }
    }
}

impl ContainerManager for MacOSContainerManager {
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
        let container_id = format!("{}", Uuid::new_v4());

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

impl RuntimeManager for MacOSContainerManager {
    fn initialize(&mut self) -> Result<()> {
        // 模拟初始化
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        // 模拟关闭
        Ok(())
    }

    fn status(&mut self) -> Result<String> {
        Ok("macOS runtime status: running".to_string())
    }

    fn version(&mut self) -> Result<String> {
        Ok("macOS runtime version: 1.0.0".to_string())
    }
}
