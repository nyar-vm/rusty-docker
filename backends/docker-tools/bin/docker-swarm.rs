use clap::{Parser, Subcommand};
use docker::Docker;
use docker_types;
use serde_json::to_string_pretty;

/// Docker Swarm 命令行工具
///
/// 用于管理 Docker 集群，支持初始化、加入、离开集群，以及管理服务和节点
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化 Swarm 集群
    Init {
        ///  advertise address
        #[arg(long)]
        advertise_addr: Option<String>,
        /// Auto-lock manager
        #[arg(long)]
        auto_lock: bool,
        /// Default address pool in CIDR format
        #[arg(long)]
        default_addr_pool: Option<String>,
        /// Force new swarm creation
        #[arg(long)]
        force_new_cluster: bool,
        /// Subnet size for default address pool
        #[arg(long, default_value = "24")]
        subnet_size: u8,
    },
    /// 加入 Swarm 集群
    Join {
        /// Token for entry into the swarm
        token: String,
        /// advertise address
        #[arg(long)]
        advertise_addr: Option<String>,
        /// Listen address
        #[arg(long)]
        listen_addr: Option<String>,
        /// Manager address
        #[arg(long)]
        manager_addr: Option<String>,
    },
    /// 管理加入令牌
    JoinToken {
        /// 令牌类型 (worker/manager)
        token_type: String,
        /// 旋转令牌
        #[arg(long)]
        rotate: bool,
    },
    /// 离开 Swarm 集群
    Leave {
        /// Force leave even if this is the last manager
        #[arg(long)]
        force: bool,
    },
    /// 解锁 Swarm 集群
    Unlock {
        /// 解锁密钥
        key: String,
    },
    /// 管理解锁密钥
    UnlockKey {
        /// 旋转解锁密钥
        #[arg(long)]
        rotate: bool,
    },
    /// 管理服务
    Service {
        #[command(subcommand)]
        service_command: ServiceCommands,
    },
    /// 管理节点
    Node {
        #[command(subcommand)]
        node_command: NodeCommands,
    },
    /// 获取 Swarm 集群信息
    Info,
    /// 更新 Swarm 集群配置
    Update {
        /// Auto-lock manager
        #[arg(long)]
        auto_lock: Option<bool>,
        /// Default address pool in CIDR format
        #[arg(long)]
        default_addr_pool: Option<String>,
        /// Subnet size for default address pool
        #[arg(long)]
        subnet_size: Option<u8>,
        /// Maximum number of manager nodes
        #[arg(long)]
        max_managers: Option<u8>,
    },
}

#[derive(Subcommand)]
enum ServiceCommands {
    /// 创建服务
    Create {
        /// 服务名称
        name: String,
        /// 镜像名称
        image: String,
        /// 端口映射
        #[arg(short, long)]
        publish: Vec<String>,
        /// 副本数
        #[arg(short, long, default_value = "1")]
        replicas: u32,
        /// 环境变量
        #[arg(short, long)]
        env: Vec<String>,
        /// 挂载卷
        #[arg(short, long)]
        mount: Vec<String>,
        /// 服务模式 (replicated/global)
        #[arg(long, default_value = "replicated")]
        mode: String,
        /// 重启策略
        #[arg(long, default_value = "any")]
        restart_condition: String,
        /// 最大重试次数
        #[arg(long, default_value = "3")]
        restart_max_attempts: u32,
        /// 重试间隔 (秒)
        #[arg(long, default_value = "10")]
        restart_delay: u32,
    },
    /// 列出服务
    Ls,
    /// 查看服务详情
    Inspect {
        /// 服务名称或 ID
        service: String,
    },
    /// 更新服务
    Update {
        /// 服务名称或 ID
        service: String,
        /// 镜像名称
        #[arg(long)]
        image: Option<String>,
        /// 副本数
        #[arg(short, long)]
        replicas: Option<u32>,
        /// 重启策略
        #[arg(long)]
        restart_condition: Option<String>,
        /// 最大重试次数
        #[arg(long)]
        restart_max_attempts: Option<u32>,
        /// 重试间隔 (秒)
        #[arg(long)]
        restart_delay: Option<u32>,
    },
    /// 删除服务
    Rm {
        /// 服务名称或 ID
        service: String,
    },
    /// 扩展服务
    Scale {
        /// 服务名称和副本数 (e.g., service=3)
        service: String,
    },
    /// 查看服务日志
    Logs {
        /// 服务名称或 ID
        service: String,
        /// 显示最新的 N 行
        #[arg(short, long, default_value = "100")]
        tail: u32,
        /// 持续跟踪日志
        #[arg(short, long)]
        follow: bool,
    },
}

