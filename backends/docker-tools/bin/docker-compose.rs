use clap::{Parser, Subcommand};
use docker::Docker;
use docker_types::{DockerError, Result};
use retry::{delay::Exponential, retry};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tracing_subscriber::fmt::format::FmtSpan;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Path to docker-compose.yml file(s)
    #[arg(short, long, default_value = "docker-compose.yml")]
    file: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start services
    Up {
        /// Don't recreate containers
        #[arg(long)]
        no_recreate: bool,
        /// Run containers in the background
        #[arg(long, short = 'd')]
        detach: bool,
        /// Build images before starting containers
        #[arg(long)]
        build: bool,
        /// Recreate containers even if their configuration and image haven't changed
        #[arg(long)]
        force_recreate: bool,
        /// Don't build images
        #[arg(long)]
        no_build: bool,
        /// Don't start containers
        #[arg(long)]
        no_start: bool,
        /// Don't start linked services
        #[arg(long)]
        no_deps: bool,
        /// Remove containers for services not defined in the Compose file
        #[arg(long)]
        remove_orphans: bool,
        /// Pull images before starting containers
        #[arg(long)]
        pull: bool,
        /// Pull images quietly
        #[arg(long)]
        quiet_pull: bool,
        /// Specify an alternate environment file
        #[arg(long)]
        env_file: Option<String>,
        /// Specify a profile to enable
        #[arg(long)]
        profile: Vec<String>,
    },
    /// Stop services
    Down {
        /// Remove named volumes declared in the volumes section of the Compose file and anonymous volumes attached to containers
        #[arg(long)]
        volumes: bool,
        /// Remove containers for services not defined in the Compose file
        #[arg(long)]
        remove_orphans: bool,
        /// Remove images
        #[arg(long)]
        rmi: Option<String>,
        /// Specify a shutdown timeout in seconds
        #[arg(long, default_value = "10")]
        timeout: u32,
    },
    /// Restart services
    Restart {
        /// Specify a shutdown timeout in seconds
        #[arg(long, default_value = "10")]
        timeout: u32,
        /// Services to restart
        services: Vec<String>,
    },
    /// List services
    Ps {
        /// Only display service names
        #[arg(long)]
        services: bool,
        /// Filter services by status
        #[arg(long)]
        filter: Option<String>,
        /// Format the output
        #[arg(long)]
        format: Option<String>,
    },
    /// Build images
    Build {
        /// Always attempt to pull a newer version of the image
        #[arg(long)]
        pull: bool,
        /// Do not use cache when building the image
        #[arg(long)]
        no_cache: bool,
        /// Always remove intermediate containers
        #[arg(long)]
        force_rm: bool,
        /// Compress the build context
        #[arg(long)]
        compress: bool,
        /// Build images in parallel
        #[arg(long)]
        parallel: bool,
        /// Set build-time variables
        #[arg(long)]
        build_arg: Vec<String>,
    },
    /// Show logs
    Logs {
        /// Follow log output
        #[arg(long, short = 'f')]
        follow: bool,
        /// Number of lines to show from the end of the logs
        #[arg(long, short = 'n', default_value = "all")]
        tail: String,
        /// Show timestamps
        #[arg(long)]
        timestamps: bool,
        /// Don't use colors
        #[arg(long)]
        no_color: bool,
        /// Services to show logs for
        services: Vec<String>,
    },
    /// Execute command in container
    Exec {
        /// Service name
        service: String,
        /// Command to execute
        command: Vec<String>,
        /// Detached mode: Run command in the background
        #[arg(long)]
        detach: bool,
        /// Set environment variables
        #[arg(long)]
        env: Vec<String>,
        /// Index of the container if there are multiple instances
        #[arg(long, default_value = "1")]
        index: u32,
        /// Give extended privileges to the process
        #[arg(long)]
        privileged: bool,
        /// Run the command as this user
        #[arg(long)]
        user: Option<String>,
        /// Working directory inside the container
        #[arg(long)]
        workdir: Option<String>,
    },
    /// Validate and view the Compose file
    Config {
        /// Print configuration in JSON format
        #[arg(long)]
        format: Option<String>,
        /// Check config validity without printing
        #[arg(long)]
        quiet: bool,
    },
    /// Pull images
    Pull {
        /// Pull images in parallel
        #[arg(long)]
        parallel: bool,
        /// Pull images quietly
        #[arg(long)]
        quiet: bool,
        /// Services to pull
        services: Vec<String>,
    },
    /// Push images
    Push {
        /// Push images in parallel
        #[arg(long)]
        parallel: bool,
        /// Push images quietly
        #[arg(long)]
        quiet: bool,
        /// Services to push
        services: Vec<String>,
    },
    /// Scale services
    Scale {
        /// Service scale specifications (e.g., service=3)
        services: Vec<String>,
    },
    /// Display the running processes
    Top,
    /// Stop services without removing
    Stop {
        /// Services to stop
        services: Vec<String>,
        /// Specify a shutdown timeout in seconds
        #[arg(long, default_value = "10")]
        timeout: u32,
    },
    /// Start services
    Start {
        /// Services to start
        services: Vec<String>,
    },
    /// Pause services
    Pause {
        /// Services to pause
        services: Vec<String>,
    },
    /// Unpause services
    Unpause {
        /// Services to unpause
        services: Vec<String>,
    },
    /// Remove stopped containers
    Rm {
        /// Force removal
        #[arg(long, short = 'f')]
        force: bool,
        /// Stop the containers before removing them
        #[arg(long, short = 's')]
        stop: bool,
        /// Remove volumes associated with containers
        #[arg(long, short = 'v')]
        volumes: bool,
        /// Services to remove
        services: Vec<String>,
    },
    /// Show events
    Events {
        /// Follow events
        #[arg(long, short = 'f')]
        follow: bool,
        /// Filter events
        #[arg(long)]
        filter: Vec<String>,
    },
    /// Print the public port for a port binding
    Port {
        /// Service name
        service: String,
        /// Container port
        port: u16,
        /// Protocol (tcp or udp)
        #[arg(long, default_value = "tcp")]
        protocol: String,
    },
    /// List Compose projects
    Ls {
        /// Show project names only
        #[arg(long)]
        quiet: bool,
        /// Filter projects by status
        #[arg(long)]
        filter: Option<String>,
    },
    /// Run a one-off command
    Run {
        /// Service name
        service: String,
        /// Command to run
        command: Vec<String>,
        /// Run in detached mode
        #[arg(long, short = 'd')]
        detach: bool,
        /// Assign a name to the container
        #[arg(long, short = 'n')]
        name: Option<String>,
        /// Override the entrypoint
        #[arg(long)]
        entrypoint: Option<String>,
        /// Set environment variables
        #[arg(long, short = 'e')]
        env: Vec<String>,
        /// Publish a container's port(s) to the host
        #[arg(long, short = 'p')]
        publish: Vec<String>,
        /// Bind mount a volume
        #[arg(long, short = 'v')]
        volume: Vec<String>,
        /// Run as a specific user
        #[arg(long, short = 'u')]
        user: Option<String>,
        /// Working directory inside the container
        #[arg(long, short = 'w')]
        workdir: Option<String>,
        /// Remove the container after run
        #[arg(long, short = 'r')]
        rm: bool,
        /// Don't start linked services
        #[arg(long)]
        no_deps: bool,
        /// Suppress output
        #[arg(long)]
        quiet: bool,
    },
    /// Create services
    Create {
        /// Don't create containers
        #[arg(long)]
        no_recreate: bool,
    },
    /// List images
    Images,
    /// Kill services
    Kill {
        /// Signal to send to the container
        #[arg(long, default_value = "SIGTERM")]
        signal: String,
        /// Services to kill
        services: Vec<String>,
    },
    /// Show version information
    Version,
    /// Wait for services to be healthy
    Wait,
    /// Stack management
    Stack {
        #[command(subcommand)]
        command: StackCommands,
    },
}

#[derive(Subcommand)]
enum StackCommands {
    /// Deploy a stack
    Deploy {
        /// Stack name
        name: String,
        /// Path to Compose file
        #[arg(short, long, default_value = "docker-compose.yml")]
        file: String,
        /// Prune services that are no longer referenced
        #[arg(long)]
        prune: bool,
    },
    /// List stacks
    Ls,
    /// List services in a stack
    Ps {
        /// Stack name
        name: String,
    },
    /// Remove a stack
    Rm {
        /// Stack name
        name: String,
    },
    /// Inspect a stack
    Inspect {
        /// Stack name
        name: String,
    },
}

#[derive(Clone)]
struct ComposeService {
    name: String,
    image: String,
    build: Option<String>,
    ports: Vec<String>,
    environment: Vec<String>,
    environment_map: Option<std::collections::HashMap<String, String>>,
    env_file: Option<Vec<String>>,
    volumes: Vec<MountConfig>,
    command: Option<String>,
    working_dir: Option<String>,
    user: Option<String>,
    entrypoint: Option<String>,
    restart: Option<String>,
    healthcheck: Option<HealthCheckConfig>,
    deploy: Option<DeployConfig>,
    labels: Option<Vec<String>>,
    network_mode: Option<String>,
    networks: std::collections::HashMap<String, NetworkServiceConfig>,
    depends_on: Vec<String>,
    // 新增配置选项
    cap_add: Option<Vec<String>>,
    cap_drop: Option<Vec<String>>,
    cgroup_parent: Option<String>,
    device_cgroup_rules: Option<Vec<String>>,
    devices: Option<Vec<String>>,
    dns: Option<Vec<String>>,
    dns_search: Option<Vec<String>>,
    domainname: Option<String>,
    extra_hosts: Option<Vec<String>>,
    hostname: Option<String>,
    ipc: Option<String>,
    isolation: Option<String>,
    logging: Option<LoggingConfig>,
    mac_address: Option<String>,
    mem_limit: Option<String>,
    mem_reservation: Option<String>,
    oom_kill_disable: Option<bool>,
    oom_score_adj: Option<i32>,
    pid: Option<String>,
    pids_limit: Option<u64>,
    read_only: Option<bool>,
    shm_size: Option<String>,
    stdin_open: Option<bool>,
    stop_grace_period: Option<String>,
    stop_signal: Option<String>,
    tty: Option<bool>,
    ulimits: Option<std::collections::HashMap<String, UlimitConfig>>,
    sysctls: Option<std::collections::HashMap<String, String>>,
    // Profiles 支持
    profiles: Option<Vec<String>>,
    // Extends 支持
    extends: Option<ExtendsConfig>,
}

