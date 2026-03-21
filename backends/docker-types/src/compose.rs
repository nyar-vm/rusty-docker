use std::{collections::HashMap, fs::File, io::Read, path::Path};

use crate::{DockerError, Result};
use serde_yaml;

/// 加载 .env 文件中的环境变量
pub fn load_env_file() -> HashMap<String, String> {
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
pub fn replace_env_vars(s: &str, env_vars: &HashMap<String, String>) -> String {
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

/// 加载单个 Compose 文件
pub fn load_single_compose_file(path: &str) -> Result<serde_yaml::Value> {
    // 加载环境变量
    let env_vars = load_env_file();

    // 检查文件是否存在
    if !Path::new(path).exists() {
        return Err(DockerError::io_error("open file", format!("File not found: {}", path)));
    }

    // 读取文件内容
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // 替换环境变量
    let content = replace_env_vars(&content, &env_vars);

    // 使用 serde-yaml 解析 YAML
    match serde_yaml::from_str(&content) {
        Ok(root) => Ok(root),
        Err(e) => Err(DockerError::invalid_params("compose_file", format!("Failed to parse compose file: {}", e))),
    }
}

/// 合并多个 Compose 文件
pub fn merge_compose_files(paths: &[String]) -> Result<serde_yaml::Value> {
    if paths.is_empty() {
        return Err(DockerError::invalid_params("compose_files", "No compose files specified"));
    }

    // 加载第一个文件作为基础
    let mut merged = load_single_compose_file(&paths[0])?;

    // 依次合并其他文件
    for path in &paths[1..] {
        let current = load_single_compose_file(path)?;
        merged = merge_yaml(merged, current);
    }

    // 检查是否存在 override 文件
    let override_path = "docker-compose.override.yml";
    if Path::new(override_path).exists() && !paths.contains(&override_path.to_string()) {
        let override_content = load_single_compose_file(override_path)?;
        merged = merge_yaml(merged, override_content);
    }

    Ok(merged)
}

/// 合并两个 YAML 值
fn merge_yaml(a: serde_yaml::Value, b: serde_yaml::Value) -> serde_yaml::Value {
    match (a, b) {
        (serde_yaml::Value::Mapping(mut a_map), serde_yaml::Value::Mapping(b_map)) => {
            for (k, v) in b_map {
                if let Some(existing) = a_map.get(&k) {
                    a_map.insert(k, merge_yaml(existing.clone(), v));
                }
                else {
                    a_map.insert(k, v);
                }
            }
            serde_yaml::Value::Mapping(a_map)
        }
        (_, b) => b,
    }
}

/// 验证 Compose 配置文件
pub fn validate_compose_config(config: &serde_yaml::Value) -> Result<()> {
    // 检查是否包含 services 部分
    if !config.get("services").is_some() {
        return Err(DockerError::invalid_params("compose_config", "No services defined in compose file"));
    }

    Ok(())
}

/// 解析 Compose 配置文件中的服务
pub fn parse_services(config: &serde_yaml::Value) -> Result<Vec<ComposeService>> {
    let mut services = Vec::new();

    if let Some(services_map) = config.get("services").and_then(|v| v.as_mapping()) {
        for (name, service_value) in services_map {
            if let Some(name_str) = name.as_str() {
                let service = parse_service(name_str, service_value)?;
                services.push(service);
            }
        }
    }

    Ok(services)
}

/// 解析单个服务配置
pub fn parse_service(name: &str, value: &serde_yaml::Value) -> Result<ComposeService> {
    let mut service = ComposeService::default(name);

    // 解析镜像
    if let Some(image) = value.get("image").and_then(|v| v.as_str()) {
        service.image = image.to_string();
    }

    // 解析构建配置
    if let Some(build) = value.get("build") {
        if let Some(build_str) = build.as_str() {
            service.build = Some(build_str.to_string());
        }
    }

    // 解析端口
    if let Some(ports) = value.get("ports").and_then(|v| v.as_sequence()) {
        for port in ports {
            if let Some(port_str) = port.as_str() {
                service.ports.push(port_str.to_string());
            }
        }
    }

    // 解析环境变量
    if let Some(env) = value.get("environment") {
        if let Some(env_seq) = env.as_sequence() {
            for env_var in env_seq {
                if let Some(env_str) = env_var.as_str() {
                    service.environment.push(env_str.to_string());
                }
            }
        }
        else if let Some(env_map) = env.as_mapping() {
            let mut env_map_hash = HashMap::new();
            for (k, v) in env_map {
                if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                    env_map_hash.insert(key.to_string(), val.to_string());
                }
            }
            service.environment_map = Some(env_map_hash);
        }
    }

    // 解析环境变量文件
    if let Some(env_file) = value.get("env_file") {
        if let Some(env_file_seq) = env_file.as_sequence() {
            let mut env_files = Vec::new();
            for file in env_file_seq {
                if let Some(file_str) = file.as_str() {
                    env_files.push(file_str.to_string());
                }
            }
            service.env_file = Some(env_files);
        }
        else if let Some(file_str) = env_file.as_str() {
            service.env_file = Some(vec![file_str.to_string()]);
        }
    }

    // 解析卷
    if let Some(volumes) = value.get("volumes").and_then(|v| v.as_sequence()) {
        for volume in volumes {
            if let Some(volume_str) = volume.as_str() {
                // 解析卷格式：source:target:mode
                let parts: Vec<&str> = volume_str.split(':').collect();
                if parts.len() >= 2 {
                    let source = parts[0].to_string();
                    let target = parts[1].to_string();
                    let read_only = parts.len() >= 3 && parts[2] == "ro";
                    let mount_type = if source.starts_with("/") { "bind" } else { "volume" };

                    service.volumes.push(MountConfig {
                        source,
                        target,
                        read_only,
                        mount_type: mount_type.to_string(),
                        consistency: None,
                        tmpfs_size: None,
                        tmpfs_mode: None,
                    });
                }
            }
        }
    }

    // 解析命令
    if let Some(command) = value.get("command").and_then(|v| v.as_str()) {
        service.command = Some(command.to_string());
    }

    // 解析工作目录
    if let Some(working_dir) = value.get("working_dir").and_then(|v| v.as_str()) {
        service.working_dir = Some(working_dir.to_string());
    }

    // 解析用户
    if let Some(user) = value.get("user").and_then(|v| v.as_str()) {
        service.user = Some(user.to_string());
    }

    // 解析入口点
    if let Some(entrypoint) = value.get("entrypoint").and_then(|v| v.as_str()) {
        service.entrypoint = Some(entrypoint.to_string());
    }

    // 解析重启策略
    if let Some(restart) = value.get("restart").and_then(|v| v.as_str()) {
        service.restart = Some(restart.to_string());
    }

    // 解析健康检查
    if let Some(healthcheck) = value.get("healthcheck") {
        let mut healthcheck_config =
            HealthCheckConfig { test: Vec::new(), interval: None, timeout: None, retries: None, start_period: None };

        if let Some(test) = healthcheck.get("test").and_then(|v| v.as_sequence()) {
            for test_cmd in test {
                if let Some(test_str) = test_cmd.as_str() {
                    healthcheck_config.test.push(test_str.to_string());
                }
            }
        }

        if let Some(interval) = healthcheck.get("interval").and_then(|v| v.as_str()) {
            healthcheck_config.interval = Some(interval.to_string());
        }

        if let Some(timeout) = healthcheck.get("timeout").and_then(|v| v.as_str()) {
            healthcheck_config.timeout = Some(timeout.to_string());
        }

        if let Some(retries) = healthcheck.get("retries").and_then(|v| v.as_u64()) {
            healthcheck_config.retries = Some(retries as u32);
        }

        if let Some(start_period) = healthcheck.get("start_period").and_then(|v| v.as_str()) {
            healthcheck_config.start_period = Some(start_period.to_string());
        }

        service.healthcheck = Some(healthcheck_config);
    }

    // 解析部署配置
    if let Some(deploy) = value.get("deploy") {
        let mut deploy_config = DeployConfig { replicas: None, restart_policy: None, resources: None, labels: None };

        if let Some(replicas) = deploy.get("replicas").and_then(|v| v.as_u64()) {
            deploy_config.replicas = Some(replicas as u32);
        }

        if let Some(restart_policy) = deploy.get("restart_policy") {
            let mut restart_policy_config =
                RestartPolicyConfig { condition: None, delay: None, max_attempts: None, window: None };

            if let Some(condition) = restart_policy.get("condition").and_then(|v| v.as_str()) {
                restart_policy_config.condition = Some(condition.to_string());
            }

            if let Some(delay) = restart_policy.get("delay").and_then(|v| v.as_str()) {
                restart_policy_config.delay = Some(delay.to_string());
            }

            if let Some(max_attempts) = restart_policy.get("max_attempts").and_then(|v| v.as_u64()) {
                restart_policy_config.max_attempts = Some(max_attempts as u32);
            }

            if let Some(window) = restart_policy.get("window").and_then(|v| v.as_str()) {
                restart_policy_config.window = Some(window.to_string());
            }

            deploy_config.restart_policy = Some(restart_policy_config);
        }

        if let Some(resources) = deploy.get("resources") {
            let mut resources_config = ResourcesConfig { limits: None, reservations: None };

            if let Some(limits) = resources.get("limits") {
                let mut limits_config = ResourceLimits { cpus: None, memory: None };

                if let Some(cpus) = limits.get("cpus").and_then(|v| v.as_str()) {
                    limits_config.cpus = Some(cpus.to_string());
                }

                if let Some(memory) = limits.get("memory").and_then(|v| v.as_str()) {
                    limits_config.memory = Some(memory.to_string());
                }

                resources_config.limits = Some(limits_config);
            }

            if let Some(reservations) = resources.get("reservations") {
                let mut reservations_config = ResourceReservations { cpus: None, memory: None };

                if let Some(cpus) = reservations.get("cpus").and_then(|v| v.as_str()) {
                    reservations_config.cpus = Some(cpus.to_string());
                }

                if let Some(memory) = reservations.get("memory").and_then(|v| v.as_str()) {
                    reservations_config.memory = Some(memory.to_string());
                }

                resources_config.reservations = Some(reservations_config);
            }

            deploy_config.resources = Some(resources_config);
        }

        if let Some(labels) = deploy.get("labels").and_then(|v| v.as_sequence()) {
            let mut labels_vec = Vec::new();
            for label in labels {
                if let Some(label_str) = label.as_str() {
                    labels_vec.push(label_str.to_string());
                }
            }
            deploy_config.labels = Some(labels_vec);
        }

        service.deploy = Some(deploy_config);
    }

    // 解析标签
    if let Some(labels) = value.get("labels").and_then(|v| v.as_sequence()) {
        let mut labels_vec = Vec::new();
        for label in labels {
            if let Some(label_str) = label.as_str() {
                labels_vec.push(label_str.to_string());
            }
        }
        service.labels = Some(labels_vec);
    }

    // 解析网络模式
    if let Some(network_mode) = value.get("network_mode").and_then(|v| v.as_str()) {
        service.network_mode = Some(network_mode.to_string());
    }

    // 解析网络
    if let Some(networks) = value.get("networks").and_then(|v| v.as_mapping()) {
        for (network_name, network_config) in networks {
            if let Some(network_name_str) = network_name.as_str() {
                let mut network_service_config =
                    NetworkServiceConfig { aliases: Vec::new(), ipv4_address: None, ipv6_address: None };

                if let Some(config_map) = network_config.as_mapping() {
                    if let Some(aliases) = config_map.get("aliases").and_then(|v| v.as_sequence()) {
                        for alias in aliases {
                            if let Some(alias_str) = alias.as_str() {
                                network_service_config.aliases.push(alias_str.to_string());
                            }
                        }
                    }

                    if let Some(ipv4) = config_map.get("ipv4_address").and_then(|v| v.as_str()) {
                        network_service_config.ipv4_address = Some(ipv4.to_string());
                    }

                    if let Some(ipv6) = config_map.get("ipv6_address").and_then(|v| v.as_str()) {
                        network_service_config.ipv6_address = Some(ipv6.to_string());
                    }
                }

                service.networks.insert(network_name_str.to_string(), network_service_config);
            }
        }
    }

    // 解析依赖关系
    if let Some(depends_on) = value.get("depends_on").and_then(|v| v.as_sequence()) {
        for dep in depends_on {
            if let Some(dep_str) = dep.as_str() {
                service.depends_on.push(dep_str.to_string());
            }
        }
    }

    Ok(service)
}

/// 解析 Compose 配置文件中的网络
pub fn parse_networks(config: &serde_yaml::Value) -> Vec<NetworkConfig> {
    let mut networks = Vec::new();

    if let Some(networks_map) = config.get("networks").and_then(|v| v.as_mapping()) {
        for (name, network_value) in networks_map {
            if let Some(name_str) = name.as_str() {
                let mut network = NetworkConfig {
                    name: name_str.to_string(),
                    driver: "bridge".to_string(),
                    driver_opts: None,
                    ipam: None,
                    internal: false,
                    external: false,
                    attachable: false,
                    enable_ipv6: false,
                    labels: None,
                };

                if let Some(driver) = network_value.get("driver").and_then(|v| v.as_str()) {
                    network.driver = driver.to_string();
                }

                if let Some(driver_opts) = network_value.get("driver_opts").and_then(|v| v.as_mapping()) {
                    let mut opts = HashMap::new();
                    for (k, v) in driver_opts {
                        if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                            opts.insert(key.to_string(), val.to_string());
                        }
                    }
                    network.driver_opts = Some(opts);
                }

                if let Some(ipam) = network_value.get("ipam") {
                    let mut ipam_config = IpamConfig { driver: "default".to_string(), config: Vec::new() };

                    if let Some(driver) = ipam.get("driver").and_then(|v| v.as_str()) {
                        ipam_config.driver = driver.to_string();
                    }

                    if let Some(configs) = ipam.get("config").and_then(|v| v.as_sequence()) {
                        for config in configs {
                            if let Some(config_map) = config.as_mapping() {
                                let mut subnet_config =
                                    IpamSubnetConfig { subnet: "".to_string(), gateway: None, ip_range: None };

                                if let Some(subnet) = config_map.get("subnet").and_then(|v| v.as_str()) {
                                    subnet_config.subnet = subnet.to_string();
                                }

                                if let Some(gateway) = config_map.get("gateway").and_then(|v| v.as_str()) {
                                    subnet_config.gateway = Some(gateway.to_string());
                                }

                                if let Some(ip_range) = config_map.get("ip_range").and_then(|v| v.as_str()) {
                                    subnet_config.ip_range = Some(ip_range.to_string());
                                }

                                ipam_config.config.push(subnet_config);
                            }
                        }
                    }

                    network.ipam = Some(ipam_config);
                }

                if let Some(internal) = network_value.get("internal").and_then(|v| v.as_bool()) {
                    network.internal = internal;
                }

                if let Some(external) = network_value.get("external").and_then(|v| v.as_bool()) {
                    network.external = external;
                }

                if let Some(attachable) = network_value.get("attachable").and_then(|v| v.as_bool()) {
                    network.attachable = attachable;
                }

                if let Some(enable_ipv6) = network_value.get("enable_ipv6").and_then(|v| v.as_bool()) {
                    network.enable_ipv6 = enable_ipv6;
                }

                if let Some(labels) = network_value.get("labels").and_then(|v| v.as_mapping()) {
                    let mut labels_map = HashMap::new();
                    for (k, v) in labels {
                        if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                            labels_map.insert(key.to_string(), val.to_string());
                        }
                    }
                    network.labels = Some(labels_map);
                }

                networks.push(network);
            }
        }
    }

    // 如果没有定义网络，添加默认网络
    if networks.is_empty() {
        networks.push(NetworkConfig {
            name: "default".to_string(),
            driver: "bridge".to_string(),
            driver_opts: None,
            ipam: None,
            internal: false,
            external: false,
            attachable: false,
            enable_ipv6: false,
            labels: None,
        });
    }

    networks
}

