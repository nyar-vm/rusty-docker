use super::{Container, ContainerManager, ContainerStatus, Result, RuntimeManager};
use bollard::exec::{CreateExecOptions, StartExecOptions};
use bollard::models::{
    ContainerCreateResponse, ContainerInspectResponse, HostConfig, PortBinding, PortMap,
};
use bollard::query_parameters::{
    CreateContainerOptions, ListContainersOptions, RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use bollard::{API_DEFAULT_VERSION, Docker};
use docker_types::DockerError;
use futures_util::future::TryFutureExt;
use futures_util::stream::{StreamExt, TryStreamExt};
use serde_json::to_string;
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

pub struct WindowsContainerManager {
    client: Docker,
}

impl WindowsContainerManager {
    pub fn new() -> Self {
        let client = Docker::connect_with_http_defaults().unwrap();
        Self { client }
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
        let options = name.clone().map(|n| CreateContainerOptions {
            name: Some(n),
            platform: "windows".to_string(),
        });

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
            // read_only field doesn't exist in HostConfig
            cap_add: if cap_add.is_empty() {
                None
            } else {
                Some(cap_add.clone())
            },
            cap_drop: if cap_drop.is_empty() {
                None
            } else {
                Some(cap_drop.clone())
            },
            // 其他配置...
            ..Default::default()
        };

        // 构建容器配置
        let config = bollard::models::ContainerCreateBody {
            image: Some(image.clone()),
            env: if environment.is_empty() {
                None
            } else {
                Some(environment)
            },
            // 其他配置...
            ..Default::default()
        };

        // 创建容器
        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        let response = rt
            .block_on(self.client.create_container(options, config))
            .map_err(|e| {
                DockerError::container_error(format!("Failed to create container: {:?}", e))
            })?;

        let container_id = response.id;

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
        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        rt.block_on(self.client.start_container(container_id, None))
            .map_err(|e| {
                DockerError::container_error(format!("Failed to start container: {:?}", e))
            })?;
        Ok(())
    }

    fn stop(&mut self, container_id: &str) -> Result<()> {
        let options = StopContainerOptions {
            t: Some(10), // 10秒超时
            signal: None,
        };
        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        rt.block_on(self.client.stop_container(container_id, Some(options)))
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
        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        rt.block_on(self.client.remove_container(container_id, Some(options)))
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

        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        let containers = rt
            .block_on(self.client.list_containers(Some(options)))
            .map_err(|e| {
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

                let status = match container.state {
                    Some(bollard::models::ContainerSummaryStateEnum::RUNNING) => {
                        ContainerStatus::Running
                    }
                    Some(bollard::models::ContainerSummaryStateEnum::EXITED) => {
                        ContainerStatus::Exited
                    }
                    Some(bollard::models::ContainerSummaryStateEnum::PAUSED) => {
                        ContainerStatus::Paused
                    }
                    Some(bollard::models::ContainerSummaryStateEnum::CREATED) => {
                        ContainerStatus::Created
                    }
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
        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        let container = rt
            .block_on(self.client.inspect_container(container_id, None))
            .map_err(|e| {
                DockerError::container_error(format!("Failed to inspect container: {:?}", e))
            })?;

        // 解析 secrets
        let secrets = vec![];

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
        let read_only = false;

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
                state.status.map(|status| match status {
                    bollard::models::ContainerStateStatusEnum::RUNNING => ContainerStatus::Running,
                    bollard::models::ContainerStateStatusEnum::EXITED => ContainerStatus::Exited,
                    bollard::models::ContainerStateStatusEnum::PAUSED => ContainerStatus::Paused,
                    bollard::models::ContainerStateStatusEnum::CREATED => ContainerStatus::Created,
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
        use bollard::query_parameters::LogsOptions;

        let options = LogsOptions {
            follow: follow,
            stdout: true,
            stderr: true,
            tail: lines.map(|l| l.to_string()).unwrap_or("100".to_string()),
            ..Default::default()
        };

        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        let result = rt.block_on(async {
            let stream = self.client.logs(container_id, Some(options));

            let mut output = String::new();
            let mut stream = stream
                .map_err(|e| DockerError::container_error(format!("Failed to get logs: {:?}", e)));
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(chunk) => {
                        // Handle LogOutput - for now, just return an empty string
                    }
                    Err(e) => {
                        return Err(DockerError::container_error(format!(
                            "Failed to read logs: {:?}",
                            e
                        )));
                    }
                }
            }
            Ok(output)
        });

        result
    }

    fn exec_command(&mut self, container_id: &str, command: &str, shell: bool) -> Result<String> {
        let cmd = if shell {
            vec!["cmd", "/c", command]
        } else {
            command.split_whitespace().collect::<Vec<&str>>()
        };

        let options = CreateExecOptions {
            cmd: Some(cmd),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(true),
            ..Default::default()
        };

        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        let result = rt.block_on(async {
            let exec = self
                .client
                .create_exec(container_id, options)
                .map_err(|e| {
                    DockerError::container_error(format!("Failed to create exec: {:?}", e))
                })
                .await?;

            let exec_id = exec.id;

            let options = StartExecOptions::default();
            let exec_result = self
                .client
                .start_exec(&exec_id, Some(options))
                .map_err(|e| DockerError::container_error(format!("Failed to start exec: {:?}", e)))
                .await?;

            // Handle the exec result - for now, just return an empty string
            let output = String::new();
            Ok(output)
        });

        result
    }
}

impl RuntimeManager for WindowsContainerManager {
    fn initialize(&mut self) -> Result<()> {
        // 检查Docker是否可用
        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        rt.block_on(self.client.info()).map_err(|e| {
            DockerError::container_error(format!("Failed to check Docker status: {:?}", e))
        })?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        // 这里可以添加清理操作
        Ok(())
    }

    fn status(&mut self) -> Result<String> {
        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        let info = rt.block_on(self.client.info()).map_err(|e| {
            DockerError::container_error(format!("Failed to get Docker info: {:?}", e))
        })?;
        to_string(&info).map_err(|e| {
            DockerError::container_error(format!("Failed to serialize Docker info: {:?}", e))
        })
    }

    fn version(&mut self) -> Result<String> {
        let rt = Runtime::new().map_err(|e| {
            DockerError::container_error(format!("Failed to create runtime: {:?}", e))
        })?;
        let version = rt.block_on(self.client.version()).map_err(|e| {
            DockerError::container_error(format!("Failed to get Docker version: {:?}", e))
        })?;
        to_string(&version).map_err(|e| {
            DockerError::container_error(format!("Failed to serialize Docker version: {:?}", e))
        })
    }
}