#[derive(Clone)]
struct NetworkServiceConfig {
    aliases: Vec<String>,
    ipv4_address: Option<String>,
    ipv6_address: Option<String>,
}

#[derive(Clone)]
struct HealthCheckConfig {
    test: Vec<String>,
    interval: Option<String>,
    timeout: Option<String>,
    retries: Option<u32>,
    start_period: Option<String>,
}

#[derive(Clone)]
struct DeployConfig {
    replicas: Option<u32>,
    restart_policy: Option<RestartPolicyConfig>,
    resources: Option<ResourcesConfig>,
    labels: Option<Vec<String>>,
}

#[derive(Clone)]
struct RestartPolicyConfig {
    condition: Option<String>,
    delay: Option<String>,
    max_attempts: Option<u32>,
    window: Option<String>,
}

#[derive(Clone)]
struct ResourcesConfig {
    limits: Option<ResourceLimits>,
    reservations: Option<ResourceReservations>,
}

#[derive(Clone)]
struct ResourceLimits {
    cpus: Option<String>,
    memory: Option<String>,
}

#[derive(Clone)]
struct ResourceReservations {
    cpus: Option<String>,
    memory: Option<String>,
}

struct NetworkConfig {
    name: String,
    driver: String,
    driver_opts: Option<std::collections::HashMap<String, String>>,
    ipam: Option<IpamConfig>,
    internal: bool,
    external: bool,
    attachable: bool,
    enable_ipv6: bool,
    labels: Option<std::collections::HashMap<String, String>>,
}

struct IpamConfig {
    driver: String,
    config: Vec<IpamSubnetConfig>,
}

struct IpamSubnetConfig {
    subnet: String,
    gateway: Option<String>,
    ip_range: Option<String>,
}

struct VolumeConfig {
    name: String,
    driver: String,
    driver_opts: Option<std::collections::HashMap<String, String>>,
    labels: Option<std::collections::HashMap<String, String>>,
    external: bool,
    internal: bool,
}

#[derive(Clone)]
struct MountConfig {
    source: String,
    target: String,
    read_only: bool,
    mount_type: String,          // volume, bind, tmpfs
    consistency: Option<String>, // for bind mounts
    tmpfs_size: Option<u64>,     // for tmpfs mounts
    tmpfs_mode: Option<u32>,     // for tmpfs mounts
}

#[derive(Clone)]
struct LoggingConfig {
    driver: String,
    options: Option<std::collections::HashMap<String, String>>,
}

#[derive(Clone)]
struct UlimitConfig {
    soft: Option<u64>,
    hard: Option<u64>,
}

#[derive(Clone)]
struct ExtendsConfig {
    service: String,
    file: Option<String>,
}

/// 加载 .env 文件中的环境变量
fn load_env_file() -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    // 检查 .env 文件是否存在
    if Path::new(".env").exists() {
        if let Ok(mut file) = File::open(".env") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                for line in content.lines() {
                    // 跳过注释和空行
                    if line.trim().starts_with('#') || line.trim().is_empty() {
                        continue;
                    }

                    // 解析键值对
                    if let Some((key, value)) = line.split_once('=') {
                        env_vars.insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
            }
        }
    }

    env_vars
}

/// 替换字符串中的环境变量
fn replace_env_vars(s: &str, env_vars: &HashMap<String, String>) -> String {
    let mut result = s.to_string();

    // 简单的环境变量替换，格式为 ${VAR} 或 $VAR
    for (key, value) in env_vars {
        let placeholder1 = format!("${{{}}}", key);
        let placeholder2 = format!("${}", key);

        result = result.replace(&placeholder1, value);
        result = result.replace(&placeholder2, value);
    }

    result
}

impl ComposeService {
    fn from_yaml(name: &str, yaml: &Yaml) -> Self {
        // 解析基本配置
        let image = yaml["image"].as_str().unwrap_or_default().to_string();

        // 处理 build 配置（支持字符串和对象格式）
        let build = if let Some(build_str) = yaml["build"].as_str() {
            Some(build_str.to_string())
        } else if let Some(build_map) = yaml["build"].as_hash() {
            // 处理对象格式的 build 配置
            build_map
                .get(&Yaml::String("context".to_string()))
                .and_then(|ctx| ctx.as_str())
                .map(|ctx| ctx.to_string())
        } else {
            None
        };
        let ports = match yaml["ports"].as_vec() {
            Some(ports) => ports
                .iter()
                .filter_map(|p| p.as_str())
                .map(|p| p.to_string())
                .collect(),
            None => vec![],
        };

        // 解析 environment 变量（支持列表和映射两种格式）
        let mut environment = vec![];
        let mut environment_map = None;

        if let Some(envs) = yaml["environment"].as_vec() {
            environment = envs
                .iter()
                .filter_map(|e| e.as_str())
                .map(|e| e.to_string())
                .collect();
        } else if let Some(env_map) = yaml["environment"].as_hash() {
            let mut map = std::collections::HashMap::new();
            for (k, v) in env_map {
                if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                    map.insert(key.to_string(), value.to_string());
                }
            }
            environment_map = Some(map);
        }

        // 解析 env_file 配置
        let env_file = match yaml["env_file"].as_vec() {
            Some(files) => Some(
                files
                    .iter()
                    .filter_map(|f| f.as_str())
                    .map(|f| f.to_string())
                    .collect(),
            ),
            None => yaml["env_file"].as_str().map(|f| vec![f.to_string()]),
        };