/// 解析 Compose 配置文件中的卷
pub fn parse_volumes(config: &serde_yaml::Value) -> Vec<VolumeConfig> {
    let mut volumes = Vec::new();

    if let Some(volumes_map) = config.get("volumes").and_then(|v| v.as_mapping()) {
        for (name, volume_value) in volumes_map {
            if let Some(name_str) = name.as_str() {
                let mut volume = VolumeConfig {
                    name: name_str.to_string(),
                    driver: "local".to_string(),
                    driver_opts: None,
                    labels: None,
                    external: false,
                    internal: false,
                };

                if let Some(driver) = volume_value.get("driver").and_then(|v| v.as_str()) {
                    volume.driver = driver.to_string();
                }

                if let Some(driver_opts) = volume_value.get("driver_opts").and_then(|v| v.as_mapping()) {
                    let mut opts = HashMap::new();
                    for (k, v) in driver_opts {
                        if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                            opts.insert(key.to_string(), val.to_string());
                        }
                    }
                    volume.driver_opts = Some(opts);
                }

                if let Some(labels) = volume_value.get("labels").and_then(|v| v.as_mapping()) {
                    let mut labels_map = HashMap::new();
                    for (k, v) in labels {
                        if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                            labels_map.insert(key.to_string(), val.to_string());
                        }
                    }
                    volume.labels = Some(labels_map);
                }

                if let Some(external) = volume_value.get("external").and_then(|v| v.as_bool()) {
                    volume.external = external;
                }

                if let Some(internal) = volume_value.get("internal").and_then(|v| v.as_bool()) {
                    volume.internal = internal;
                }

                volumes.push(volume);
            }
        }
    }

    volumes
}

