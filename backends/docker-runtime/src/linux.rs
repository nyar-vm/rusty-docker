use super::{Container, ContainerManager, ContainerStatus, Result, RuntimeManager};
use bollard::container::{
    CreateContainerOptions, InspectContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::models::{Config, ContainerCreateResponse, ContainerInspectResponse, HostConfig};
use bollard::{API_DEFAULT_VERSION, Docker};
use docker_types::DockerError;
use serde_json::from_str;
use std::time::Duration;

pub struct LinuxContainerManager {
    docker: Docker,
}

impl LinuxContainerManager {
    pub fn new() -> Self {
        let docker = Docker::connect_with_defaults().expect("Failed to connect to Docker");
        Self { docker }
    }
}

impl ContainerManager for LinuxContainerManager {
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
        let options = CreateContainerOptions {
            name: name.clone(),
            ..Default::default()
        };

        let host_config = HostConfig {
            privileged: Some(privileged),
            read_only: Some(read_only),
            cap_add: Some(cap_add),
            cap_drop: Some(cap_drop),
            restart_policy: restart_policy.map(|policy| bollard::models::RestartPolicy {
                name: Some(policy),
                maximum_retry_count: None,
            }),
            ..Default::default()
        };

        let config = Config {
            image: Some(image.clone()),
            env: Some(environment.clone()),
            ..Default::default()
        };

        let response = self
            .docker
            .create_container(Some(options), config)
            .map_err(|e| {
                DockerError::container_error(format!("Failed to create container: {:?}", e))
            })?;

        let container_id = response.id.unwrap_or_else(|| "".to_string());

        Ok(Container {
            id: container_id,
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
            t: 10, // 10秒超时
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
            all: all,
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).map_err(|e| {
            DockerError::container_error(format!("Failed to list containers: {:?}", e))
        })?;

        let containers: Vec<Container> = containers
            .into_iter()
            .map(|container| {
                let status = match container.state.as_deref() {
                    Some("running") => ContainerStatus::Running,
                    Some("exited") => ContainerStatus::Exited,
                    Some("paused") => ContainerStatus::Paused,
                    Some("created") => ContainerStatus::Created,
                    _ => ContainerStatus::Stopped,
                };

                Container {
                    id: container.id.unwrap_or_else(|| "".to_string()),
                    name: container.names.and_then(|names| {
                        names
                            .first()
                            .map(|name| name.trim_start_matches('/').to_string())
                    }),
                    image: container.image.unwrap_or_else(|| "".to_string()),
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

        Ok(containers)
    }

    fn inspect(&mut self, container_id: &str) -> Result<Container> {
        let options = InspectContainerOptions { size: false };

        let container = self
            .docker
            .inspect_container(container_id, Some(options))
            .map_err(|e| {
                DockerError::container_error(format!("Failed to inspect container: {:?}", e))
            })?;

        // 解析 secrets
        let secrets = if let Some(secrets) = &container.config.and_then(|c| c.secrets) {
            secrets
                .iter()
                .filter_map(|secret| secret.name.clone())
                .collect()
        } else {
            vec![]
        };

        // 解析 cap_add
        let cap_add = if let Some(cap_add) = &container.host_config.and_then(|c| c.cap_add) {
            cap_add.clone()
        } else {
            vec![]
        };

        // 解析 cap_drop
        let cap_drop = if let Some(cap_drop) = &container.host_config.and_then(|c| c.cap_drop) {
            cap_drop.clone()
        } else {
            vec![]
        };

        // 解析 privileged
        let privileged = container
            .host_config
            .and_then(|c| c.privileged)
            .unwrap_or(false);

        // 解析 read_only
        let read_only = container
            .host_config
            .and_then(|c| c.read_only)
            .unwrap_or(false);

        // 解析 status
        let status = match container.state.and_then(|s| s.status).as_deref() {
            Some("running") => ContainerStatus::Running,
            Some("exited") => ContainerStatus::Exited,
            Some("paused") => ContainerStatus::Paused,
            Some("created") => ContainerStatus::Created,
            _ => ContainerStatus::Stopped,
        };

        Ok(Container {
            id: container.id.unwrap_or_else(|| "".to_string()),
            name: container
                .name
                .map(|name| name.trim_start_matches('/').to_string()),
            image: container
                .config
                .and_then(|c| c.image)
                .unwrap_or_else(|| "".to_string()),
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
        use futures_util::StreamExt;

        let options = LogsOptions {
            follow: follow,
            stdout: true,
            stderr: true,
            tail: lines.map(|l| l.to_string()),
            ..Default::default()
        };

        let mut stream = self
            .docker
            .logs(container_id, Some(options))
            .map_err(|e| DockerError::container_error(format!("Failed to get logs: {:?}", e)))?;

        let mut logs = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Some(stdout) = chunk.stdout {
                        logs.push_str(&String::from_utf8_lossy(&stdout));
                    }
                    if let Some(stderr) = chunk.stderr {
                        logs.push_str(&String::from_utf8_lossy(&stderr));
                    }
                }
                Err(e) => {
                    return Err(DockerError::container_error(format!(
                        "Failed to read logs: {:?}",
                        e
                    )));
                }
            }
        }

        Ok(logs)
    }

    fn exec_command(&mut self, container_id: &str, command: &str, shell: bool) -> Result<String> {
        use bollard::container::{CreateExecOptions, StartExecOptions};
        use futures_util::StreamExt;

        let cmd = if shell {
            vec!["/bin/sh", "-c", command]
        } else {
            command.split_whitespace().collect()
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

        let exec_id = exec
            .id
            .ok_or_else(|| DockerError::container_error("Failed to get exec id".to_string()))?;

        let options = StartExecOptions {
            detach: false,
            tty: true,
        };

        let mut stream = self
            .docker
            .start_exec(&exec_id, Some(options))
            .map_err(|e| DockerError::container_error(format!("Failed to start exec: {:?}", e)))?;

        let mut output = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Some(stdout) = chunk.stdout {
                        output.push_str(&String::from_utf8_lossy(&stdout));
                    }
                    if let Some(stderr) = chunk.stderr {
                        output.push_str(&String::from_utf8_lossy(&stderr));
                    }
                }
                Err(e) => {
                    return Err(DockerError::container_error(format!(
                        "Failed to read exec output: {:?}",
                        e
                    )));
                }
            }
        }

        Ok(output)
    }
}

impl RuntimeManager for LinuxContainerManager {
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
        serde_json::to_string(&info).map_err(|e| {
            DockerError::container_error(format!("Failed to serialize Docker info: {:?}", e))
        })
    }

    fn version(&mut self) -> Result<String> {
        let version = self.docker.version().map_err(|e| {
            DockerError::container_error(format!("Failed to get Docker version: {:?}", e))
        })?;
        serde_json::to_string(&version).map_err(|e| {
            DockerError::container_error(format!("Failed to serialize Docker version: {:?}", e))
        })
    }
}