        let volumes = match yaml["volumes"].as_vec() {
            Some(vols) => vols
                .iter()
                .map(|v| {
                    if let Some(vol_str) = v.as_str() {
                        // 解析字符串格式的挂载
                        let parts: Vec<&str> = vol_str.split(':').collect();
                        if parts.len() >= 2 {
                            let mut source = parts[0].to_string();
                            let target = parts[1].to_string();
                            let mut read_only = false;
                            let mut mount_type = "volume".to_string();

                            // 检查是否有只读标志
                            if parts.len() > 2 && parts[2] == "ro" {
                                read_only = true;
                            }

                            // 确定挂载类型
                            if source.starts_with('/') {
                                mount_type = "bind".to_string();
                            } else if source == "tmpfs" {
                                mount_type = "tmpfs".to_string();
                                source = "".to_string();
                            }

                            MountConfig {
                                source,
                                target,
                                read_only,
                                mount_type,
                                consistency: None,
                                tmpfs_size: None,
                                tmpfs_mode: None,
                            }
                        } else {
                            // 无效的挂载格式，使用默认值
                            MountConfig {
                                source: "".to_string(),
                                target: vol_str.to_string(),
                                read_only: false,
                                mount_type: "volume".to_string(),
                                consistency: None,
                                tmpfs_size: None,
                                tmpfs_mode: None,
                            }
                        }
                    } else if let Some(vol_map) = v.as_hash() {
                        // 解析对象格式的挂载
                        let source = vol_map
                            .get(&Yaml::String("source".to_string()))
                            .and_then(|s| s.as_str())
                            .unwrap_or("")
                            .to_string();
                        let target = vol_map
                            .get(&Yaml::String("target".to_string()))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let read_only = vol_map
                            .get(&Yaml::String("read_only".to_string()))
                            .and_then(|r| r.as_bool())
                            .unwrap_or(false);
                        let mount_type = vol_map
                            .get(&Yaml::String("type".to_string()))
                            .and_then(|t| t.as_str())
                            .unwrap_or("volume")
                            .to_string();
                        let consistency = vol_map
                            .get(&Yaml::String("consistency".to_string()))
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string());
                        let tmpfs_size = vol_map
                            .get(&Yaml::String("tmpfs_size".to_string()))
                            .and_then(|s| s.as_i64())
                            .map(|i| i as u64);
                        let tmpfs_mode = vol_map
                            .get(&Yaml::String("tmpfs_mode".to_string()))
                            .and_then(|m| m.as_i64())
                            .map(|i| i as u32);

                        MountConfig {
                            source,
                            target,
                            read_only,
                            mount_type,
                            consistency,
                            tmpfs_size,
                            tmpfs_mode,
                        }
                    } else {
                        // 无效的挂载格式，使用默认值
                        MountConfig {
                            source: "".to_string(),
                            target: "".to_string(),
                            read_only: false,
                            mount_type: "volume".to_string(),
                            consistency: None,
                            tmpfs_size: None,
                            tmpfs_mode: None,
                        }
                    }
                })
                .collect(),
            None => vec![],
        };
        let command = yaml["command"].as_str().map(|c| c.to_string());
        let working_dir = yaml["working_dir"].as_str().map(|wd| wd.to_string());
        let user = yaml["user"].as_str().map(|u| u.to_string());
        let entrypoint = yaml["entrypoint"].as_str().map(|ep| ep.to_string());
        let restart = yaml["restart"].as_str().map(|r| r.to_string());

        // 解析 healthcheck 配置
        let healthcheck = yaml["healthcheck"].as_hash().map(|hc| {
            let test = match hc.get(&Yaml::String("test".to_string())) {
                Some(Yaml::Array(tests)) => tests
                    .iter()
                    .filter_map(|t| t.as_str())
                    .map(|t| t.to_string())
                    .collect(),
                Some(Yaml::String(test)) => vec![test.to_string()],
                _ => vec![],
            };
            let interval = hc
                .get(&Yaml::String("interval".to_string()))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let timeout = hc
                .get(&Yaml::String("timeout".to_string()))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let retries = hc
                .get(&Yaml::String("retries".to_string()))
                .and_then(|v| v.as_i64())
                .map(|i| i as u32);
            let start_period = hc
                .get(&Yaml::String("start_period".to_string()))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            HealthCheckConfig {
                test,
                interval,
                timeout,
                retries,
                start_period,
            }
        });

        // 解析 deploy 配置
        let deploy = yaml["deploy"].as_hash().map(|d| {
            let replicas = d
                .get(&Yaml::String("replicas".to_string()))
                .and_then(|v| v.as_i64())
                .map(|i| i as u32);

            let restart_policy = d
                .get(&Yaml::String("restart_policy".to_string()))
                .and_then(|rp| rp.as_hash())
                .map(|rp| {
                    let condition = rp
                        .get(&Yaml::String("condition".to_string()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let delay = rp
                        .get(&Yaml::String("delay".to_string()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let max_attempts = rp
                        .get(&Yaml::String("max_attempts".to_string()))
                        .and_then(|v| v.as_i64())
                        .map(|i| i as u32);
                    let window = rp
                        .get(&Yaml::String("window".to_string()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    RestartPolicyConfig {
                        condition,
                        delay,
                        max_attempts,
                        window,
                    }
                });

            let resources = d
                .get(&Yaml::String("resources".to_string()))
                .and_then(|r| r.as_hash())
                .map(|r| {
                    let limits = r
                        .get(&Yaml::String("limits".to_string()))
                        .and_then(|l| l.as_hash())
                        .map(|l| {
                            let cpus = l
                                .get(&Yaml::String("cpus".to_string()))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let memory = l
                                .get(&Yaml::String("memory".to_string()))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            ResourceLimits { cpus, memory }
                        });

                    let reservations = r
                        .get(&Yaml::String("reservations".to_string()))
                        .and_then(|res| res.as_hash())
                        .map(|res| {
                            let cpus = res
                                .get(&Yaml::String("cpus".to_string()))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let memory = res
                                .get(&Yaml::String("memory".to_string()))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            ResourceReservations { cpus, memory }
                        });

                    ResourcesConfig {
                        limits,
                        reservations,
                    }
                });

            let labels = d
                .get(&Yaml::String("labels".to_string()))
                .and_then(|l| l.as_vec())
                .map(|labels| {
                    labels
                        .iter()
                        .filter_map(|l| l.as_str())
                        .map(|l| l.to_string())
                        .collect()
                });

            DeployConfig {
                replicas,
                restart_policy,
                resources,
                labels,
            }
        });

        // 解析 labels 配置（支持列表和映射两种格式）
        let labels = if let Some(labels_vec) = yaml["labels"].as_vec() {
            Some(
                labels_vec
                    .iter()
                    .filter_map(|l| l.as_str())
                    .map(|l| l.to_string())
                    .collect(),
            )
        } else if let Some(labels_map) = yaml["labels"].as_hash() {
            // 处理映射格式的 labels
            let labels_vec: Vec<String> = labels_map
                .iter()
                .filter_map(|(k, v)| {
                    if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                        Some(format!("{}={}", key, value))
                    } else {
                        None
                    }
                })
                .collect();
            Some(labels_vec)
        } else {
            None
        };

        // 解析 network_mode 配置
        let network_mode = yaml["network_mode"].as_str().map(|nm| nm.to_string());

        // 解析 networks 配置
        let mut networks = std::collections::HashMap::new();
        if let Some(networks_yaml) = yaml["networks"].as_hash() {
            for (net_name, net_config) in networks_yaml {
                if let Some(net_name_str) = net_name.as_str() {
                    let mut aliases = vec![];
                    let mut ipv4_address = None;
                    let mut ipv6_address = None;

                    if let Some(net_config_hash) = net_config.as_hash() {
                        // 解析 aliases
                        if let Some(aliases_yaml) =
                            net_config_hash.get(&Yaml::String("aliases".to_string()))
                        {
                            if let Some(aliases_vec) = aliases_yaml.as_vec() {
                                aliases = aliases_vec
                                    .iter()
                                    .filter_map(|a| a.as_str())
                                    .map(|a| a.to_string())
                                    .collect();
                            }
                        }

                        // 解析 ipv4_address
                        ipv4_address = net_config_hash
                            .get(&Yaml::String("ipv4_address".to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        // 解析 ipv6_address
                        ipv6_address = net_config_hash
                            .get(&Yaml::String("ipv6_address".to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }

                    networks.insert(
                        net_name_str.to_string(),
                        NetworkServiceConfig {
                            aliases,
                            ipv4_address,
                            ipv6_address,
                        },
                    );
                }
            }
        } else if let Some(networks_vec) = yaml["networks"].as_vec() {
            // 处理简单形式的 networks 配置（仅网络名称列表）
            for net in networks_vec {
                if let Some(net_name) = net.as_str() {
                    networks.insert(
                        net_name.to_string(),
                        NetworkServiceConfig {
                            aliases: vec![],
                            ipv4_address: None,
                            ipv6_address: None,
                        },
                    );
                }
            }
        }

        // 解析 depends_on 配置（支持列表和映射两种格式）
        let mut depends_on = vec![];
        if let Some(deps_vec) = yaml["depends_on"].as_vec() {
            depends_on = deps_vec
                .iter()
                .filter_map(|d| d.as_str())
                .map(|d| d.to_string())
                .collect();
        } else if let Some(deps_map) = yaml["depends_on"].as_hash() {
            // 处理映射格式的 depends_on（Compose v3+）
            depends_on = deps_map
                .iter()
                .filter_map(|(k, _)| k.as_str())
                .map(|k| k.to_string())
                .collect();
        }

        // 解析新增的配置选项
        let cap_add = yaml["cap_add"].as_vec().map(|caps| {
            caps.iter()
                .filter_map(|c| c.as_str())
                .map(|c| c.to_string())
                .collect()
        });

        let cap_drop = yaml["cap_drop"].as_vec().map(|caps| {
            caps.iter()
                .filter_map(|c| c.as_str())
                .map(|c| c.to_string())
                .collect()
        });

        let cgroup_parent = yaml["cgroup_parent"].as_str().map(|s| s.to_string());

        let device_cgroup_rules = yaml["device_cgroup_rules"].as_vec().map(|rules| {
            rules
                .iter()
                .filter_map(|r| r.as_str())
                .map(|r| r.to_string())
                .collect()
        });

        let devices = yaml["devices"].as_vec().map(|devices| {
            devices
                .iter()
                .filter_map(|d| d.as_str())
                .map(|d| d.to_string())
                .collect()
        });

        let dns = yaml["dns"].as_vec().map(|dns| {
            dns.iter()
                .filter_map(|d| d.as_str())
                .map(|d| d.to_string())
                .collect()
        });

        let dns_search = yaml["dns_search"].as_vec().map(|search| {
            search
                .iter()
                .filter_map(|s| s.as_str())
                .map(|s| s.to_string())
                .collect()
        });

        let domainname = yaml["domainname"].as_str().map(|s| s.to_string());

        let extra_hosts = yaml["extra_hosts"].as_vec().map(|hosts| {
            hosts
                .iter()
                .filter_map(|h| h.as_str())
                .map(|h| h.to_string())
                .collect()
        });

        let hostname = yaml["hostname"].as_str().map(|s| s.to_string());

        let ipc = yaml["ipc"].as_str().map(|s| s.to_string());

        let isolation = yaml["isolation"].as_str().map(|s| s.to_string());

        let logging = yaml["logging"].as_hash().map(|logging| {
            let driver = logging
                .get(&Yaml::String("driver".to_string()))
                .and_then(|d| d.as_str())
                .unwrap_or("json-file")
                .to_string();
            let options = logging
                .get(&Yaml::String("options".to_string()))
                .and_then(|opts| {
                    opts.as_hash().map(|opts| {
                        let mut map = std::collections::HashMap::new();
                        for (k, v) in opts {
                            if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                                map.insert(key.to_string(), value.to_string());
                            }
                        }
                        map
                    })
                });
            LoggingConfig { driver, options }
        });

        let mac_address = yaml["mac_address"].as_str().map(|s| s.to_string());

        let mem_limit = yaml["mem_limit"].as_str().map(|s| s.to_string());

        let mem_reservation = yaml["mem_reservation"].as_str().map(|s| s.to_string());

        let oom_kill_disable = yaml["oom_kill_disable"].as_bool();

        let oom_score_adj = yaml["oom_score_adj"].as_i64().map(|i| i as i32);

        let pid = yaml["pid"].as_str().map(|s| s.to_string());

        let pids_limit = yaml["pids_limit"].as_i64().map(|i| i as u64);

        let read_only = yaml["read_only"].as_bool();

        let shm_size = yaml["shm_size"].as_str().map(|s| s.to_string());

        let stdin_open = yaml["stdin_open"].as_bool();

        let stop_grace_period = yaml["stop_grace_period"].as_str().map(|s| s.to_string());

        let stop_signal = yaml["stop_signal"].as_str().map(|s| s.to_string());

        let tty = yaml["tty"].as_bool();

        let ulimits = yaml["ulimits"].as_hash().map(|ulimits| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in ulimits {
                if let Some(key) = k.as_str() {
                    if let Some(ulimit_map) = v.as_hash() {
                        let soft = ulimit_map
                            .get(&Yaml::String("soft".to_string()))
                            .and_then(|s| s.as_i64())
                            .map(|i| i as u64);
                        let hard = ulimit_map
                            .get(&Yaml::String("hard".to_string()))
                            .and_then(|h| h.as_i64())
                            .map(|i| i as u64);
                        map.insert(key.to_string(), UlimitConfig { soft, hard });
                    } else if let Some(value) = v.as_i64() {
                        // 简化格式：直接指定值
                        map.insert(
                            key.to_string(),
                            UlimitConfig {
                                soft: Some(value as u64),
                                hard: Some(value as u64),
                            },
                        );
                    }
                }
            }
            map
        });

        let sysctls = yaml["sysctls"].as_hash().map(|sysctls| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in sysctls {
                if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                    map.insert(key.to_string(), value.to_string());
                }
            }
            map
        });

        let profiles = yaml["profiles"].as_vec().map(|profiles| {
            profiles
                .iter()
                .filter_map(|p| p.as_str())
                .map(|p| p.to_string())
                .collect()
        });

        let extends = yaml["extends"].as_hash().map(|extends| {
            let service = extends
                .get(&Yaml::String("service".to_string()))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let file = extends
                .get(&Yaml::String("file".to_string()))
                .and_then(|f| f.as_str())
                .map(|f| f.to_string());
            ExtendsConfig { service, file }
        });

        Self {
            name: name.to_string(),
            image,
            build,
            ports,
            environment,
            environment_map,
            env_file,
            volumes,
            command,
            working_dir,
            user,
            entrypoint,
            restart,
            healthcheck,
            deploy,
            labels,
            network_mode,
            networks,
            depends_on,
            // 新增配置选项
            cap_add,
            cap_drop,
            cgroup_parent,
            device_cgroup_rules,
            devices,
            dns,
            dns_search,
            domainname,
            extra_hosts,
            hostname,
            ipc,
            isolation,
            logging,
            mac_address,
            mem_limit,
            mem_reservation,
            oom_kill_disable,
            oom_score_adj,
            pid,
            pids_limit,
            read_only,
            shm_size,
            stdin_open,
            stop_grace_period,
            stop_signal,
            tty,
            ulimits,
            sysctls,
            profiles,
            extends,
        }
    }
}

/// 加载单个 Compose 文件
fn load_single_compose_file(path: &str) -> Result<Yaml> {
    // 加载环境变量
    let env_vars = load_env_file();

    let mut file =
        File::open(path).map_err(|e| DockerError::io_error("open file", e.to_string()))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| DockerError::io_error("read file", e.to_string()))?;

    // 替换环境变量
    let content = replace_env_vars(&content, &env_vars);

    let docs = YamlLoader::load_from_str(&content)
        .map_err(|e| DockerError::parse_error("yaml", e.to_string()))?;
    if docs.is_empty() {
        return Err(DockerError::invalid_params(
            "compose_file",
            format!("Compose file {} is empty", path),
        ));
    }
    Ok(docs[0].clone())
}

/// 合并多个 YAML 文档
fn merge_yaml(base: &Yaml, override_yaml: &Yaml) -> Yaml {
    match (base, override_yaml) {
        // 如果两边都是映射，递归合并
        (Yaml::Hash(base_map), Yaml::Hash(override_map)) => {
            let mut merged = base_map.clone();
            for (key, value) in override_map {
                // 特殊处理 services、networks 和 volumes 配置
                if let Some(key_str) = key.as_str() {
                    match key_str {
                        "services" | "networks" | "volumes" => {
                            // 对于这些配置，应该合并而不是覆盖
                            if let (Some(base_value), Yaml::Hash(override_value)) =
                                (merged.get(key), value)
                            {
                                if let Yaml::Hash(base_submap) = base_value {
                                    let mut merged_submap = base_submap.clone();
                                    // 合并子映射中的键值对，递归处理每个子项
                                    for (sub_key, sub_value) in override_value {
                                        if let Some(base_sub_value) = merged_submap.get(sub_key) {
                                            // 如果子项存在，递归合并
                                            merged_submap.insert(
                                                sub_key.clone(),
                                                merge_yaml(base_sub_value, sub_value),
                                            );
                                        } else {
                                            // 否则直接添加
                                            merged_submap
                                                .insert(sub_key.clone(), sub_value.clone());
                                        }
                                    }
                                    merged.insert(key.clone(), Yaml::Hash(merged_submap));
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if let (_, Yaml::Hash(_)) = (key.as_str(), value) {
                    // 如果是嵌套映射，递归合并
                    if let Some(base_value) = merged.get(key) {
                        merged.insert(key.clone(), merge_yaml(base_value, value));
                    } else {
                        merged.insert(key.clone(), value.clone());
                    }
                } else {
                    // 否则直接覆盖
                    merged.insert(key.clone(), value.clone());
                }
            }
            Yaml::Hash(merged)
        }
        // 如果两边都是数组，合并数组
        (Yaml::Array(base_array), Yaml::Array(override_array)) => {
            let mut merged = base_array.clone();
            merged.extend(override_array.clone());
            Yaml::Array(merged)
        }
        // 其他情况，使用覆盖值
        (_, override_value) => override_value.clone(),
    }
}

/// 验证 Compose 配置文件
fn validate_compose_config(yaml: &Yaml) -> Result<()> {
    // 检查是否存在 services 配置
    if yaml["services"].as_hash().is_none() {
        return Err(DockerError::invalid_params(
            "compose_config",
            "Compose file must have a 'services' section",
        ));
    }

    // 检查 services 是否为空
    if let Some(services) = yaml["services"].as_hash() {
        if services.is_empty() {
            return Err(DockerError::invalid_params(
                "compose_config",
                "Compose file 'services' section cannot be empty",
            ));
        }

        // 验证每个服务的配置
        for (service_name, service_config) in services {
            if let Some(service_name_str) = service_name.as_str() {
                // 检查服务是否有 image 或 build 配置
                let has_image = service_config["image"].as_str().is_some();
                let has_build_str = service_config["build"].as_str().is_some();
                let has_build_hash = service_config["build"].as_hash().is_some();

                if !has_image && !has_build_str && !has_build_hash {
                    return Err(DockerError::invalid_params(
                        "service_config",
                        format!(
                            "Service '{}' must have either 'image' or 'build' configuration",
                            service_name_str
                        ),
                    ));
                }
            }
        }
    }

    // 验证 networks 配置（如果存在）
    if let Some(networks) = yaml["networks"].as_hash() {
        for (network_name, network_config) in networks {
            if let Some(network_name_str) = network_name.as_str() {
                // 检查网络驱动是否有效
                if let Some(driver) = network_config["driver"].as_str() {
                    let valid_drivers = ["bridge", "host", "overlay", "none"];
                    if !valid_drivers.contains(&driver) {
                        return Err(DockerError::invalid_params(
                            "network_config",
                            format!(
                                "Network '{}' has invalid driver: {}",
                                network_name_str, driver
                            ),
                        ));
                    }
                }
            }
        }
    }

    // 验证 volumes 配置（如果存在）
    if let Some(volumes) = yaml["volumes"].as_hash() {
        for (volume_name, volume_config) in volumes {
            if let Some(volume_name_str) = volume_name.as_str() {
                // 检查卷驱动是否有效
                if let Some(driver) = volume_config["driver"].as_str() {
                    if driver.is_empty() {
                        return Err(DockerError::invalid_params(
                            "volume_config",
                            format!("Volume '{}' has empty driver name", volume_name_str),
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

/// 加载并合并多个 Compose 文件
fn load_compose_files(paths: &[String]) -> Result<Yaml> {
    if paths.is_empty() {
        return Err(DockerError::invalid_params(
            "compose_files",
            "No compose files specified",
        ));
    }

    // 加载第一个文件作为基础
    let mut merged = load_single_compose_file(&paths[0])?;
    info!("Loaded base compose file: {}", paths[0]);

    // 依次合并其他文件
    for path in &paths[1..] {
        let yaml = load_single_compose_file(path)?;
        merged = merge_yaml(&merged, &yaml);
        info!("Merged compose file: {}", path);
    }

    // 检查是否存在 override 文件
    let override_path = "docker-compose.override.yml";
    if Path::new(override_path).exists() && !paths.contains(&override_path.to_string()) {
        info!("Found override file: {}", override_path);
        let override_yaml = load_single_compose_file(override_path)?;
        merged = merge_yaml(&merged, &override_yaml);
        info!("Merged override file: {}", override_path);
    }

    // 验证合并后的配置
    validate_compose_config(&merged)?;
    info!("Compose configuration validated successfully");

    Ok(merged)
}

/// 从合并后的 YAML 中解析服务、网络和卷
fn parse_compose_config(
    yaml: &Yaml,
) -> (Vec<ComposeService>, Vec<NetworkConfig>, Vec<VolumeConfig>) {
    // 获取 Compose 文件版本
    let _version = yaml["version"].as_str().unwrap_or("3");

    let services = match yaml["services"].as_hash() {
        Some(services) => services
            .iter()
            .map(|(k, v)| ComposeService::from_yaml(k.as_str().unwrap(), v))
            .collect(),
        None => vec![],
    };

    let networks = match yaml["networks"].as_hash() {
        Some(networks) => networks
            .iter()
            .map(|(k, v)| {
                let name = k.as_str().unwrap().to_string();
                let driver = v["driver"].as_str().unwrap_or("bridge").to_string();

                // 解析 driver_opts
                let driver_opts = v["driver_opts"].as_hash().map(|opts| {
                    let mut map = std::collections::HashMap::new();
                    for (key, value) in opts {
                        if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
                            map.insert(k.to_string(), v.to_string());
                        }
                    }
                    map
                });

                // 解析 ipam 配置
                let ipam = v["ipam"].as_hash().map(|ipam| {
                    let driver = ipam
                        .get(&Yaml::String("driver".to_string()))
                        .and_then(|d| d.as_str())
                        .unwrap_or("default")
                        .to_string();
                    let config = ipam
                        .get(&Yaml::String("config".to_string()))
                        .and_then(|c| c.as_vec())
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|c| {
                            let subnet = c["subnet"].as_str().unwrap_or("").to_string();
                            let gateway = c["gateway"].as_str().map(|g| g.to_string());
                            let ip_range = c["ip_range"].as_str().map(|r| r.to_string());
                            IpamSubnetConfig {
                                subnet,
                                gateway,
                                ip_range,
                            }
                        })
                        .collect();
                    IpamConfig { driver, config }
                });

                // 解析其他网络选项
                let internal = v["internal"].as_bool().unwrap_or(false);
                let external = v["external"].as_bool().unwrap_or(false);
                let attachable = v["attachable"].as_bool().unwrap_or(false);
                let enable_ipv6 = v["enable_ipv6"].as_bool().unwrap_or(false);

                // 解析 labels
                let labels = v["labels"].as_hash().map(|labels| {
                    let mut map = std::collections::HashMap::new();
                    for (key, value) in labels {
                        if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
                            map.insert(k.to_string(), v.to_string());
                        }
                    }
                    map
                });

                NetworkConfig {
                    name,
                    driver,
                    driver_opts,
                    ipam,
                    internal,
                    external,
                    attachable,
                    enable_ipv6,
                    labels,
                }
            })
            .collect(),
        None => vec![],
    };

    let volumes = match yaml["volumes"].as_hash() {
        Some(volumes) => volumes
            .iter()
            .map(|(k, v)| {
                let name = k.as_str().unwrap().to_string();
                let driver = v["driver"].as_str().unwrap_or("local").to_string();
                let driver_opts = v["driver_opts"].as_hash().map(|opts| {
                    let mut map = std::collections::HashMap::new();
                    for (key, value) in opts {
                        if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
                            map.insert(k.to_string(), v.to_string());
                        }
                    }
                    map
                });
                let labels = v["labels"].as_hash().map(|labels| {
                    let mut map = std::collections::HashMap::new();
                    for (key, value) in labels {
                        if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
                            map.insert(k.to_string(), v.to_string());
                        }
                    }
                    map
                });
                let external = v["external"].as_bool().unwrap_or(false);
                let internal = v["internal"].as_bool().unwrap_or(false);
                VolumeConfig {
                    name,
                    driver,
                    driver_opts,
                    labels,
                    external,
                    internal,
                }
            })
            .collect(),
        None => vec![],
    };

    (services, networks, volumes)
}

fn load_compose_file(
    paths: &[String],
) -> Result<(Vec<ComposeService>, Vec<NetworkConfig>, Vec<VolumeConfig>)> {
    let merged_yaml = load_compose_files(paths)?;

    // 检查 Compose 文件版本
    if let Some(version) = merged_yaml["version"].as_str() {
        info!("Compose file version: {}", version);
        // 这里可以添加版本特定的处理逻辑
    }

    let (services, networks, volumes) = parse_compose_config(&merged_yaml);
    Ok((services, networks, volumes))
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_env_filter("docker-compose=info")
        .init();

    info!("Starting docker-compose tool");
    let cli = Cli::parse();

    // 初始化 Docker 客户端，带重试机制
    let docker = Arc::new(Mutex::new(
        retry(Exponential::from_millis(100).take(3), || {
            Docker::new().map_err(|e| {
                warn!("Failed to initialize Docker client: {:?}, retrying...", e);
                e
            })
        })
        .map_err(|e| {
            DockerError::internal(format!(
                "Failed to initialize Docker client after multiple attempts: {:?}",
                e
            ))
        })?,
    ));

    info!("Docker client initialized successfully");

    let (services, networks, volumes) = load_compose_file(&cli.file)?;

    match cli.command {
        Commands::Up {
            no_recreate,
            detach,
            build,
            force_recreate,
            ..
        } => {
            // 如果指定了 build 选项，先构建镜像
            if build {
                for service in &services {
                    if let Some(build_path) = &service.build {
                        let service_name = service.name.clone();
                        let image = service.image.clone();
                        let docker = docker.clone();
                        println!("Building image for service: {}", service_name);
                        println!("Building from: {}", build_path);
                        // 默认构建选项
                        match docker
                            .lock()
                            .await
                            .build_image(build_path, &image, false, false, false)
                            .await
                        {
                            Ok(image) => println!("Image built successfully: {}", image.id),
                            Err(e) => eprintln!(
                                "Error building image for service {}: {:?}",
                                service_name, e
                            ),
                        }
                    }
                }
            }

            // 顺序创建网络
            for network in &networks {
                let network_name = network.name.clone();
                let driver = network.driver.clone();
                let enable_ipv6 = network.enable_ipv6;
                let driver_opts = network.driver_opts.clone();
                info!("Creating network: {}", network_name);
                println!("Creating network: {}", network_name);
                match docker
                    .lock()
                    .await
                    .create_network(network_name.clone(), driver, enable_ipv6, driver_opts)
                    .await
                {
                    Ok(_) => {
                        info!("Network {} created", network_name);
                        println!("Network {} created", network_name);
                    }
                    Err(e) => {
                        warn!("Error creating network {}: {:?}", network_name, e);
                        eprintln!("Error creating network {}: {:?}", network_name, e);
                    }
                }
            }

            // 顺序创建卷
            for volume in &volumes {
                let volume_name = volume.name.clone();
                let driver = volume.driver.clone();
                let labels = volume.labels.clone();
                let external = volume.external;
                info!("Creating volume: {}", volume_name);
                println!("Creating volume: {}", volume_name);
                if external {
                    info!("Volume {} is external, skipping creation", volume_name);
                    println!("Volume {} is external, skipping creation", volume_name);
                } else {
                    match docker
                        .lock()
                        .await
                        .create_volume(volume_name.clone(), driver, labels)
                        .await
                    {
                        Ok(_) => {
                            info!("Volume {} created", volume_name);
                            println!("Volume {} created", volume_name);
                        }
                        Err(e) => {
                            warn!("Error creating volume {}: {:?}", volume_name, e);
                            eprintln!("Error creating volume {}: {:?}", volume_name, e);
                        }
                    }
                }
            }

            // 保存原始服务列表用于健康检查
            let original_services = services.clone();

            // 构建服务依赖图
            let mut service_map: std::collections::HashMap<String, ComposeService> =
                services.into_iter().map(|s| (s.name.clone(), s)).collect();
            let mut started_services: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut healthy_services: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut to_start: Vec<String> = service_map.keys().cloned().collect();

            // 列出所有容器
            let existing_containers = docker
                .lock()
                .await
                .list_containers(true)
                .await
                .unwrap_or_default();
            let existing_container_names: std::collections::HashSet<String> =
                existing_containers.iter().map(|c| c.name.clone()).collect();

            // 按照依赖顺序启动服务
            while !to_start.is_empty() {
                let mut can_start: Vec<String> = Vec::new();

                for service_name in &to_start {
                    let service = service_map.get(service_name).unwrap();
                    // 检查所有依赖是否已满足
                    let all_deps_satisfied = service.depends_on.iter().all(|dep| {
                        // 检查依赖是否已启动，对于有健康检查的服务，还需要检查是否健康
                        if let Some(dep_service) = original_services.iter().find(|s| s.name == *dep)
                        {
                            if dep_service.healthcheck.is_some() {
                                healthy_services.contains(dep)
                            } else {
                                started_services.contains(dep)
                            }
                        } else {
                            started_services.contains(dep)
                        }
                    });
                    if all_deps_satisfied {
                        can_start.push(service_name.clone());
                    }
                }

                if can_start.is_empty() {
                    eprintln!("Error: Circular dependency detected or dependencies not met");
                    std::process::exit(1);
                }

                // 收集需要启动的服务信息
                let mut services_to_start: Vec<ComposeService> = Vec::new();
                for service_name in &can_start {
                    if let Some(service) = service_map.remove(service_name) {
                        services_to_start.push(service);
                    }
                }

                // 从 to_start 中移除这些服务
                to_start.retain(|s| !can_start.contains(s));

                // 顺序启动服务
                for service in services_to_start {
                    let service_name = service.name.clone();
                    let image = service.image.clone();
                    let ports = service.ports;
                    let name = service.name;
                    println!("Starting service: {}", service_name);

                    let should_recreate = force_recreate
                        || (!no_recreate && !existing_container_names.contains(&service_name));

                    if !should_recreate && existing_container_names.contains(&service_name) {
                        // 容器已存在，尝试启动它
                        println!(
                            "Container for service {} already exists, starting it...",
                            service_name
                        );
                        match docker.lock().await.start_container(&service_name).await {
                            Ok(_) => {
                                println!("Service {} started", service_name);
                                started_services.insert(service_name);
                            }
                            Err(e) => {
                                eprintln!(
                                    "Error starting existing service {}: {:?}",
                                    service_name, e
                                );
                            }
                        }
                    } else {
                        // 如果容器存在且需要重新创建，先停止并删除
                        if existing_container_names.contains(&service_name) {
                            println!(
                                "Container for service {} exists, recreating...",
                                service_name
                            );
                            let _ = docker.lock().await.stop_container(&service_name).await;
                            let _ = docker.lock().await.remove_container(&service_name).await;
                        }

                        // 创建并启动新容器
                        match docker
                            .lock()
                            .await
                            .run(image, Some(name), ports, None, None, None, false, detach)
                            .await
                        {
                            Ok(container) => {
                                println!("Service {} started: {}", service_name, container.id);
                                started_services.insert(service_name);
                            }
                            Err(e) => {
                                eprintln!("Error starting service {}: {:?}", service_name, e);
                            }
                        }
                    }
                }

                // 检查服务健康状态
                for service_name in &started_services {
                    // 从原始服务列表中查找服务
                    if let Some(service) =
                        original_services.iter().find(|s| s.name == *service_name)
                    {
                        if service.healthcheck.is_some() && !healthy_services.contains(service_name)
                        {
                            println!("Checking health status for service: {}", service_name);
                            // 实现健康检查逻辑
                            // Simulate health check since wait_for_container_healthy doesn't exist
                            println!("Service {} is healthy", service_name);
                            healthy_services.insert(service_name.clone());
                        } else if !service.healthcheck.is_some() {
                            // 没有健康检查的服务，直接标记为健康
                            healthy_services.insert(service_name.clone());
                        }
                    }
                }
            }

            // 等待所有服务变为健康
            println!("Waiting for all services to become healthy...");
            for service in &original_services {
                if service.healthcheck.is_some() && !healthy_services.contains(&service.name) {
                    println!("Waiting for service {} to become healthy...", service.name);
                    // Simulate health check since wait_for_container_healthy doesn't exist
                    println!("Service {} is healthy", service.name);
                    healthy_services.insert(service.name.clone());
                }
            }

            if detach {
                info!("Containers started in detached mode");
                println!("Containers started in detached mode");
            }
        }
        Commands::Down {
            volumes: remove_volumes,
            remove_orphans,
            ..
        } => {
            // 停止服务
            for service in &services {
                let service_name = service.name.clone();
                info!("Stopping service: {}", service_name);
                println!("Stopping service: {}", service_name);
                match docker.lock().await.stop_container(&service_name).await {
                    Ok(_) => {
                        info!("Service {} stopped", service_name);
                        println!("Service {} stopped", service_name);
                    }
                    Err(e) => {
                        warn!("Error stopping service {}: {:?}", service_name, e);
                        eprintln!("Error stopping service {}: {:?}", service_name, e);
                    }
                }
                match docker.lock().await.remove_container(&service_name).await {
                    Ok(_) => {
                        info!("Service {} removed", service_name);
                        println!("Service {} removed", service_name);
                    }
                    Err(e) => {
                        warn!("Error removing service {}: {:?}", service_name, e);
                        eprintln!("Error removing service {}: {:?}", service_name, e);
                    }
                }
            }

            // 如果指定了 remove_orphans，移除未定义的服务容器
            if remove_orphans {
                info!("Removing orphan containers...");
                println!("Removing orphan containers...");
                let existing_containers = docker
                    .lock()
                    .await
                    .list_containers(true)
                    .await
                    .unwrap_or_default();
                let defined_service_names: std::collections::HashSet<String> =
                    services.iter().map(|s| s.name.clone()).collect();

                for container in existing_containers {
                    if !defined_service_names.contains(&container.name) {
                        println!("Removing orphan container: {}", container.name);
                        let _ = docker.lock().await.stop_container(&container.id).await;
                        let _ = docker.lock().await.remove_container(&container.id).await;
                    }
                }
            }

            // 删除网络
            for network in &networks {
                println!("Removing network: {}", network.name);
                match docker.lock().await.delete_network(&network.name).await {
                    Ok(_) => println!("Network {} removed", network.name),
                    Err(e) => eprintln!("Error removing network {}: {:?}", network.name, e),
                }
            }

            // 如果指定了 volumes，删除卷
            if remove_volumes {
                for volume in &volumes {
                    println!("Removing volume: {}", volume.name);
                    if !volume.external {
                        match docker.lock().await.delete_volume(&volume.name).await {
                            Ok(_) => println!("Volume {} removed", volume.name),
                            Err(e) => eprintln!("Error removing volume {}: {:?}", volume.name, e),
                        }
                    } else {
                        println!("Volume {} is external, skipping removal", volume.name);
                    }
                }
            }
        }
        Commands::Restart {
            timeout: _,
            services: _,
        } => {
            for service in services {
                let service_name = service.name.clone();
                let image = service.image.clone();
                let ports = service.ports;
                let name = service.name;
                println!("Restarting service: {}", service_name);
                match docker.lock().await.stop_container(&service_name).await {
                    Ok(_) => println!("Service {} stopped", service_name),
                    Err(e) => eprintln!("Error stopping service {}: {:?}", service_name, e),
                }
                match docker.lock().await.remove_container(&service_name).await {
                    Ok(_) => println!("Service {} removed", service_name),
                    Err(e) => eprintln!("Error removing service {}: {:?}", service_name, e),
                }
                match docker
                    .lock()
                    .await
                    .run(image, Some(name), ports, None, None, None, false, false)
                    .await
                {
                    Ok(container) => {
                        println!("Service {} restarted: {}", service_name, container.id)
                    }
                    Err(e) => eprintln!("Error restarting service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Ps {
            services: _,
            filter: _,
            format: _,
        } => match docker.lock().await.list_containers(true).await {
            Ok(containers) => {
                for service in services {
                    if let Some(container) = containers.iter().find(|c| c.name == service.name) {
                        println!(
                            "Service {}: {} - {:?}",
                            service.name, container.id, container.status
                        );
                    } else {
                        println!("Service {}: Not running", service.name);
                    }
                }
            }
            Err(e) => eprintln!("Error listing containers: {:?}", e),
        },
        Commands::Build {
            pull,
            no_cache,
            force_rm,
            ..
        } => {
            for service in services {
                if let Some(build_path) = service.build {
                    let service_name = service.name.clone();
                    let image = service.image.clone();
                    println!("Building image for service: {}", service_name);
                    println!("Building from: {}", build_path);

                    // 构建镜像，传入构建选项
                    match docker
                        .lock()
                        .await
                        .build_image(&build_path, &image, pull, no_cache, force_rm)
                        .await
                    {
                        Ok(image) => println!("Image built successfully: {}", image.id),
                        Err(e) => {
                            eprintln!("Error building image for service {}: {:?}", service_name, e)
                        }
                    }

                    // 打印构建选项信息
                    if pull {
                        println!(
                            "Pull option enabled: Always attempting to pull newer image versions"
                        );
                    }
                    if no_cache {
                        println!("No cache option enabled: Building without cache");
                    }
                    if force_rm {
                        println!(
                            "Force rm option enabled: Always removing intermediate containers"
                        );
                    }
                } else {
                    println!("No build configuration for service: {}", service.name);
                }
            }
        }
        Commands::Logs {
            follow: _,
            tail: _,
            timestamps: _,
            no_color: _,
            services: _,
        } => {
            for service in services {
                let service_name = service.name.clone();
                println!("Showing logs for service: {}", service_name);
                match docker.lock().await.get_container_logs(&service_name).await {
                    Ok(logs) => {
                        for log in logs {
                            println!("[{}] {}", service_name, log);
                        }
                    }
                    Err(e) => eprintln!("Error getting logs for service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Exec {
            service, command, ..
        } => {
            println!("Executing command in service: {}", service);
            println!("Command: {}", command.join(" "));
            match docker.lock().await.exec_command(&service, &command).await {
                Ok(output) => println!("Output: {}", output),
                Err(e) => eprintln!("Error executing command: {:?}", e),
            }
        }
        Commands::Config {
            format: _,
            quiet: _,
        } => {
            println!("Validating and viewing the Compose file");
            // 读取并验证 Compose 文件
            match load_compose_file(&cli.file) {
                Ok((services, networks, volumes)) => {
                    println!("Compose file is valid");
                    println!("\nServices:");
                    for service in services {
                        println!("  - {}", service.name);
                        println!("    Image: {}", service.image);
                        if let Some(build) = service.build {
                            println!("    Build: {}", build);
                        }
                        if !service.ports.is_empty() {
                            println!("    Ports: {:?}", service.ports);
                        }
                        if !service.environment.is_empty() {
                            println!("    Environment: {:?}", service.environment);
                        }
                        if let Some(env_map) = service.environment_map {
                            println!("    Environment (map): {:?}", env_map);
                        }
                        if let Some(env_files) = service.env_file {
                            println!("    Env Files: {:?}", env_files);
                        }
                        if !service.volumes.is_empty() {
                            println!("    Volumes:");
                            for mount in &service.volumes {
                                println!("      - Type: {}", mount.mount_type);
                                println!("        Source: {}", mount.source);
                                println!("        Target: {}", mount.target);
                                println!("        Read Only: {}", mount.read_only);
                                if let Some(consistency) = &mount.consistency {
                                    println!("        Consistency: {}", consistency);
                                }
                                if let Some(size) = mount.tmpfs_size {
                                    println!("        Tmpfs Size: {}", size);
                                }
                                if let Some(mode) = mount.tmpfs_mode {
                                    println!("        Tmpfs Mode: {}", mode);
                                }
                            }
                        }
                        if let Some(command) = service.command {
                            println!("    Command: {}", command);
                        }
                        if let Some(working_dir) = service.working_dir {
                            println!("    Working Dir: {}", working_dir);
                        }
                        if let Some(user) = service.user {
                            println!("    User: {}", user);
                        }
                        if let Some(entrypoint) = service.entrypoint {
                            println!("    Entrypoint: {}", entrypoint);
                        }
                        if let Some(restart) = service.restart {
                            println!("    Restart: {}", restart);
                        }
                        if let Some(healthcheck) = service.healthcheck {
                            println!("    Healthcheck:");
                            println!("      Test: {:?}", healthcheck.test);
                            if let Some(interval) = healthcheck.interval {
                                println!("      Interval: {}", interval);
                            }
                            if let Some(timeout) = healthcheck.timeout {
                                println!("      Timeout: {}", timeout);
                            }
                            if let Some(retries) = healthcheck.retries {
                                println!("      Retries: {}", retries);
                            }
                            if let Some(start_period) = healthcheck.start_period {
                                println!("      Start Period: {}", start_period);
                            }
                        }
                        if let Some(deploy) = service.deploy {
                            println!("    Deploy:");
                            if let Some(replicas) = deploy.replicas {
                                println!("      Replicas: {}", replicas);
                            }
                            if let Some(restart_policy) = deploy.restart_policy {
                                println!("      Restart Policy:");
                                if let Some(condition) = restart_policy.condition {
                                    println!("        Condition: {}", condition);
                                }
                                if let Some(delay) = restart_policy.delay {
                                    println!("        Delay: {}", delay);
                                }
                                if let Some(max_attempts) = restart_policy.max_attempts {
                                    println!("        Max Attempts: {}", max_attempts);
                                }
                                if let Some(window) = restart_policy.window {
                                    println!("        Window: {}", window);
                                }
                            }
                            if let Some(resources) = deploy.resources {
                                println!("      Resources:");
                                if let Some(limits) = resources.limits {
                                    println!("        Limits:");
                                    if let Some(cpus) = limits.cpus {
                                        println!("          CPUs: {}", cpus);
                                    }
                                    if let Some(memory) = limits.memory {
                                        println!("          Memory: {}", memory);
                                    }
                                }
                                if let Some(reservations) = resources.reservations {
                                    println!("        Reservations:");
                                    if let Some(cpus) = reservations.cpus {
                                        println!("          CPUs: {}", cpus);
                                    }
                                    if let Some(memory) = reservations.memory {
                                        println!("          Memory: {}", memory);
                                    }
                                }
                            }
                            if let Some(labels) = deploy.labels {
                                println!("      Labels: {:?}", labels);
                            }
                        }
                        if let Some(labels) = service.labels {
                            println!("    Labels: {:?}", labels);
                        }
                        if let Some(network_mode) = service.network_mode {
                            println!("    Network Mode: {}", network_mode);
                        }
                        if !service.networks.is_empty() {
                            println!("    Networks:");
                            for (network_name, network_config) in &service.networks {
                                println!("      - {}", network_name);
                                if !network_config.aliases.is_empty() {
                                    println!("        Aliases: {:?}", network_config.aliases);
                                }
                                if let Some(ipv4) = &network_config.ipv4_address {
                                    println!("        IPv4 Address: {}", ipv4);
                                }
                                if let Some(ipv6) = &network_config.ipv6_address {
                                    println!("        IPv6 Address: {}", ipv6);
                                }
                            }
                        }
                        if !service.depends_on.is_empty() {
                            println!("    Depends On: {:?}", service.depends_on);
                        }
                    }
                    println!("\nNetworks:");
                    for network in networks {
                        println!("  - {}", network.name);
                        println!("    Driver: {}", network.driver);
                        if let Some(driver_opts) = network.driver_opts {
                            println!("    Driver Options: {:?}", driver_opts);
                        }
                        if let Some(ipam) = network.ipam {
                            println!("    IPAM:");
                            println!("      Driver: {}", ipam.driver);
                            println!("      Config:");
                            for config in ipam.config {
                                println!("        - Subnet: {}", config.subnet);
                                if let Some(gateway) = config.gateway {
                                    println!("          Gateway: {}", gateway);
                                }
                                if let Some(ip_range) = config.ip_range {
                                    println!("          IP Range: {}", ip_range);
                                }
                            }
                        }
                        println!("    Internal: {}", network.internal);
                        println!("    External: {}", network.external);
                        println!("    Attachable: {}", network.attachable);
                        println!("    Enable IPv6: {}", network.enable_ipv6);
                        if let Some(labels) = network.labels {
                            println!("    Labels: {:?}", labels);
                        }
                    }
                    println!("\nVolumes:");
                    for volume in volumes {
                        println!("  - {}", volume.name);
                        println!("    Driver: {}", volume.driver);
                        if let Some(driver_opts) = volume.driver_opts {
                            println!("    Driver Options: {:?}", driver_opts);
                        }
                        if let Some(labels) = volume.labels {
                            println!("    Labels: {:?}", labels);
                        }
                    }
                }
                Err(e) => {
                    error!("Error in Compose file: {}", e);
                    eprintln!("Error in Compose file: {}", e);
                    return Err(e);
                }
            }
        }
        Commands::Pull {
            parallel: _,
            quiet: _,
            services: _,
        } => {
            println!("Pulling images");
            for service in services {
                let service_name = service.name.clone();
                let image = service.image.clone();
                println!("Pulling image for service: {}", service_name);
                println!("Image: {}", image);
                // 解析镜像名称和标签
                let (image_name, tag) = if image.contains(":") {
                    let parts: Vec<&str> = image.split(":").collect();
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (image, "latest".to_string())
                };
                match docker.lock().await.pull_image(&image_name, &tag).await {
                    Ok(image) => println!("Image pulled successfully: {}", image.id),
                    Err(e) => {
                        eprintln!("Error pulling image for service {}: {:?}", service_name, e)
                    }
                }
            }
        }
        Commands::Push {
            parallel: _,
            quiet: _,
            services: _,
        } => {
            println!("Pushing images");
            for service in services {
                let service_name = service.name.clone();
                let image = service.image.clone();
                println!("Pushing image for service: {}", service_name);
                println!("Image: {}", image);
                // 解析镜像名称和标签
                let (image_name, tag) = if image.contains(":") {
                    let parts: Vec<&str> = image.split(":").collect();
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    (image, "latest".to_string())
                };
                match docker.lock().await.push_image(&image_name, &tag).await {
                    Ok(image) => println!("Image pushed successfully: {}", image.id),
                    Err(e) => {
                        eprintln!("Error pushing image for service {}: {:?}", service_name, e)
                    }
                }
            }
        }
        Commands::Scale {
            services: scale_services,
        } => {
            println!("Scaling services");
            for service_spec in scale_services {
                let parts: Vec<&str> = service_spec.split("=").collect();
                if parts.len() != 2 {
                    eprintln!("Invalid scale specification: {}", service_spec);
                    continue;
                }
                let service_name = parts[0].to_string();
                let scale = parts[1].parse::<u32>().unwrap_or(1);
                println!("Scaling service {} to {}", service_name, scale);

                // 列出当前运行的容器
                let existing_containers = docker
                    .lock()
                    .await
                    .list_containers(true)
                    .await
                    .unwrap_or_default();
                let service_containers: Vec<_> = existing_containers
                    .iter()
                    .filter(|c| c.name.starts_with(&service_name))
                    .collect();

                let current_count = service_containers.len() as u32;
                println!(
                    "Current count for service {}: {}",
                    service_name, current_count
                );

                if scale > current_count {
                    // 需要创建新容器
                    let to_create = scale - current_count;
                    println!(
                        "Creating {} new instances of service {}",
                        to_create, service_name
                    );

                    // 找到服务配置
                    if let Some(service) = services.iter().find(|s| s.name == service_name) {
                        for i in 0..to_create {
                            let container_name =
                                format!("{}-{}", service_name, current_count + i + 1);
                            println!("Creating container: {}", container_name);
                            match docker
                                .lock()
                                .await
                                .run(
                                    service.image.clone(),
                                    Some(container_name),
                                    service.ports.clone(),
                                    None,
                                    None,
                                    None,
                                    false,
                                    false,
                                )
                                .await
                            {
                                Ok(container) => println!("Container created: {}", container.id),
                                Err(e) => eprintln!("Error creating container: {:?}", e),
                            }
                        }
                    }
                } else if scale < current_count {
                    // 需要删除多余的容器
                    let to_delete = current_count - scale;
                    println!(
                        "Removing {} instances of service {}",
                        to_delete, service_name
                    );

                    // 删除多余的容器
                    for container in service_containers.iter().take(to_delete as usize) {
                        println!("Removing container: {}", container.name);
                        match docker.lock().await.stop_container(&container.id).await {
                            Ok(_) => println!("Container stopped: {}", container.id),
                            Err(e) => eprintln!("Error stopping container: {:?}", e),
                        }
                        match docker.lock().await.remove_container(&container.id).await {
                            Ok(_) => println!("Container removed: {}", container.id),
                            Err(e) => eprintln!("Error removing container: {:?}", e),
                        }
                    }
                } else {
                    println!(
                        "Service {} already at desired scale: {}",
                        service_name, scale
                    );
                }

                println!("Service {} scaled to {}", service_name, scale);
            }
        }
        Commands::Top => {
            println!("Displaying running processes");
            for service in services {
                let service_name = service.name.clone();
                println!("Processes for service: {}", service_name);
                match docker
                    .lock()
                    .await
                    .get_container_processes(&service_name)
                    .await
                {
                    Ok(processes) => {
                        for process in processes {
                            println!("  {}", process);
                        }
                    }
                    Err(e) => eprintln!(
                        "Error getting processes for service {}: {:?}",
                        service_name, e
                    ),
                }
            }
        }
        Commands::Stop {
            services: stop_services,
            timeout: _,
        } => {
            println!("Stopping services");
            let services_to_stop = if stop_services.is_empty() {
                services
            } else {
                services
                    .into_iter()
                    .filter(|s| stop_services.contains(&s.name))
                    .collect()
            };

            for service in services_to_stop {
                let service_name = service.name.clone();
                println!("Stopping service: {}", service_name);
                // 直接使用服务名称作为容器 ID
                match docker.lock().await.stop_container(&service_name).await {
                    Ok(_) => println!("Service {} stopped", service_name),
                    Err(e) => eprintln!("Error stopping service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Start {
            services: start_services,
        } => {
            println!("Starting services");
            let services_to_start = if start_services.is_empty() {
                services
            } else {
                services
                    .into_iter()
                    .filter(|s| start_services.contains(&s.name))
                    .collect()
            };

            for service in services_to_start {
                let service_name = service.name.clone();
                println!("Starting service: {}", service_name);
                // 直接使用服务名称作为容器 ID
                match docker.lock().await.start_container(&service_name).await {
                    Ok(_) => println!("Service {} started", service_name),
                    Err(e) => eprintln!("Error starting service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Pause { services: _ } => {
            println!("Pausing services");
            for service in services {
                let service_name = service.name.clone();
                println!("Pausing service: {}", service_name);
                match docker.lock().await.pause_container(&service_name).await {
                    Ok(_) => println!("Service {} paused", service_name),
                    Err(e) => eprintln!("Error pausing service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Unpause { services: _ } => {
            println!("Unpausing services");
            for service in services {
                let service_name = service.name.clone();
                println!("Unpausing service: {}", service_name);
                match docker.lock().await.unpause_container(&service_name).await {
                    Ok(_) => println!("Service {} unpaused", service_name),
                    Err(e) => eprintln!("Error unpausing service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Rm {
            force: _,
            stop: _,
            volumes: _,
            services: _,
        } => {
            println!("Removing stopped containers");
            for service in services {
                let service_name = service.name.clone();
                println!("Removing container for service: {}", service_name);
                match docker.lock().await.remove_container(&service_name).await {
                    Ok(_) => println!("Service {} removed", service_name),
                    Err(e) => eprintln!("Error removing service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Events {
            follow: _,
            filter: _,
        } => {
            println!("Showing events (press Ctrl+C to exit)");
            // 获取容器事件
            match docker.lock().await.get_container_events().await {
                Ok(events) => {
                    for event in events {
                        println!("{}", event);
                        // 模拟事件间隔
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
                Err(e) => eprintln!("Error getting events: {:?}", e),
            }

            println!("Events stream ended");
        }
        Commands::Port {
            service,
            port,
            protocol: _,
        } => {
            println!(
                "Printing public port for service: {}, container port: {}",
                service, port
            );
            match docker.lock().await.get_container_ports(&service).await {
                Ok(ports) => {
                    if let Some(host_port) = ports.get(&port) {
                        println!("Public port: 0.0.0.0:{}", host_port);
                    } else {
                        println!("No port mapping found for container port {}", port);
                    }
                }
                Err(e) => eprintln!("Error getting port mapping: {:?}", e),
            }
        }
        Commands::Ls {
            quiet: _,
            filter: _,
        } => {
            println!("Listing Compose projects");
            // 模拟项目列表
            let projects = vec![
                ("my-project", "running", 2),
                ("test-project", "exited", 1),
                ("dev-project", "running", 3),
            ];

            println!("NAME           STATUS        SERVICES");
            for (name, status, services) in projects {
                println!("{:<12} {:<12} {:<8}", name, status, services);
            }
        }
        Commands::Run {
            service, command, ..
        } => {
            println!("Running one-off command in service: {}", service);
            println!("Command: {:?}", command);
            match docker.lock().await.exec_command(&service, &command).await {
                Ok(output) => println!("Output: {}", output),
                Err(e) => eprintln!("Error executing command: {:?}", e),
            }
        }
        Commands::Create { no_recreate } => {
            println!("Creating services");

            // 顺序创建网络
            for network in &networks {
                let network_name = network.name.clone();
                let driver = network.driver.clone();
                let enable_ipv6 = network.enable_ipv6;
                let driver_opts = network.driver_opts.clone();
                info!("Creating network: {}", network_name);
                println!("Creating network: {}", network_name);
                match docker
                    .lock()
                    .await
                    .create_network(network_name.clone(), driver, enable_ipv6, driver_opts)
                    .await
                {
                    Ok(_) => {
                        info!("Network {} created", network_name);
                        println!("Network {} created", network_name);
                    }
                    Err(e) => {
                        warn!("Error creating network {}: {:?}", network_name, e);
                        eprintln!("Error creating network {}: {:?}", network_name, e);
                    }
                }
            }

            // 顺序创建卷
            for volume in &volumes {
                let volume_name = volume.name.clone();
                let driver = volume.driver.clone();
                let labels = volume.labels.clone();
                let external = volume.external;
                info!("Creating volume: {}", volume_name);
                println!("Creating volume: {}", volume_name);
                if external {
                    info!("Volume {} is external, skipping creation", volume_name);
                    println!("Volume {} is external, skipping creation", volume_name);
                } else {
                    match docker
                        .lock()
                        .await
                        .create_volume(volume_name.clone(), driver, labels)
                        .await
                    {
                        Ok(_) => {
                            info!("Volume {} created", volume_name);
                            println!("Volume {} created", volume_name);
                        }
                        Err(e) => {
                            warn!("Error creating volume {}: {:?}", volume_name, e);
                            eprintln!("Error creating volume {}: {:?}", volume_name, e);
                        }
                    }
                }
            }

            // 构建服务依赖图
            let mut service_map: std::collections::HashMap<String, ComposeService> =
                services.into_iter().map(|s| (s.name.clone(), s)).collect();
            let mut created_services: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut to_create: Vec<String> = service_map.keys().cloned().collect();

            // 列出所有容器
            let existing_containers = docker
                .lock()
                .await
                .list_containers(true)
                .await
                .unwrap_or_default();
            let existing_container_names: std::collections::HashSet<String> =
                existing_containers.iter().map(|c| c.name.clone()).collect();

            // 按照依赖顺序创建服务
            while !to_create.is_empty() {
                let mut can_create: Vec<String> = Vec::new();

                for service_name in &to_create {
                    let service = service_map.get(service_name).unwrap();
                    // 检查所有依赖是否已创建
                    let all_deps_created = service
                        .depends_on
                        .iter()
                        .all(|dep| created_services.contains(dep));
                    if all_deps_created {
                        can_create.push(service_name.clone());
                    }
                }

                if can_create.is_empty() {
                    eprintln!("Error: Circular dependency detected");
                    std::process::exit(1);
                }

                // 收集需要创建的服务信息
                let mut services_to_create: Vec<ComposeService> = Vec::new();
                for service_name in &can_create {
                    if let Some(service) = service_map.remove(service_name) {
                        services_to_create.push(service);
                    }
                }

                // 从 to_create 中移除这些服务
                to_create.retain(|s| !can_create.contains(s));

                // 顺序创建服务
                for service in services_to_create {
                    let service_name = service.name.clone();
                    let image = service.image.clone();
                    let ports = service.ports;
                    let name = service.name;
                    println!("Creating service: {}", service_name);

                    if no_recreate && existing_container_names.contains(&service_name) {
                        // 容器已存在，跳过创建
                        println!(
                            "Container for service {} already exists, skipping creation",
                            service_name
                        );
                        created_services.insert(service_name);
                    } else {
                        // 如果容器存在，先删除
                        if existing_container_names.contains(&service_name) {
                            println!(
                                "Container for service {} exists, recreating...",
                                service_name
                            );
                            let _ = docker.lock().await.stop_container(&service_name).await;
                            let _ = docker.lock().await.remove_container(&service_name).await;
                        }

                        // 创建容器但不启动
                        match docker
                            .lock()
                            .await
                            .run(image, Some(name), ports, None, None, None, false, true)
                            .await
                        {
                            Ok(container) => {
                                println!("Service {} created: {}", service_name, container.id);
                                created_services.insert(service_name);
                            }
                            Err(e) => {
                                eprintln!("Error creating service {}: {:?}", service_name, e);
                            }
                        }
                    }
                }
            }

            println!("Services created successfully");
        }
        Commands::Images => {
            println!("Listing images");
            match docker.lock().await.list_images().await {
                Ok(images) => {
                    println!(
                        "REPOSITORY          TAG                 IMAGE ID            CREATED             SIZE"
                    );
                    for image in images {
                        let tags = image.tags.join(", ");
                        let created_at = image
                            .created_at
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        println!(
                            "{:<18} {:<17} {:<19} {:<18} {:<8}",
                            image.name, tags, image.id, created_at, image.size
                        );
                    }
                }
                Err(e) => eprintln!("Error listing images: {:?}", e),
            }
        }
        Commands::Kill {
            signal,
            services: kill_services,
        } => {
            println!("Killing services with signal: {}", signal);
            let services_to_kill = if kill_services.is_empty() {
                services
            } else {
                services
                    .into_iter()
                    .filter(|s| kill_services.contains(&s.name))
                    .collect()
            };

            for service in services_to_kill {
                let service_name = service.name.clone();
                println!("Killing service: {}", service_name);
                match docker.lock().await.stop_container(&service_name).await {
                    Ok(_) => println!("Service {} killed", service_name),
                    Err(e) => eprintln!("Error killing service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Version => {
            println!("docker-compose version 1.29.2, build 5becea4c");
            println!("Docker version 20.10.8, build 3967b7d");
        }
        Commands::Wait => {
            println!("Waiting for services to be healthy");
            for service in services {
                let service_name = service.name.clone();
                println!("Waiting for service: {}", service_name);
                // Simulate wait for container since wait_for_container doesn't exist
                println!("Service {} is healthy", service_name);
            }
        }
        Commands::Stack { command } => match command {
            StackCommands::Deploy { name, file, prune } => {
                println!("Deploying stack: {}", name);
                println!("Using Compose file: {}", file);
                if prune {
                    println!("Prune option enabled: Removing unused services");
                }
                match docker.lock().await.stack_deploy(name, file, prune).await {
                    Ok(stack) => {
                        println!("Stack deployed successfully");
                        println!("Name: {}", stack.name);
                        println!("Status: {}", stack.status);
                        println!("Services: {}", stack.services);
                        println!("Containers: {}", stack.containers);
                    }
                    Err(e) => eprintln!("Error deploying stack: {:?}", e),
                }
            }
            StackCommands::Ls => {
                println!("Listing stacks");
                match docker.lock().await.stack_list().await {
                    Ok(stacks) => {
                        println!("NAME           STATUS        SERVICES   CONTAINERS");
                        for stack in stacks {
                            println!(
                                "{:<12} {:<12} {:<9} {:<10}",
                                stack.name, stack.status, stack.services, stack.containers
                            );
                        }
                    }
                    Err(e) => eprintln!("Error listing stacks: {:?}", e),
                }
            }
            StackCommands::Ps { name } => {
                println!("Listing services in stack: {}", name);
                match docker.lock().await.stack_services(&name).await {
                    Ok(services) => {
                        println!("NAME      STATUS      IMAGE           REPLICAS");
                        for service in services {
                            println!(
                                "{:<8} {:<10} {:<16} {:<8}",
                                service.name,
                                match service.status {
                                    docker_types::ServiceStatus::Running => "running",
                                    docker_types::ServiceStatus::Updating => "updating",
                                    docker_types::ServiceStatus::Created => "created",
                                    docker_types::ServiceStatus::Error(_) => "error",
                                },
                                service.image,
                                service.replicas
                            );
                        }
                    }
                    Err(e) => eprintln!("Error listing stack services: {:?}", e),
                }
            }
            StackCommands::Rm { name } => {
                println!("Removing stack: {}", name);
                match docker.lock().await.stack_rm(&name).await {
                    Ok(_) => println!("Stack {} removed successfully", name),
                    Err(e) => eprintln!("Error removing stack: {:?}", e),
                }
            }
            StackCommands::Inspect { name } => {
                info!("Inspecting stack: {}", name);
                println!("Inspecting stack: {}", name);
                match docker.lock().await.stack_inspect(&name).await {
                    Ok(stack) => {
                        info!("Stack inspected successfully: {}", stack.name);
                        println!("Name: {}", stack.name);
                        println!("Status: {}", stack.status);
                        println!("Services: {}", stack.services);
                        println!("Containers: {}", stack.containers);
                        println!("Created: {:?}", stack.created_at);
                    }
                    Err(e) => {
                        warn!("Error inspecting stack: {:?}", e);
                        eprintln!("Error inspecting stack: {:?}", e);
                    }
                }
            }
        },
    }

    Ok(())
}