#[derive(Clone)]
pub struct ComposeService {
    pub name: String,
    pub image: String,
    pub build: Option<String>,
    pub ports: Vec<String>,
    pub environment: Vec<String>,
    pub environment_map: Option<HashMap<String, String>>,
    pub env_file: Option<Vec<String>>,
    pub volumes: Vec<MountConfig>,
    pub command: Option<String>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
    pub entrypoint: Option<String>,
    pub restart: Option<String>,
    pub healthcheck: Option<HealthCheckConfig>,
    pub deploy: Option<DeployConfig>,
    pub labels: Option<Vec<String>>,
    pub network_mode: Option<String>,
    pub networks: HashMap<String, NetworkServiceConfig>,
    pub depends_on: Vec<String>,
    // 新增配置选项
    pub cap_add: Option<Vec<String>>,
    pub cap_drop: Option<Vec<String>>,
    pub cgroup_parent: Option<String>,
    pub device_cgroup_rules: Option<Vec<String>>,
    pub devices: Option<Vec<String>>,
    pub dns: Option<Vec<String>>,
    pub dns_search: Option<Vec<String>>,
    pub domainname: Option<String>,
    pub extra_hosts: Option<Vec<String>>,
    pub hostname: Option<String>,
    pub ipc: Option<String>,
    pub isolation: Option<String>,
    pub logging: Option<LoggingConfig>,
    pub mac_address: Option<String>,
    pub mem_limit: Option<String>,
    pub mem_reservation: Option<String>,
    pub oom_kill_disable: Option<bool>,
    pub oom_score_adj: Option<i32>,
    pub pid: Option<String>,
    pub pids_limit: Option<u64>,
    pub read_only: Option<bool>,
    pub shm_size: Option<String>,
    pub stdin_open: Option<bool>,
    pub stop_grace_period: Option<String>,
    pub stop_signal: Option<String>,
    pub tty: Option<bool>,
    pub ulimits: Option<HashMap<String, UlimitConfig>>,
    pub sysctls: Option<HashMap<String, String>>,
    // Profiles 支持
    pub profiles: Option<Vec<String>>,
    // Extends 支持
    pub extends: Option<ExtendsConfig>,
}

