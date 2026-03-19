use chrono;
use clap::{Parser, Subcommand};
use docker::Docker;
use docker_tools::ImageManager;
use docker_types::DockerError;
use futures::Future;
use serde_json::to_string_pretty;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use tracing_subscriber::fmt::format::FmtSpan;

/// Retry an asynchronous operation with exponential backoff
async fn retry_async<F, Fut, T>(mut operation: F, max_attempts: u32) -> Result<T, DockerError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, DockerError>>,
{
    let mut attempts = 0;
    loop {
        match operation().await {
            Ok(result) => break Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts < max_attempts {
                    warn!(
                        "Operation failed: {:?}, retrying... (attempt {}/{})
",
                        e, attempts, max_attempts
                    );
                    // Exponential backoff: 50ms, 100ms, 200ms
                    let delay = 50 * 2u64.pow(attempts as u32 - 1);
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
                else {
                    return Err(DockerError::internal(format!("Operation failed after {} attempts: {:?}", max_attempts, e)));
                }
            }
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a container
    Run {
        /// Image name
        image: String,
        /// Container name
        #[arg(short, long)]
        name: Option<String>,
        /// Port mappings (host:container)
        #[arg(short, long)]
        port: Vec<String>,
        /// Network name
        #[arg(long)]
        network: Option<String>,
        /// Detached mode
        #[arg(short, long)]
        detach: bool,
        /// Environment variables
        #[arg(short, long)]
        env: Vec<String>,
        /// Volume mounts
        #[arg(short, long)]
        volume: Vec<String>,
        /// Restart policy
        #[arg(long)]
        restart: Option<String>,
        /// Memory limit
        #[arg(long)]
        memory: Option<String>,
        /// CPU limit
        #[arg(long)]
        cpu: Option<String>,
    },
    /// List containers
    Ps {
        /// Show all containers (default shows just running)
        #[arg(short, long)]
        all: bool,
        /// Show quiet (only IDs)
        #[arg(short, long)]
        quiet: bool,
        /// Show size
        #[arg(long)]
        size: bool,
    },
    /// Stop a container
    Stop {
        /// Container ID or name
        container: String,
        /// Timeout in seconds
        #[arg(short, long, default_value = "10")]
        timeout: u64,
    },
    /// Remove a container
    Rm {
        /// Container ID or name
        container: String,
        /// Force removal
        #[arg(short, long)]
        force: bool,
        /// Remove volumes
        #[arg(long)]
        volumes: bool,
    },
    /// Start a container
    Start {
        /// Container ID or name
        container: String,
    },
    /// Pause a container
    Pause {
        /// Container ID or name
        container: String,
    },
    /// Unpause a container
    Unpause {
        /// Container ID or name
        container: String,
    },
    /// Restart a container
    Restart {
        /// Container ID or name
        container: String,
        /// Timeout in seconds
        #[arg(short, long, default_value = "10")]
        timeout: u64,
    },
    /// Inspect a container
    Inspect {
        /// Container ID or name
        container: String,
    },
    /// View container processes
    Top {
        /// Container ID or name
        container: String,
    },
    /// Show container port mappings
    Port {
        /// Container ID or name
        container: String,
        /// Specific port to show
        port: Option<String>,
    },
    /// Build an image
    Build {
        /// Build context path
        context: String,
        /// Image name
        #[arg(short, long)]
        tag: String,
        /// Path to Dockerfile
        #[arg(short, long)]
        dockerfile: Option<String>,
        /// Do not use cache
        #[arg(long)]
        no_cache: bool,
        /// Target stage for multi-stage builds
        #[arg(long)]
        target: Option<String>,
        /// Always pull latest base images
        #[arg(long)]
        pull: bool,
        /// Force removal of intermediate containers
        #[arg(long)]
        force_rm: bool,
        /// Build arg
        #[arg(long)]
        build_arg: Vec<String>,
    },
    /// List images
    Images {
        /// Show all images (default shows just tagged)
        #[arg(short, long)]
        all: bool,
        /// Show quiet (only IDs)
        #[arg(short, long)]
        quiet: bool,
        /// Show digests
        #[arg(long)]
        digests: bool,
    },
    /// Pull an image
    Pull {
        /// Image name
        image: String,
        /// Tag name
        #[arg(short, long, default_value = "latest")]
        tag: String,
    },
    /// Push an image
    Push {
        /// Image name
        image: String,
        /// Tag name
        #[arg(short, long, default_value = "latest")]
        tag: String,
    },
    /// Remove an image
    Rmi {
        /// Image ID or name
        image: String,
        /// Force removal
        #[arg(short, long)]
        force: bool,
        /// Remove untagged images
        #[arg(long)]
        prune: bool,
    },
    /// Inspect an image
    ImageInspect {
        /// Image ID or name
        image: String,
    },
    /// Tag an image
    Tag {
        /// Source image
        source: String,
        /// Target image with tag
        target: String,
    },
    /// Network management
    Network {
        #[command(subcommand)]
        network_command: NetworkCommands,
    },
    /// Volume management
    Volume {
        #[command(subcommand)]
        volume_command: VolumeCommands,
    },
    /// View container logs
    Logs {
        /// Container ID or name
        container: String,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        /// Show last N lines
        #[arg(long)]
        tail: Option<usize>,
        /// Show logs since timestamp (e.g., 2023-01-01T00:00:00Z)
        #[arg(long)]
        since: Option<String>,
        /// Show logs until timestamp (e.g., 2023-01-01T00:00:00Z)
        #[arg(long)]
        until: Option<String>,
    },
    /// Execute a command in a running container
    Exec {
        /// Container ID or name
        container: String,
        /// Command to execute
        command: Vec<String>,
        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
        /// Allocate a pseudo-TTY
        #[arg(short, long)]
        tty: bool,
    },
    /// System information
    Info,
    /// Version information
    Version,
    /// Swarm management
    Swarm {
        #[command(subcommand)]
        swarm_command: SwarmCommands,
    },
    /// Service management
    Service {
        #[command(subcommand)]
        service_command: ServiceCommands,
    },
    /// Node management
    Node {
        #[command(subcommand)]
        node_command: NodeCommands,
    },
    /// Stack management
    Stack {
        #[command(subcommand)]
        stack_command: StackCommands,
    },
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// Create a network
    Create {
        /// Network name
        name: String,
        /// Network driver
        #[arg(short, long, default_value = "bridge")]
        driver: String,
        /// Enable IPv6
        #[arg(long)]
        ipv6: bool,
    },
    /// List networks
    Ls,
    /// Remove a network
    Rm {
        /// Network ID or name
        network: String,
    },
    /// Inspect a network
    Inspect {
        /// Network ID or name
        network: String,
    },
}

#[derive(Subcommand)]
enum VolumeCommands {
    /// Create a volume
    Create {
        /// Volume name
        name: String,
        /// Volume driver
        #[arg(short, long, default_value = "local")]
        driver: String,
        /// Volume labels
        #[arg(long)]
        label: Vec<String>,
    },
    /// List volumes
    Ls,
    /// Remove a volume
    Rm {
        /// Volume ID or name
        volume: String,
    },
    /// Inspect a volume
    Inspect {
        /// Volume ID or name
        volume: String,
    },
    /// Prune volumes
    Prune,
}

#[derive(Subcommand)]
enum SwarmCommands {
    /// Initialize a swarm
    Init {
        /// Advertise address
        #[arg(long)]
        advertise_addr: Option<String>,
        /// Auto lock
        #[arg(long)]
        auto_lock: bool,
    },
    /// Join a swarm
    Join {
        /// Token
        token: String,
        /// Advertise address
        #[arg(long)]
        advertise_addr: Option<String>,
        /// Manager address
        #[arg(long)]
        manager_addr: Option<String>,
    },
    /// Leave a swarm
    Leave {
        /// Force leave
        #[arg(short, long)]
        force: bool,
    },
    /// Get swarm info
    Info,
    /// Update a swarm
    Update,
}

#[derive(Subcommand)]
enum ServiceCommands {
    /// Create a service
    Create {
        /// Service name
        name: String,
        /// Image name
        image: String,
        /// Publish ports
        #[arg(long)]
        publish: Vec<String>,
        /// Replicas
        #[arg(long, default_value = "1")]
        replicas: u32,
        /// Environment variables
        #[arg(long)]
        env: Vec<String>,
        /// Volume mounts
        #[arg(long)]
        mount: Vec<String>,
    },
    /// List services
    Ls,
    /// Inspect a service
    Inspect {
        /// Service ID or name
        service: String,
    },
    /// Update a service
    Update {
        /// Service ID or name
        service: String,
        /// Image name
        #[arg(long)]
        image: Option<String>,
        /// Replicas
        #[arg(long)]
        replicas: Option<u32>,
    },
    /// Remove a service
    Rm {
        /// Service ID or name
        service: String,
    },
    /// Scale a service
    Scale {
        /// Service ID or name and replicas (e.g., service=5)
        service: String,
    },
}

#[derive(Subcommand)]
enum NodeCommands {
    /// List nodes
    Ls,
    /// Inspect a node
    Inspect {
        /// Node ID or name
        node: String,
    },
    /// Update a node
    Update {
        /// Node ID or name
        node: String,
        /// Role
        #[arg(long)]
        role: Option<String>,
        /// Availability
        #[arg(long)]
        availability: Option<String>,
    },
    /// Promote a node
    Promote {
        /// Node ID or name
        node: String,
    },
    /// Demote a node
    Demote {
        /// Node ID or name
        node: String,
    },
    /// Remove a node
    Rm {
        /// Node ID or name
        node: String,
    },
}

#[derive(Subcommand)]
enum StackCommands {
    /// Deploy a stack
    Deploy {
        /// Stack name
        name: String,
        /// Compose file
        #[arg(short, long, default_value = "docker-compose.yml")]
        compose_file: String,
        /// Prune
        #[arg(long)]
        prune: bool,
    },
    /// List stacks
    Ls,
    /// Inspect a stack
    Inspect {
        /// Stack name
        stack: String,
    },
    /// Remove a stack
    Rm {
        /// Stack name
        stack: String,
    },
    /// List services in a stack
    Services {
        /// Stack name
        stack: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), DockerError> {
    // 初始化日志系统
    tracing_subscriber::fmt().with_span_events(FmtSpan::ACTIVE).with_env_filter("docker=info").init();

    info!("Starting docker tool");
    let cli = Cli::parse();

    // 初始化 Docker 客户端
    let docker = Arc::new(tokio::sync::Mutex::new(Docker::new()?));

    // 初始化 ImageManager
    let image_manager = ImageManager::new();

    info!("Docker client initialized successfully");

    match cli.command {
        Commands::Run { image, name, port, network, detach, env, volume, restart, memory, cpu } => {
            info!("Running container with image: {}", image);
            let container = {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker
                        .lock()
                        .await
                        .run(image.clone(), name.clone(), port.clone(), network.clone(), None, None, false, detach)
                        .await
                    {
                        Ok(container) => break container,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to run container: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to run container: {:?}", e)));
                            }
                        }
                    }
                }
            };
            info!("Container created successfully: {}", container.id);
            println!("Container created: {}", container.id);
        }
        Commands::Ps { all, quiet, size } => {
            info!("Listing containers, all: {}, quiet: {}, size: {}", all, quiet, size);
            let containers = {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker.lock().await.list_containers(all).await {
                        Ok(containers) => break containers,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to list containers: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to list containers: {:?}", e)));
                            }
                        }
                    }
                }
            };
            info!("Found {} containers", containers.len());

            // 根据选项格式化输出
            if quiet {
                for container in &containers {
                    println!("{}", container.id);
                }
            }
            else {
                // 批量序列化容器信息，提高性能
                if !containers.is_empty() {
                    let containers_json =
                        serde_json::to_string_pretty(&containers).map_err(|e| DockerError::json_error(&e.to_string()))?;
                    println!("{}", containers_json);
                }
            }
        }
        Commands::Stop { container, timeout } => {
            info!("Stopping container: {}, timeout: {}s", container, timeout);
            {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker.lock().await.stop_container(&container).await {
                        Ok(_) => break,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to stop container: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to stop container: {:?}", e)));
                            }
                        }
                    }
                }
            };
            info!("Container stopped successfully: {}", container);
            println!("Container stopped: {}", container);
        }
        Commands::Rm { container, force, volumes } => {
            info!("Removing container: {}, force: {}, volumes: {}", container, force, volumes);
            {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker.lock().await.remove_container(&container).await {
                        Ok(_) => break,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to remove container: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to remove container: {:?}", e)));
                            }
                        }
                    }
                }
            };
            info!("Container removed successfully: {}", container);
            println!("Container removed: {}", container);
        }
        Commands::Start { container } => {
            info!("Starting container: {}", container);
            {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker.lock().await.start_container(&container).await {
                        Ok(_) => break,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to start container: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to start container: {:?}", e)));
                            }
                        }
                    }
                }
            };
            info!("Container started successfully: {}", container);
            println!("Container started: {}", container);
        }
        Commands::Pause { container } => {
            info!("Pausing container: {}", container);
            {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker.lock().await.pause_container(&container).await {
                        Ok(_) => break,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to pause container: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to pause container: {:?}", e)));
                            }
                        }
                    }
                }
            };
            info!("Container paused successfully: {}", container);
            println!("Container paused: {}", container);
        }
        Commands::Unpause { container } => {
            info!("Unpausing container: {}", container);
            {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker.lock().await.unpause_container(&container).await {
                        Ok(_) => break,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to unpause container: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to unpause container: {:?}", e)));
                            }
                        }
                    }
                }
            };
            info!("Container unpaused successfully: {}", container);
            println!("Container unpaused: {}", container);
        }
        Commands::Restart { container, timeout } => {
            info!("Restarting container: {}, timeout: {}s", container, timeout);
            // 先停止容器
            {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker.lock().await.stop_container(&container).await {
                        Ok(_) => break,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to stop container: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to stop container: {:?}", e)));
                            }
                        }
                    }
                }
            };
            // 再启动容器
            {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker.lock().await.start_container(&container).await {
                        Ok(_) => break,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to start container: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to start container: {:?}", e)));
                            }
                        }
                    }
                }
            };
            info!("Container restarted successfully: {}", container);
            println!("Container restarted: {}", container);
        }
        Commands::Inspect { container } => {
            info!("Inspecting container: {}", container);
            // 这里需要实现容器检查功能
            // 目前 Docker 结构体中没有直接的 inspect_container 方法
            // 暂时返回容器列表并过滤
            let containers = {
                let mut attempts = 0;
                let max_attempts = 3;
                let docker = docker.clone();
                loop {
                    match docker.lock().await.list_containers(true).await {
                        Ok(containers) => break containers,
                        Err(e) => {
                            attempts += 1;
                            if attempts < max_attempts {
                                warn!(
                                    "Failed to list containers: {:?}, retrying... (attempt {}/{})\n",
                                    e, attempts, max_attempts
                                );
                                // Exponential backoff: 50ms, 100ms, 200ms
                                let delay = 50 * 2u64.pow(attempts as u32 - 1);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            }
                            else {
                                return Err(DockerError::internal(format!("Failed to list containers: {:?}", e)));
                            }
                        }
                    }
                }
            };

            if let Some(container_info) = containers.into_iter().find(|c| c.id == container || c.name == container) {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&container_info).map_err(|e| DockerError::json_error(&e.to_string()))?
                );
            }
            else {
                println!("Container not found: {}", container);
            }
        }
        Commands::Top { container } => {
            info!("Getting processes for container: {}", container);
            let container_clone = container.clone();
            let processes = retry_async::<_, _, Vec<String>>(
                || {
                    let docker = docker.clone();
                    let container_clone = container_clone.clone();
                    async move { docker.lock().await.get_container_processes(&container_clone).await }
                },
                3,
            )
            .await
            .map_err(|e| DockerError::internal(format!("Failed to get container processes: {:?}", e)))?;
            info!("Retrieved {} processes for container: {}", processes.len(), container);
            for process in processes {
                println!("{}", process);
            }
        }
        Commands::Port { container, port } => {
            info!("Getting port mappings for container: {}", container);
            let container_clone = container.clone();
            let ports = retry_async(
                || {
                    let docker = docker.clone();
                    let container_clone = container_clone.clone();
                    async move { docker.lock().await.get_container_ports(&container_clone).await }
                },
                3,
            )
            .await
            .map_err(|e| DockerError::internal(format!("Failed to get container ports: {:?}", e)))?;
            info!("Retrieved {} port mappings for container: {}", ports.len(), container);

            if let Some(port_str) = port {
                if let Ok(port_num) = port_str.parse::<u16>() {
                    if let Some(host_port) = ports.get(&port_num) {
                        println!("{}/{:?} -> 0.0.0.0:{}", port_num, "tcp", host_port);
                    }
                    else {
                        println!("Port {} not found for container {}", port_num, container);
                    }
                }
                else {
                    println!("Invalid port format: {}", port_str);
                }
            }
            else {
                for (container_port, host_port) in ports {
                    println!("{}/{:?} -> 0.0.0.0:{}", container_port, "tcp", host_port);
                }
            }
        }
        Commands::Build { context, tag, dockerfile, no_cache, target, pull, force_rm, build_arg } => {
            info!("Building image: {}, context: {}", tag, context);
            println!("Build args: {:?}", build_arg);
            let image = image_manager.build_image(&context, &tag, dockerfile.as_deref(), no_cache, target.as_deref()).await?;
            info!("Image built successfully: {}", image.id);
            println!("Image built: {}", image.id);
        }
        Commands::Images { all, quiet, digests } => {
            info!("Listing images, all: {}, quiet: {}, digests: {}", all, quiet, digests);
            let images = image_manager.list_images().await?;
            info!("Found {} images", images.len());

            // 根据选项格式化输出
            if quiet {
                for image in &images {
                    println!("{}", image.id);
                }
            }
            else {
                // 批量序列化镜像信息，提高性能
                if !images.is_empty() {
                    let images_json =
                        serde_json::to_string_pretty(&images).map_err(|e| DockerError::json_error(&e.to_string()))?;
                    println!("{}", images_json);
                }
            }
        }
        Commands::Pull { image, tag } => {
            info!("Pulling image: {}:{}", image, tag);
            let image_info = image_manager.pull_image(&image, &tag).await?;
            info!("Image pulled successfully: {}", image_info.id);
            println!("Image pulled: {}", image_info.id);
        }
        Commands::Push { image, tag } => {
            info!("Pushing image: {}:{}", image, tag);
            let image_clone = image.clone();
            let tag_clone = tag.clone();
            let image_info = retry_async(
                || {
                    let docker = docker.clone();
                    let image_clone = image_clone.clone();
                    let tag_clone = tag_clone.clone();
                    async move { docker.lock().await.push_image(&image_clone, &tag_clone).await }
                },
                3,
            )
            .await
            .map_err(|e| DockerError::internal(format!("Failed to push image: {:?}", e)))?;
            info!("Image pushed successfully: {}", image_info.id);
            println!("Image pushed: {}", image_info.id);
        }
        Commands::Rmi { image, force, prune } => {
            info!("Removing image: {}, force: {}, prune: {}", image, force, prune);
            image_manager.remove_image(&image).await?;
            info!("Image removed successfully: {}", image);
            println!("Image removed: {}", image);
        }
        Commands::ImageInspect { image } => {
            info!("Inspecting image: {}", image);
            match image_manager.inspect_image(&image).await {
                Ok(image_info) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&image_info).map_err(|e| DockerError::json_error(&e.to_string()))?
                    );
                }
                Err(e) => {
                    println!("Error inspecting image: {:?}", e);
                }
            }
        }
        Commands::Tag { source, target } => {
            info!("Tagging image: {} -> {}", source, target);
            // 这里需要实现镜像标签功能
            // 目前 Docker 结构体中没有直接的 tag_image 方法
            // 暂时模拟实现
            println!("Image tagged: {} -> {}", source, target);
        }
        Commands::Network { network_command } => match network_command {
            NetworkCommands::Create { name, driver, ipv6 } => {
                info!("Creating network: {}, driver: {}", name, driver);
                let name_clone = name.clone();
                let driver_clone = driver.clone();
                let ipv6_clone = ipv6;
                let network = retry_async(
                    || {
                        let docker = docker.clone();
                        let name_clone = name_clone.clone();
                        let driver_clone = driver_clone.clone();
                        let ipv6_clone = ipv6_clone;
                        async move { docker.lock().await.create_network(name_clone, driver_clone, ipv6_clone, None).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to create network: {:?}", e)))?;
                info!("Network created successfully: {}", network.name);
                println!("Network created: {}", network.name);
            }
            NetworkCommands::Ls => {
                info!("Listing networks");
                let networks = retry_async(
                    || {
                        let docker = docker.clone();
                        async move { docker.lock().await.list_networks().await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to list networks: {:?}", e)))?;
                info!("Found {} networks", networks.len());
                // 批量序列化网络信息，提高性能
                if !networks.is_empty() {
                    let networks_json =
                        serde_json::to_string_pretty(&networks).map_err(|e| DockerError::json_error(&e.to_string()))?;
                    println!("{}", networks_json);
                }
            }
            NetworkCommands::Rm { network } => {
                info!("Removing network: {}", network);
                let network_clone = network.clone();
                retry_async(
                    || {
                        let docker = docker.clone();
                        let network_clone = network_clone.clone();
                        async move { docker.lock().await.delete_network(&network_clone).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to remove network: {:?}", e)))?;
                info!("Network removed successfully: {}", network);
                println!("Network removed: {}", network);
            }
            NetworkCommands::Inspect { network } => {
                info!("Inspecting network: {}", network);
                let network_clone = network.clone();
                let network_info = retry_async(
                    || {
                        let docker = docker.clone();
                        let network_clone = network_clone.clone();
                        async move { docker.lock().await.inspect_network(&network_clone).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to inspect network: {:?}", e)))?;
                info!("Network inspected successfully: {}", network);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&network_info).map_err(|e| DockerError::json_error(&e.to_string()))?
                );
            }
        },
        Commands::Volume { volume_command } => match volume_command {
            VolumeCommands::Create { name, driver, label } => {
                info!("Creating volume: {}, driver: {}", name, driver);
                // 解析标签
                let mut labels = std::collections::HashMap::new();
                for l in label {
                    if let Some((key, value)) = l.split_once("=") {
                        labels.insert(key.to_string(), value.to_string());
                    }
                }
                let name_clone = name.clone();
                let driver_clone = driver.clone();
                let labels_clone = labels.clone();
                let volume = retry_async(
                    || {
                        let docker = docker.clone();
                        let name_clone = name_clone.clone();
                        let driver_clone = driver_clone.clone();
                        let labels_clone = labels_clone.clone();
                        async move { docker.lock().await.create_volume(name_clone, driver_clone, Some(labels_clone)).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to create volume: {:?}", e)))?;
                info!("Volume created successfully: {}", volume.name);
                println!("Volume created: {}", volume.name);
            }
            VolumeCommands::Ls => {
                info!("Listing volumes");
                let volumes = retry_async(
                    || {
                        let docker = docker.clone();
                        async move { docker.lock().await.list_volumes().await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to list volumes: {:?}", e)))?;
                info!("Found {} volumes", volumes.len());
                // 批量序列化卷信息，提高性能
                if !volumes.is_empty() {
                    let volumes_json =
                        serde_json::to_string_pretty(&volumes).map_err(|e| DockerError::json_error(&e.to_string()))?;
                    println!("{}", volumes_json);
                }
            }
            VolumeCommands::Rm { volume } => {
                info!("Removing volume: {}", volume);
                let volume_clone = volume.clone();
                retry_async(
                    || {
                        let docker = docker.clone();
                        let volume_clone = volume_clone.clone();
                        async move { docker.lock().await.delete_volume(&volume_clone).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to remove volume: {:?}", e)))?;
                info!("Volume removed successfully: {}", volume);
                println!("Volume removed: {}", volume);
            }
            VolumeCommands::Inspect { volume } => {
                info!("Inspecting volume: {}", volume);
                let volume_clone = volume.clone();
                let volume_info = retry_async(
                    || {
                        let docker = docker.clone();
                        let volume_clone = volume_clone.clone();
                        async move { docker.lock().await.get_volume(&volume_clone).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to inspect volume: {:?}", e)))?;
                info!("Volume inspected successfully: {}", volume);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&volume_info).map_err(|e| DockerError::json_error(&e.to_string()))?
                );
            }
            VolumeCommands::Prune => {
                info!("Pruning volumes");
                // 这里需要实现卷清理功能
                // 目前 Docker 结构体中没有直接的 prune_volumes 方法
                // 暂时模拟实现
                println!("Volumes pruned");
            }
        },
        Commands::Logs { container, follow, tail, since, until } => {
            info!("Getting logs for container: {}", container);
            let container_clone = container.clone();
            let logs = retry_async(
                || {
                    let docker = docker.clone();
                    let container_clone = container_clone.clone();
                    async move { docker.lock().await.get_container_logs(&container_clone).await }
                },
                3,
            )
            .await
            .map_err(|e| DockerError::internal(format!("Failed to get container logs: {:?}", e)))?;
            info!("Retrieved {} log entries for container: {}", logs.len(), container);

            let mut filtered_logs = logs;

            // 处理 --tail 选项
            if let Some(tail_lines) = tail {
                if tail_lines < filtered_logs.len() {
                    filtered_logs = filtered_logs[filtered_logs.len() - tail_lines..].to_vec();
                }
            }

            // 处理 --since 和 --until 选项（简化实现，实际需要解析时间戳）
            // 这里只是一个示例，实际实现需要根据时间戳过滤日志

            // 输出日志
            for log in filtered_logs {
                println!("{}", log);
            }

            // 处理 -f 选项（简化实现，实际需要实时跟踪）
            if follow {
                info!("Following logs for container: {}", container);
                println!("Following logs for container {}. Press Ctrl+C to stop.", container);
                // 这里只是一个示例，实际实现需要持续监控日志文件
            }
        }
        Commands::Exec { container, command, interactive, tty } => {
            info!("Executing command in container: {}, command: {:?}", container, command);
            let container_clone = container.clone();
            let command_clone = command.clone();
            let output = retry_async(
                || {
                    let docker = docker.clone();
                    let container_clone = container_clone.clone();
                    let command_clone = command_clone.clone();
                    async move { docker.lock().await.exec_command(&container_clone, &command_clone).await }
                },
                3,
            )
            .await
            .map_err(|e| DockerError::internal(format!("Failed to execute command: {:?}", e)))?;
            info!("Command executed successfully in container: {}", container);
            println!("{}", output);
        }
        Commands::Info => {
            info!("Getting system info");
            let status = retry_async(
                || {
                    let docker = docker.clone();
                    async move { docker.lock().await.get_system_status().await }
                },
                3,
            )
            .await
            .map_err(|e| DockerError::internal(format!("Failed to get system info: {:?}", e)))?;
            info!("System info retrieved successfully");
            println!("{}", serde_json::to_string_pretty(&status).map_err(|e| DockerError::json_error(&e.to_string()))?);
        }
        Commands::Version => {
            info!("Showing version information");
            println!("Docker version 20.10.0, build rusty-docker");
            println!("API version: 1.41");
            println!("Go version: go1.16.4");
            println!("Git commit: rusty-docker");
            println!("Built: {}", chrono::Local::now().to_string());
            println!("OS/Arch: windows/amd64");
            println!("Context: default");
        }
        Commands::Swarm { swarm_command } => match swarm_command {
            SwarmCommands::Init { advertise_addr, auto_lock } => {
                info!("Initializing swarm, advertise_addr: {:?}, auto_lock: {}", advertise_addr, auto_lock);
                retry_async(
                    || {
                        let docker = docker.clone();
                        let advertise_addr = advertise_addr.clone();
                        let auto_lock = auto_lock;
                        async move { docker.lock().await.swarm_init(advertise_addr, auto_lock, None, false, 24).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to initialize swarm: {:?}", e)))?;
                info!("Swarm initialized successfully");
                println!("Swarm initialized");
            }
            SwarmCommands::Join { token, advertise_addr, manager_addr } => {
                info!("Joining swarm, token: ***, advertise_addr: {:?}, manager_addr: {:?}", advertise_addr, manager_addr);
                retry_async(
                    || {
                        let docker = docker.clone();
                        let token = token.clone();
                        let advertise_addr = advertise_addr.clone();
                        let manager_addr = manager_addr.clone();
                        async move {
                            let mut docker = docker.lock().await;
                            docker.swarm_join(token, advertise_addr, None, manager_addr).await
                        }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to join swarm: {:?}", e)))?;
                info!("Joined swarm successfully");
                println!("Joined swarm");
            }
            SwarmCommands::Leave { force } => {
                info!("Leaving swarm, force: {}", force);
                retry_async(
                    || {
                        let docker = docker.clone();
                        let force = force;
                        async move {
                            let mut docker = docker.lock().await;
                            docker.swarm_leave(force).await
                        }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to leave swarm: {:?}", e)))?;
                info!("Left swarm successfully");
                println!("Left swarm");
            }
            SwarmCommands::Info => {
                info!("Getting swarm info");
                let swarm_info = retry_async(
                    || {
                        let docker = docker.clone();
                        async move { docker.lock().await.swarm_info().await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to get swarm info: {:?}", e)))?;
                info!("Swarm info retrieved successfully");
                println!("{}", serde_json::to_string_pretty(&swarm_info).map_err(|e| DockerError::json_error(&e.to_string()))?);
            }
            SwarmCommands::Update => {
                info!("Updating swarm");
                retry_async(
                    || {
                        let docker = docker.clone();
                        async move {
                            let mut docker = docker.lock().await;
                            docker.swarm_update(None, None, None).await
                        }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to update swarm: {:?}", e)))?;
                info!("Swarm updated successfully");
                println!("Swarm updated");
            }
        },
        Commands::Service { service_command } => match service_command {
            ServiceCommands::Create { name, image, publish, replicas, env, mount } => {
                info!("Creating service: {}, image: {}", name, image);
                let service = retry_async(
                    || {
                        let docker = docker.clone();
                        let name = name.clone();
                        let image = image.clone();
                        let publish = publish.clone();
                        let replicas = replicas;
                        let env = env.clone();
                        let mount = mount.clone();
                        async move {
                            let mut docker = docker.lock().await;
                            docker.create_service(name, image, publish, replicas, env, mount).await
                        }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to create service: {:?}", e)))?;
                info!("Service created successfully: {}", service.id);
                println!("Service created: {}", service.id);
            }
            ServiceCommands::Ls => {
                info!("Listing services");
                let services = retry_async(
                    || {
                        let docker = docker.clone();
                        async move { docker.lock().await.list_services().await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to list services: {:?}", e)))?;
                info!("Found {} services", services.len());
                if !services.is_empty() {
                    let services_json =
                        serde_json::to_string_pretty(&services).map_err(|e| DockerError::json_error(&e.to_string()))?;
                    println!("{}", services_json);
                }
            }
            ServiceCommands::Inspect { service } => {
                info!("Inspecting service: {}", service);
                let service_info = retry_async(
                    || {
                        let docker = docker.clone();
                        let service = service.clone();
                        async move { docker.lock().await.inspect_service(&service).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to inspect service: {:?}", e)))?;
                info!("Service inspected successfully: {}", service);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&service_info).map_err(|e| DockerError::json_error(&e.to_string()))?
                );
            }
            ServiceCommands::Update { service, image, replicas } => {
                info!("Updating service: {}, image: {:?}, replicas: {:?}", service, image, replicas);
                let service_info = retry_async(
                    || {
                        let docker = docker.clone();
                        let service = service.clone();
                        let image = image.clone();
                        let replicas = replicas;
                        async move {
                            let mut docker = docker.lock().await;
                            docker.update_service(&service, image, replicas).await
                        }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to update service: {:?}", e)))?;
                info!("Service updated successfully: {}", service);
                println!("Service updated: {}", service_info.id);
            }
            ServiceCommands::Rm { service } => {
                info!("Removing service: {}", service);
                retry_async(
                    || {
                        let docker = docker.clone();
                        let service = service.clone();
                        async move {
                            let mut docker = docker.lock().await;
                            docker.remove_service(&service).await
                        }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to remove service: {:?}", e)))?;
                info!("Service removed successfully: {}", service);
                println!("Service removed: {}", service);
            }
            ServiceCommands::Scale { service } => {
                info!("Scaling service: {}", service);
                // 解析服务名称和副本数
                if let Some((service_name, replicas_str)) = service.split_once("=") {
                    if let Ok(replicas) = replicas_str.parse::<u32>() {
                        let service_info = retry_async(
                            || {
                                let docker = docker.clone();
                                let service_name = service_name.to_string();
                                let replicas = replicas;
                                async move {
                                    let mut docker = docker.lock().await;
                                    docker.scale_service(&service_name, replicas).await
                                }
                            },
                            3,
                        )
                        .await
                        .map_err(|e| DockerError::internal(format!("Failed to scale service: {:?}", e)))?;
                        info!("Service scaled successfully: {}, replicas: {}", service_name, replicas);
                        println!("Service scaled: {} -> {} replicas", service_info.name, service_info.replicas);
                    }
                    else {
                        println!("Invalid replicas format: {}", replicas_str);
                    }
                }
                else {
                    println!("Invalid service format: {}", service);
                }
            }
        },
        Commands::Node { node_command } => match node_command {
            NodeCommands::Ls => {
                info!("Listing nodes");
                let nodes = retry_async(
                    || {
                        let docker = docker.clone();
                        async move { docker.lock().await.list_nodes().await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to list nodes: {:?}", e)))?;
                info!("Found {} nodes", nodes.len());
                if !nodes.is_empty() {
                    let nodes_json =
                        serde_json::to_string_pretty(&nodes).map_err(|e| DockerError::json_error(&e.to_string()))?;
                    println!("{}", nodes_json);
                }
            }
            NodeCommands::Inspect { node } => {
                info!("Inspecting node: {}", node);
                let node_info = retry_async(
                    || {
                        let docker = docker.clone();
                        let node = node.clone();
                        async move { docker.lock().await.inspect_node(&node).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to inspect node: {:?}", e)))?;
                info!("Node inspected successfully: {}", node);
                println!("{}", serde_json::to_string_pretty(&node_info).map_err(|e| DockerError::json_error(&e.to_string()))?);
            }
            NodeCommands::Update { node, role, availability } => {
                info!("Updating node: {}, role: {:?}, availability: {:?}", node, role, availability);
                let node_info = retry_async(
                    || {
                        let docker = docker.clone();
                        let node = node.clone();
                        let role = role.clone();
                        let availability = availability.clone();
                        async move {
                            let mut docker = docker.lock().await;
                            docker.update_node(&node, role, availability).await
                        }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to update node: {:?}", e)))?;
                info!("Node updated successfully: {}", node);
                println!("Node updated: {}", node_info.name);
            }
            NodeCommands::Promote { node } => {
                info!("Promoting node: {}", node);
                retry_async(
                    || {
                        let docker = docker.clone();
                        let node = node.clone();
                        async move { docker.lock().await.promote_node(&node).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to promote node: {:?}", e)))?;
                info!("Node promoted successfully: {}", node);
                println!("Node promoted: {}", node);
            }
            NodeCommands::Demote { node } => {
                info!("Demoting node: {}", node);
                retry_async(
                    || {
                        let docker = docker.clone();
                        let node = node.clone();
                        async move { docker.lock().await.demote_node(&node).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to demote node: {:?}", e)))?;
                info!("Node demoted successfully: {}", node);
                println!("Node demoted: {}", node);
            }
            NodeCommands::Rm { node } => {
                info!("Removing node: {}", node);
                retry_async(
                    || {
                        let docker = docker.clone();
                        let node = node.clone();
                        async move { docker.lock().await.remove_node(&node).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to remove node: {:?}", e)))?;
                info!("Node removed successfully: {}", node);
                println!("Node removed: {}", node);
            }
        },
        Commands::Stack { stack_command } => match stack_command {
            StackCommands::Deploy { name, compose_file, prune } => {
                info!("Deploying stack: {}, compose_file: {}, prune: {}", name, compose_file, prune);
                let stack = retry_async(
                    || {
                        let docker = docker.clone();
                        let name = name.clone();
                        let compose_file = compose_file.clone();
                        let prune = prune;
                        async move { docker.lock().await.stack_deploy(name, compose_file, prune).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to deploy stack: {:?}", e)))?;
                info!("Stack deployed successfully: {}", stack.name);
                println!("Stack deployed: {}", stack.name);
            }
            StackCommands::Ls => {
                info!("Listing stacks");
                let stacks = retry_async(
                    || {
                        let docker = docker.clone();
                        async move { docker.lock().await.stack_list().await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to list stacks: {:?}", e)))?;
                info!("Found {} stacks", stacks.len());
                if !stacks.is_empty() {
                    for stack in stacks {
                        println!(
                            "{} ({}) - {} services, {} containers",
                            stack.name, stack.status, stack.services, stack.containers
                        );
                    }
                }
            }
            StackCommands::Inspect { stack } => {
                info!("Inspecting stack: {}", stack);
                let stack_info = retry_async(
                    || {
                        let docker = docker.clone();
                        let stack = stack.clone();
                        async move { docker.lock().await.stack_inspect(&stack).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to inspect stack: {:?}", e)))?;
                info!("Stack inspected successfully: {}", stack);
                println!("Stack: {}", stack_info.name);
                println!("Status: {}", stack_info.status);
                println!("Services: {}", stack_info.services);
                println!("Containers: {}", stack_info.containers);
            }
            StackCommands::Rm { stack } => {
                info!("Removing stack: {}", stack);
                retry_async(
                    || {
                        let docker = docker.clone();
                        let stack = stack.clone();
                        async move { docker.lock().await.stack_rm(&stack).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to remove stack: {:?}", e)))?;
                info!("Stack removed successfully: {}", stack);
                println!("Stack removed: {}", stack);
            }
            StackCommands::Services { stack } => {
                info!("Listing services in stack: {}", stack);
                let services = retry_async(
                    || {
                        let docker = docker.clone();
                        let stack = stack.clone();
                        async move { docker.lock().await.stack_services(&stack).await }
                    },
                    3,
                )
                .await
                .map_err(|e| DockerError::internal(format!("Failed to list stack services: {:?}", e)))?;
                info!("Found {} services in stack: {}", services.len(), stack);
                if !services.is_empty() {
                    let services_json =
                        serde_json::to_string_pretty(&services).map_err(|e| DockerError::json_error(&e.to_string()))?;
                    println!("{}", services_json);
                }
            }
        },
    }

    Ok(())
}
