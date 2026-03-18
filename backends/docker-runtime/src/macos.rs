use super::{Container, ContainerManager, ContainerStatus, Result, RuntimeManager};
use bollard::Docker;
use bollard::container::{
    CreateContainerOptions, InspectContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::models::{
    ContainerCreateResponse, ContainerInspectResponse, HostConfig, PortBinding, PortMap,
};
use docker_types::DockerError;
use serde_json::to_string;
use std::collections::HashMap;
use std::time::Duration;

pub struct MacOSContainerManager {
    docker: Docker,
}

impl MacOSContainerManager {
    pub fn new() -> Self {
        let docker = Docker::connect_with_defaults().expect("Failed to connect to Docker");
        Self { docker }
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
        let options = name.map(|n| CreateContainerOptions { name: n.as_str() });

        // 构建端口映射
        let mut port_bindings: PortMap = HashMap::new();
        for port in &ports {
            // 简化处理，实际需要解析端口映射格式
            // 例如 "8080:80" 表示主机 8080 映射到容器 80
            let parts: Vec<&str> = port.split(":").collect();
            if parts.len() == 2 {
                let host_port = parts[0];
                let container_port = parts[1];
                port_bindings.insert(
                    format!("{}/tcp", container_port),
                    Some(vec![PortBinding {
                        host_ip: None,
                        host_port: Some(host_port.to_string()),
                    }]),
                );
            }
        }

        // 构建主机配置
        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            privileged: Some(privileged),
            read_only: Some(read_only),
            cap_add: if cap_add.is_empty() {
                None
            } else {
                Some(cap_add)
            },
            cap_drop: if cap_drop.is_empty() {
                None
            } else {
                Some(cap_drop)
            },
            restart_policy: restart_policy.map(|policy| bollard::models::RestartPolicy {
                name: Some(policy),
                maximum_retry_count: None,
            }),
            ..Default::default()
        };

        // 构建容器配置
        let config = bollard::models::ContainerConfig {
            image: Some(image.clone()),
            env: if environment.is_empty() {
                None
            } else {
                Some(environment)
            },
            ..Default::default()
        };

        // 创建容器
        let response = self.docker.create_container(options, config).map_err(|e| {
            DockerError::container_error(format!("Failed to create container: {:?}", e))
        })?;

        let container_id = response.id.unwrap_or_default();

        Ok(Container {
            id: container_id,
            name,
            image,
            status: ContainerStatus::Created,
            ports,
            environment: vec![], // 需要通过 inspect 获取
            volumes,
            secrets,
            cap_add,
            cap_drop,
            privileged,
            read_only,
        })
    }

    fn start(&mut self, container_id: &str) -> Result<()> {
        self.docker
            .start_container(container_id, None)
            .map_err(|e| {
                DockerError::container_error(format!("Failed to start container: {:?}", e))
            })?;
        Ok(())
    }

    fn stop(&mut self, container_id: &str) -> Result<()> {
        let options = StopContainerOptions {
            t: Some(10), // 10秒超时
        };
        self.docker
            .stop_container(container_id, Some(options))
            .map_err(|e| {
                DockerError::container_error(format!("Failed to stop container: {:?}", e))
            })?;
        Ok(())
    }

    fn delete(&mut self, container_id: &str) -> Result<()> {
        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };
        self.docker
            .remove_container(container_id, Some(options))
            .map_err(|e| {
                DockerError::container_error(format!("Failed to delete container: {:?}", e))
            })?;
        Ok(())
    }

    fn list(&mut self, all: bool) -> Result<Vec<Container>> {
        let options = ListContainersOptions {
            all: Some(all),
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).map_err(|e| {
            DockerError::container_error(format!("Failed to list containers: {:?}", e))
        })?;

        let result: Vec<Container> = containers
            .into_iter()
            .map(|container| {
                let name = container.names.as_ref().and_then(|names| {
                    names
                        .first()
                        .map(|name| name.trim_start_matches('/').to_string())
                });

                let status = match container.state.as_deref() {
                    Some("running") => ContainerStatus::Running,
                    Some("exited") => ContainerStatus::Exited,
                    Some("paused") => ContainerStatus::Paused,
                    Some("created") => ContainerStatus::Created,
                    _ => ContainerStatus::Stopped,
                };

                Container {
                    id: container.id.unwrap_or_default(),
                    name,
                    image: container.image.unwrap_or_default(),
                    status,
                    ports: vec![],       // 需要解析 ports 字段
                    environment: vec![], // 需要通过 inspect 获取
                    volumes: vec![],     // 需要通过 inspect 获取
                    secrets: vec![],     // 需要通过 inspect 获取
                    cap_add: vec![],     // 需要通过 inspect 获取
                    cap_drop: vec![],    // 需要通过 inspect 获取
                    privileged: false,   // 需要通过 inspect 获取
                    read_only: false,    // 需要通过 inspect 获取
                }
            })
            .collect();

        Ok(result)
    }

    fn inspect(&mut self, container_id: &str) -> Result<Container> {
        let container = self
            .docker
            .inspect_container(container_id, None)
            .map_err(|e| {
                DockerError::container_error(format!("Failed to inspect container: {:?}", e))
            })?;

        // 解析 secrets
        let secrets = container
            .config
            .as_ref()
            .and_then(|config| {
                config.secrets.as_ref().map(|secrets| {
                    secrets
                        .iter()
                        .filter_map(|secret| secret.name.as_ref().cloned())
                        .collect()
                })
            })
            .unwrap_or_default();

        // 解析 cap_add
        let cap_add = container
            .host_config
            .as_ref()
            .and_then(|host_config| host_config.cap_add.clone())
            .unwrap_or_default();

        // 解析 cap_drop
        let cap_drop = container
            .host_config
            .as_ref()
            .and_then(|host_config| host_config.cap_drop.clone())
            .unwrap_or_default();

        // 解析 privileged
        let privileged = container
            .host_config
            .as_ref()
            .and_then(|host_config| host_config.privileged)
            .unwrap_or(false);

        // 解析 read_only
        let read_only = container
            .host_config
            .as_ref()
            .and_then(|host_config| host_config.read_only)
            .unwrap_or(false);

        let name = container
            .name
            .as_ref()
            .map(|name| name.trim_start_matches('/').to_string());

        let image = container
            .config
            .as_ref()
            .and_then(|config| config.image.clone())
            .unwrap_or_default();

        let status = container
            .state
            .as_ref()
            .and_then(|state| {
                state.status.as_deref().map(|status| match status {
                    "running" => ContainerStatus::Running,
                    "exited" => ContainerStatus::Exited,
                    "paused" => ContainerStatus::Paused,
                    "created" => ContainerStatus::Created,
                    _ => ContainerStatus::Stopped,
                })
            })
            .unwrap_or(ContainerStatus::Stopped);

        Ok(Container {
            id: container.id.unwrap_or_default(),
            name,
            image,
            status,
            ports: vec![],       // 需要解析 ports 字段
            environment: vec![], // 需要解析 environment 字段
            volumes: vec![],     // 需要解析 volumes 字段
            secrets,
            cap_add,
            cap_drop,
            privileged,
            read_only,
        })
    }

    fn get_logs(&mut self, container_id: &str, lines: Option<u32>, follow: bool) -> Result<String> {
        use bollard::container::LogsOptions;
        use std::io::Read;

        let options = LogsOptions {
            follow: follow,
            stdout: true,
            stderr: true,
            tail: lines.map(|l| l.to_string()),
            ..Default::default()
        };

        let mut logs = self
            .docker
            .logs(container_id, Some(options))
            .map_err(|e| DockerError::container_error(format!("Failed to get logs: {:?}", e)))?;

        let mut output = String::new();
        logs.read_to_string(&mut output)
            .map_err(|e| DockerError::container_error(format!("Failed to read logs: {:?}", e)))?;

        Ok(output)
    }

    fn exec_command(&mut self, container_id: &str, command: &str, shell: bool) -> Result<String> {
        use bollard::container::{CreateExecOptions, StartExecOptions};
        use std::io::Read;

        let cmd = if shell {
            vec!["/bin/sh", "-c", command]
        } else {
            command.split_whitespace().map(|s| s.to_string()).collect()
        };

        let options = CreateExecOptions {
            cmd: Some(cmd),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(true),
            ..Default::default()
        };

        let exec = self
            .docker
            .create_exec(container_id, options)
            .map_err(|e| DockerError::container_error(format!("Failed to create exec: {:?}", e)))?;

        let exec_id = exec.id.unwrap_or_default();

        let options = StartExecOptions::default();
        let mut output = self
            .docker
            .start_exec(&exec_id, Some(options))
            .map_err(|e| DockerError::container_error(format!("Failed to start exec: {:?}", e)))?;

        let mut result = String::new();
        output.read_to_string(&mut result).map_err(|e| {
            DockerError::container_error(format!("Failed to read exec output: {:?}", e))
        })?;

        Ok(result)
    }
}

impl RuntimeManager for MacOSContainerManager {
    fn initialize(&mut self) -> Result<()> {
        // 检查Docker是否可用
        self.docker.info().map_err(|e| {
            DockerError::container_error(format!("Failed to check Docker status: {:?}", e))
        })?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        // 这里可以添加清理操作
        Ok(())
    }

    fn status(&mut self) -> Result<String> {
        let info = self.docker.info().map_err(|e| {
            DockerError::container_error(format!("Failed to get Docker info: {:?}", e))
        })?;
        to_string(&info).map_err(|e| {
            DockerError::container_error(format!("Failed to serialize Docker info: {:?}", e))
        })
    }

    fn version(&mut self) -> Result<String> {
        let version = self.docker.version().map_err(|e| {
            DockerError::container_error(format!("Failed to get Docker version: {:?}", e))
        })?;
        to_string(&version).map_err(|e| {
            DockerError::container_error(format!("Failed to serialize Docker version: {:?}", e))
        })
    }
}