#[derive(Clone)]
pub struct NetworkServiceConfig {
    pub aliases: Vec<String>,
    pub ipv4_address: Option<String>,
    pub ipv6_address: Option<String>,
}

#[derive(Clone)]
pub struct HealthCheckConfig {
    pub test: Vec<String>,
    pub interval: Option<String>,
    pub timeout: Option<String>,
    pub retries: Option<u32>,
    pub start_period: Option<String>,
}

#[derive(Clone)]
pub struct DeployConfig {
    pub replicas: Option<u32>,
    pub restart_policy: Option<RestartPolicyConfig>,
    pub resources: Option<ResourcesConfig>,
    pub labels: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct RestartPolicyConfig {
    pub condition: Option<String>,
    pub delay: Option<String>,
    pub max_attempts: Option<u32>,
    pub window: Option<String>,
}

#[derive(Clone)]
pub struct ResourcesConfig {
    pub limits: Option<ResourceLimits>,
    pub reservations: Option<ResourceReservations>,
}

#[derive(Clone)]
pub struct ResourceLimits {
    pub cpus: Option<String>,
    pub memory: Option<String>,
}

#[derive(Clone)]
pub struct ResourceReservations {
    pub cpus: Option<String>,
    pub memory: Option<String>,
}

pub struct NetworkConfig {
    pub name: String,
    pub driver: String,
    pub driver_opts: Option<HashMap<String, String>>,
    pub ipam: Option<IpamConfig>,
    pub internal: bool,
    pub external: bool,
    pub attachable: bool,
    pub enable_ipv6: bool,
    pub labels: Option<HashMap<String, String>>,
}

pub struct IpamConfig {
    pub driver: String,
    pub config: Vec<IpamSubnetConfig>,
}

pub struct IpamSubnetConfig {
    pub subnet: String,
    pub gateway: Option<String>,
    pub ip_range: Option<String>,
}

pub struct VolumeConfig {
    pub name: String,
    pub driver: String,
    pub driver_opts: Option<HashMap<String, String>>,
    pub labels: Option<HashMap<String, String>>,
    pub external: bool,
    pub internal: bool,
}

#[derive(Clone)]
pub struct MountConfig {
    pub source: String,
    pub target: String,
    pub read_only: bool,
    pub mount_type: String,          // volume, bind, tmpfs
    pub consistency: Option<String>, // for bind mounts
    pub tmpfs_size: Option<u64>,     // for tmpfs mounts
    pub tmpfs_mode: Option<u32>,     // for tmpfs mounts
}

#[derive(Clone)]
pub struct LoggingConfig {
    pub driver: String,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Clone)]