#[derive(Subcommand)]
enum NodeCommands {
    /// 列出节点
    Ls,
    /// 查看节点详情
    Inspect {
        /// 节点名称或 ID
        node: String,
    },
    /// 更新节点
    Update {
        /// 节点名称或 ID
        node: String,
        /// 节点角色 (manager/worker)
        #[arg(long)]
        role: Option<String>,
        /// 节点可用性 (active/pause/drain)
        #[arg(long)]
        availability: Option<String>,
    },
    /// 提升节点为 manager
    Promote {
        /// 节点名称或 ID
        node: String,
    },
    /// 降级节点为 worker
    Demote {
        /// 节点名称或 ID
        node: String,
    },
    /// 删除节点
    Rm {
        /// 节点名称或 ID
        node: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut docker = match Docker::new() {
        Ok(docker) => docker,
        Err(e) => {
            eprintln!("Error initializing Docker: {:?}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Init { advertise_addr, auto_lock, default_addr_pool, force_new_cluster, subnet_size } => {
            println!("Initializing Swarm cluster...");
            match docker.swarm_init(advertise_addr, auto_lock, default_addr_pool, force_new_cluster, subnet_size).await {
                Ok(_) => {
                    println!("Swarm initialized successfully");
                    println!("To add a worker to this swarm, run:");
                    println!(
                        "    docker swarm join --token SWMTKN-1-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx 192.168.1.100:2377"
                    );
                    println!("\nTo add a manager to this swarm, run:");
                    println!(
                        "    docker swarm join --token SWMTKN-1-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx 192.168.1.100:2377"
                    );
                }
                Err(e) => {
                    eprintln!("Error initializing Swarm: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Join { token, advertise_addr, listen_addr, manager_addr } => {
            println!("Joining Swarm cluster...");
            match docker.swarm_join(token, advertise_addr, listen_addr, manager_addr).await {
                Ok(_) => {
                    println!("Joined Swarm cluster successfully");
                }
                Err(e) => {
                    eprintln!("Error joining Swarm: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::JoinToken { token_type: _, rotate: _ } => {
            println!("Managing join token");
            println!("Join token feature not implemented yet");
        }
        Commands::Leave { force } => {
            println!("Leaving Swarm cluster...");
            match docker.swarm_leave(force).await {
                Ok(_) => {
                    println!("Left Swarm cluster successfully");
                }
                Err(e) => {
                    eprintln!("Error leaving Swarm: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Unlock { key: _ } => {
            println!("Unlocking Swarm cluster...");
            println!("Unlock feature not implemented yet");
        }
        Commands::UnlockKey { rotate: _ } => {
            println!("Managing unlock key");
            println!("Unlock key feature not implemented yet");
        }
        Commands::Service { service_command } => {
            match service_command {
                ServiceCommands::Create {
                    name,
                    image,
                    publish,
                    replicas,
                    env,
                    mount,
                    mode: _,
                    restart_condition: _,
                    restart_max_attempts: _,
                    restart_delay: _,
                } => {
                    println!("Creating service...");
                    match docker.create_service(name, image, publish, replicas, env, mount).await {
                        Ok(service) => {
                            println!("Service created successfully");
                            println!("ID: {}", service.id);
                            println!("Name: {}", service.name);
                            println!("Image: {}", service.image);
                            println!("Replicas: {}", service.replicas);
                        }
                        Err(e) => {
                            eprintln!("Error creating service: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                }
                ServiceCommands::Ls => {
                    println!("Listing services...");
                    match docker.list_services().await {
                        Ok(services) => {
                            println!(
                                "ID                  NAME                MODE                REPLICAS            IMAGE               PORTS"
                            );
                            for service in services {
                                let ports_str = service
                                    .ports
                                    .iter()
                                    .map(|(host, container)| format!("*:{}->{}tcp", host, container))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                println!(
                                    "{:<20} {:<20} {:<20} {:<20} {:<20} {:<20}",
                                    service.id,
                                    service.name,
                                    "replicated",
                                    format!("{}/{}", service.replicas, service.replicas),
                                    service.image,
                                    ports_str
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("Error listing services: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                }
                ServiceCommands::Inspect { service } => {
                    println!("Inspecting service: {}", service);
                    match docker.inspect_service(&service).await {
                        Ok(service_info) => {
                            println!("{}", to_string_pretty(&service_info).unwrap());
                        }
                        Err(e) => {
                            eprintln!("Error inspecting service: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                }
                ServiceCommands::Update {
                    service,
                    image,
                    replicas,
                    restart_condition: _,
                    restart_max_attempts: _,
                    restart_delay: _,
                } => {
                    println!("Updating service: {}", service);
                    match docker.update_service(&service, image, replicas).await {
                        Ok(service_info) => {
                            println!("Service updated successfully");
                            println!("ID: {}", service_info.id);
                            println!("Name: {}", service_info.name);
                            println!("Image: {}", service_info.image);
                            println!("Replicas: {}", service_info.replicas);
                        }
                        Err(e) => {
                            eprintln!("Error updating service: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                }
                ServiceCommands::Rm { service } => {
                    println!("Removing service: {}", service);
                    match docker.remove_service(&service).await {
                        Ok(_) => {
                            println!("Service removed successfully");
                        }
                        Err(e) => {
                            eprintln!("Error removing service: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                }
                ServiceCommands::Scale { service } => {
                    println!("Scaling service: {}", service);
                    // 解析服务名称和副本数
                    if let Some((service_name, replicas_str)) = service.split_once("=") {
                        if let Ok(replicas) = replicas_str.parse::<u32>() {
                            match docker.scale_service(service_name, replicas).await {
                                Ok(service_info) => {
                                    println!("Service scaled successfully");
                                    println!("ID: {}", service_info.id);
                                    println!("Name: {}", service_info.name);
                                    println!("Replicas: {}", service_info.replicas);
                                }
                                Err(e) => {
                                    eprintln!("Error scaling service: {:?}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        else {
                            eprintln!("Invalid replicas count: {}", replicas_str);
                            std::process::exit(1);
                        }
                    }
                    else {
                        eprintln!("Invalid service scaling format. Use service=replicas");
                        std::process::exit(1);
                    }
                }
                ServiceCommands::Logs { service: _, tail: _, follow: _ } => {
                    println!("Fetching logs for service");
                    println!("Service logs feature not implemented yet");
                }
            }
        }
        Commands::Node { node_command } => match node_command {
            NodeCommands::Ls => {
                println!("Listing nodes...");
                match docker.list_nodes().await {
                    Ok(nodes) => {
                        println!(
                            "ID                            HOSTNAME            STATUS              AVAILABILITY        MANAGER STATUS      ENGINE VERSION"
                        );
                        for node in nodes {
                            let manager_status = match node.role {
                                docker_types::NodeRole::Manager => "Leader",
                                docker_types::NodeRole::Worker => "",
                            };
                            println!(
                                "{:<32} {:<20} {:<20} {:<20} {:<20} {:<20}",
                                node.id,
                                node.name,
                                match node.status {
                                    docker_types::NodeStatus::Ready => "Ready",
                                    docker_types::NodeStatus::Down => "Down",
                                    docker_types::NodeStatus::Unknown => "Unknown",
                                },
                                match node.availability {
                                    docker_types::NodeAvailability::Active => "Active",
                                    docker_types::NodeAvailability::Pause => "Pause",
                                    docker_types::NodeAvailability::Drain => "Drain",
                                },
                                manager_status,
                                node.version
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Error listing nodes: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
            NodeCommands::Inspect { node } => {
                println!("Inspecting node: {}", node);
                match docker.inspect_node(&node).await {
                    Ok(node_info) => {
                        println!("{}", to_string_pretty(&node_info).unwrap());
                    }
                    Err(e) => {
                        eprintln!("Error inspecting node: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
            NodeCommands::Update { node, role, availability } => {
                println!("Updating node: {}", node);
                match docker.update_node(&node, role, availability).await {
                    Ok(node_info) => {
                        println!("Node updated successfully");
                        println!("ID: {}", node_info.id);
                        println!("Name: {}", node_info.name);
                        println!("Role: {:?}", node_info.role);
                        println!("Availability: {:?}", node_info.availability);
                    }
                    Err(e) => {
                        eprintln!("Error updating node: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
            NodeCommands::Promote { node } => {
                println!("Promoting node: {} to manager", node);
                match docker.promote_node(&node).await {
                    Ok(_) => {
                        println!("Node promoted successfully");
                    }
                    Err(e) => {
                        eprintln!("Error promoting node: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
            NodeCommands::Demote { node } => {
                println!("Demoting node: {} to worker", node);
                match docker.demote_node(&node).await {
                    Ok(_) => {
                        println!("Node demoted successfully");
                    }
                    Err(e) => {
                        eprintln!("Error demoting node: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
            NodeCommands::Rm { node } => {
                println!("Removing node: {}", node);
                match docker.remove_node(&node).await {
                    Ok(_) => {
                        println!("Node removed successfully");
                    }
                    Err(e) => {
                        eprintln!("Error removing node: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::Info => {
            println!("Swarm cluster info:");
            match docker.swarm_info().await {
                Ok(info) => {
                    println!("{}", to_string_pretty(&info).unwrap());
                }
                Err(e) => {
                    eprintln!("Error getting Swarm info: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Update { auto_lock, default_addr_pool, subnet_size, max_managers: _ } => {
            println!("Updating Swarm cluster...");
            match docker.swarm_update(auto_lock, default_addr_pool, subnet_size).await {
                Ok(_) => {
                    println!("Swarm cluster updated successfully");
                }
                Err(e) => {
                    eprintln!("Error updating Swarm cluster: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
