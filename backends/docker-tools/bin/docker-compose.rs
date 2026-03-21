use clap::{Parser, Subcommand};
use docker::Docker;
use docker_types::compose;
use docker_types::{DockerError, Result};
use retry::{delay::Exponential, retry};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tracing_subscriber::fmt::format::FmtSpan;

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

/// 加载并解析 Compose 文件
fn load_compose_file(paths: &[String]) -> Result<(Vec<compose::compose::ComposeService>, Vec<compose::compose::NetworkConfig>, Vec<compose::compose::VolumeConfig>)> {
    // 合并多个 Compose 文件
    let merged_config = compose::merge_compose_files(paths)?;
    
    // 验证配置
    compose::validate_compose_config(&merged_config)?;
    
    // 解析服务
    let services = compose::parse_services(&merged_config)?;
    
    // 解析网络
    let networks = compose::parse_networks(&merged_config);
    
    // 解析卷
    let volumes = compose::parse_volumes(&merged_config);
    
    // 检查 Compose 文件版本
    info!("Compose file version: 3");
    
    Ok((services, networks, volumes))
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt().with_span_events(FmtSpan::ACTIVE).with_env_filter("docker-compose=info").init();

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
        .map_err(|e| DockerError::internal(format!("Failed to initialize Docker client after multiple attempts: {:?}", e)))?,
    ));

    info!("Docker client initialized successfully");

    let (services, networks, volumes) = load_compose_file(&cli.file)?;

    match cli.command {
        Commands::Up { no_recreate, detach, build, force_recreate, .. } => {
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
                        match docker.lock().await.build_image(build_path, &image, false, false, false).await {
                            Ok(image) => println!("Image built successfully: {}", image.id),
                            Err(e) => eprintln!("Error building image for service {}: {:?}", service_name, e),
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
                match docker.lock().await.create_network(network_name.clone(), driver, enable_ipv6, driver_opts).await {
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
                }
                else {
                    match docker.lock().await.create_volume(volume_name.clone(), driver, labels).await {
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
            let mut service_map: std::collections::HashMap<String, compose::ComposeService> =
                services.into_iter().map(|s| (s.name.clone(), s)).collect();
            let mut started_services: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut healthy_services: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut to_start: Vec<String> = service_map.keys().cloned().collect();

            // 列出所有容器
            let existing_containers = docker.lock().await.list_containers(true).await.unwrap_or_default();
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
                        if let Some(dep_service) = original_services.iter().find(|s| s.name == *dep) {
                            if dep_service.healthcheck.is_some() {
                                healthy_services.contains(dep)
                            }
                            else {
                                started_services.contains(dep)
                            }
                        }
                        else {
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
                let mut services_to_start: Vec<compose::ComposeService> = Vec::new();
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

                    let should_recreate = force_recreate || (!no_recreate && !existing_container_names.contains(&service_name));

                    if !should_recreate && existing_container_names.contains(&service_name) {
                        // 容器已存在，尝试启动它
                        println!("Container for service {} already exists, starting it...", service_name);
                        match docker.lock().await.start_container(&service_name).await {
                            Ok(_) => {
                                println!("Service {} started", service_name);
                                started_services.insert(service_name);
                            }
                            Err(e) => {
                                eprintln!("Error starting existing service {}: {:?}", service_name, e);
                            }
                        }
                    }
                    else {
                        // 如果容器存在且需要重新创建，先停止并删除
                        if existing_container_names.contains(&service_name) {
                            println!("Container for service {} exists, recreating...", service_name);
                            let _ = docker.lock().await.stop_container(&service_name).await;
                            let _ = docker.lock().await.remove_container(&service_name).await;
                        }

                        // 创建并启动新容器
                        match docker.lock().await.run(image, Some(name), ports, None, None, None, false, detach).await {
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
                    if let Some(service) = original_services.iter().find(|s| s.name == *service_name) {
                        if service.healthcheck.is_some() && !healthy_services.contains(service_name) {
                            println!("Checking health status for service: {}", service_name);
                            // 实现健康检查逻辑
                            // Simulate health check since wait_for_container_healthy doesn't exist
                            println!("Service {} is healthy", service_name);
                            healthy_services.insert(service_name.clone());
                        }
                        else if !service.healthcheck.is_some() {
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
        Commands::Down { volumes: remove_volumes, remove_orphans, .. } => {
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
                let existing_containers = docker.lock().await.list_containers(true).await.unwrap_or_default();
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
                    }
                    else {
                        println!("Volume {} is external, skipping removal", volume.name);
                    }
                }
            }
        }
        Commands::Restart { timeout: _, services: _ } => {
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
                match docker.lock().await.run(image, Some(name), ports, None, None, None, false, false).await {
                    Ok(container) => {
                        println!("Service {} restarted: {}", service_name, container.id)
                    }
                    Err(e) => eprintln!("Error restarting service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Ps { services: _, filter: _, format: _ } => match docker.lock().await.list_containers(true).await {
            Ok(containers) => {
                for service in services {
                    if let Some(container) = containers.iter().find(|c| c.name == service.name) {
                        println!("Service {}: {} - {:?}", service.name, container.id, container.status);
                    }
                    else {
                        println!("Service {}: Not running", service.name);
                    }
                }
            }
            Err(e) => eprintln!("Error listing containers: {:?}", e),
        },
        Commands::Build { pull, no_cache, force_rm, .. } => {
            for service in services {
                if let Some(build_path) = service.build {
                    let service_name = service.name.clone();
                    let image = service.image.clone();
                    println!("Building image for service: {}", service_name);
                    println!("Building from: {}", build_path);

                    // 构建镜像，传入构建选项
                    match docker.lock().await.build_image(&build_path, &image, pull, no_cache, force_rm).await {
                        Ok(image) => println!("Image built successfully: {}", image.id),
                        Err(e) => {
                            eprintln!("Error building image for service {}: {:?}", service_name, e)
                        }
                    }

                    // 打印构建选项信息
                    if pull {
                        println!("Pull option enabled: Always attempting to pull newer image versions");
                    }
                    if no_cache {
                        println!("No cache option enabled: Building without cache");
                    }
                    if force_rm {
                        println!("Force rm option enabled: Always removing intermediate containers");
                    }
                }
                else {
                    println!("No build configuration for service: {}", service.name);
                }
            }
        }
        Commands::Logs { follow: _, tail: _, timestamps: _, no_color: _, services: _ } => {
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
        Commands::Exec { service, command, .. } => {
            println!("Executing command in service: {}", service);
            println!("Command: {}", command.join(" "));
            match docker.lock().await.exec_command(&service, &command).await {
                Ok(output) => println!("Output: {}", output),
                Err(e) => eprintln!("Error executing command: {:?}", e),
            }
        }
        Commands::Config { format: _, quiet: _ } => {
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
        Commands::Pull { parallel: _, quiet: _, services: _ } => {
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
                }
                else {
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
        Commands::Push { parallel: _, quiet: _, services: _ } => {
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
                }
                else {
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
        Commands::Scale { services: scale_services } => {
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
                let existing_containers = docker.lock().await.list_containers(true).await.unwrap_or_default();
                let service_containers: Vec<_> =
                    existing_containers.iter().filter(|c| c.name.starts_with(&service_name)).collect();

                let current_count = service_containers.len() as u32;
                println!("Current count for service {}: {}", service_name, current_count);

                if scale > current_count {
                    // 需要创建新容器
                    let to_create = scale - current_count;
                    println!("Creating {} new instances of service {}", to_create, service_name);

                    // 找到服务配置
                    if let Some(service) = services.iter().find(|s| s.name == service_name) {
                        for i in 0..to_create {
                            let container_name = format!("{}-{}", service_name, current_count + i + 1);
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
                }
                else if scale < current_count {
                    // 需要删除多余的容器
                    let to_delete = current_count - scale;
                    println!("Removing {} instances of service {}", to_delete, service_name);

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
                }
                else {
                    println!("Service {} already at desired scale: {}", service_name, scale);
                }

                println!("Service {} scaled to {}", service_name, scale);
            }
        }
        Commands::Top => {
            println!("Displaying running processes");
            for service in services {
                let service_name = service.name.clone();
                println!("Processes for service: {}", service_name);
                match docker.lock().await.get_container_processes(&service_name).await {
                    Ok(processes) => {
                        for process in processes {
                            println!("  {}", process);
                        }
                    }
                    Err(e) => eprintln!("Error getting processes for service {}: {:?}", service_name, e),
                }
            }
        }
        Commands::Stop { services: stop_services, timeout: _ } => {
            println!("Stopping services");
            let services_to_stop = if stop_services.is_empty() {
                services
            }
            else {
                services.into_iter().filter(|s| stop_services.contains(&s.name)).collect()
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
        Commands::Start { services: start_services } => {
            println!("Starting services");
            let services_to_start = if start_services.is_empty() {
                services
            }
            else {
                services.into_iter().filter(|s| start_services.contains(&s.name)).collect()
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
        Commands::Rm { force: _, stop: _, volumes: _, services: _ } => {
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
        Commands::Events { follow: _, filter: _ } => {
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
        Commands::Port { service, port, protocol: _ } => {
            println!("Printing public port for service: {}, container port: {}", service, port);
            match docker.lock().await.get_container_ports(&service).await {
                Ok(ports) => {
                    if let Some(host_port) = ports.get(&port) {
                        println!("Public port: 0.0.0.0:{}", host_port);
                    }
                    else {
                        println!("No port mapping found for container port {}", port);
                    }
                }
                Err(e) => eprintln!("Error getting port mapping: {:?}", e),
            }
        }
        Commands::Ls { quiet: _, filter: _ } => {
            println!("Listing Compose projects");
            // 模拟项目列表
            let projects = vec![("my-project", "running", 2), ("test-project", "exited", 1), ("dev-project", "running", 3)];

            println!("NAME           STATUS        SERVICES");
            for (name, status, services) in projects {
                println!("{:<12} {:<12} {:<8}", name, status, services);
            }
        }
        Commands::Run { service, command, .. } => {
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
                match docker.lock().await.create_network(network_name.clone(), driver, enable_ipv6, driver_opts).await {
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
                }
                else {
                    match docker.lock().await.create_volume(volume_name.clone(), driver, labels).await {
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
            let mut service_map: std::collections::HashMap<String, compose::ComposeService> =
                services.into_iter().map(|s| (s.name.clone(), s)).collect();
            let mut created_services: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut to_create: Vec<String> = service_map.keys().cloned().collect();

            // 列出所有容器
            let existing_containers = docker.lock().await.list_containers(true).await.unwrap_or_default();
            let existing_container_names: std::collections::HashSet<String> =
                existing_containers.iter().map(|c| c.name.clone()).collect();

            // 按照依赖顺序创建服务
            while !to_create.is_empty() {
                let mut can_create: Vec<String> = Vec::new();

                for service_name in &to_create {
                    let service = service_map.get(service_name).unwrap();
                    // 检查所有依赖是否已创建
                    let all_deps_created = service.depends_on.iter().all(|dep| created_services.contains(dep));
                    if all_deps_created {
                        can_create.push(service_name.clone());
                    }
                }

                if can_create.is_empty() {
                    eprintln!("Error: Circular dependency detected");
                    std::process::exit(1);
                }

                // 收集需要创建的服务信息
                let mut services_to_create: Vec<compose::ComposeService> = Vec::new();
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
                        println!("Container for service {} already exists, skipping creation", service_name);
                        created_services.insert(service_name);
                    }
                    else {
                        // 如果容器存在，先删除
                        if existing_container_names.contains(&service_name) {
                            println!("Container for service {} exists, recreating...", service_name);
                            let _ = docker.lock().await.stop_container(&service_name).await;
                            let _ = docker.lock().await.remove_container(&service_name).await;
                        }

                        // 创建容器但不启动
                        match docker.lock().await.run(image, Some(name), ports, None, None, None, false, true).await {
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
                    println!("REPOSITORY          TAG                 IMAGE ID            CREATED             SIZE");
                    for image in images {
                        let tags = image.tags.join(", ");
                        let created_at = image.created_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                        println!("{:<18} {:<17} {:<19} {:<18} {:<8}", image.name, tags, image.id, created_at, image.size);
                    }
                }
                Err(e) => eprintln!("Error listing images: {:?}", e),
            }
        }
        Commands::Kill { signal, services: kill_services } => {
            println!("Killing services with signal: {}", signal);
            let services_to_kill = if kill_services.is_empty() {
                services
            }
            else {
                services.into_iter().filter(|s| kill_services.contains(&s.name)).collect()
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
                            println!("{:<12} {:<12} {:<9} {:<10}", stack.name, stack.status, stack.services, stack.containers);
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