pub struct UlimitConfig {
    pub soft: Option<u64>,
    pub hard: Option<u64>,
}

#[derive(Clone)]
pub struct ExtendsConfig {
    pub service: String,
    pub file: Option<String>,
}

impl ComposeService {
    pub fn default(name: &str) -> Self {
        Self {
            name: name.to_string(),
            image: "nginx:latest".to_string(),
            build: None,
            ports: vec!["80:80".to_string()],
            environment: vec![],
            environment_map: None,
            env_file: None,
            volumes: vec![],
            command: None,
            working_dir: None,
            user: None,
            entrypoint: None,
            restart: None,
            healthcheck: None,
            deploy: None,
            labels: None,
            network_mode: None,
            networks: HashMap::new(),
            depends_on: vec![],
            // 新增配置选项
            cap_add: None,
            cap_drop: None,
            cgroup_parent: None,
            device_cgroup_rules: None,
            devices: None,
            dns: None,
            dns_search: None,
            domainname: None,
            extra_hosts: None,
            hostname: None,
            ipc: None,
            isolation: None,
            logging: None,
            mac_address: None,
            mem_limit: None,
            mem_reservation: None,
            oom_kill_disable: None,
            oom_score_adj: None,
            pid: None,
            pids_limit: None,
            read_only: None,
            shm_size: None,
            stdin_open: None,
            stop_grace_period: None,
            stop_signal: None,
            tty: None,
            ulimits: None,
            sysctls: None,
            profiles: None,
            extends: None,
        }
    }
}
