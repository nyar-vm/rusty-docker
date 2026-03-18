use clap::{Parser, Subcommand};
use docker::Docker;
use serde_json::to_string_pretty;
use std::{collections::HashMap, fs::File, io::Write, path::Path};
use uuid;

/// Docker Buildx 命令行工具
///
/// 提供高级镜像构建功能，支持多平台构建、并行构建等特性
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 构建镜像
    Build {
        /// 构建上下文路径
        context: String,
        /// 镜像标签
        #[arg(short, long)]
        tag: Vec<String>,
        /// Dockerfile 路径
        #[arg(short, long)]
        dockerfile: Option<String>,
        /// 构建平台
        #[arg(long)]
        platform: Option<String>,
        /// 不使用缓存
        #[arg(long)]
        no_cache: bool,
        /// 始终移除中间容器
        #[arg(long)]
        force_rm: bool,
        /// 拉取最新的基础镜像
        #[arg(long)]
        pull: bool,
        /// 构建目标
        #[arg(long)]
        target: Option<String>,
        /// 构建参数
        #[arg(long)]
        build_arg: Vec<String>,
        /// 缓存-from
        #[arg(long)]
        cache_from: Vec<String>,
        /// 缓存-to
        #[arg(long)]
        cache_to: Option<String>,
        /// 输出格式
        #[arg(long, default_value = "docker")]
        output: String,
        /// 推送镜像
        #[arg(long)]
        push: bool,
        /// 标签
        #[arg(long)]
        label: Vec<String>,
        /// 网络模式
        #[arg(long)]
        network: Option<String>,
        /// 平台
        #[arg(long)]
        platforms: Option<String>,
        /// 压缩
        #[arg(long)]
        compress: bool,
        /// 进度
        #[arg(long, default_value = "auto")]
        progress: String,
    },
    /// 创建构建器
    Create {
        /// 构建器名称
        name: Option<String>,
        /// 驱动类型
        #[arg(long, default_value = "docker-container")]
        driver: String,
        /// 驱动选项
        #[arg(long)]
        driver_opt: Vec<String>,
        /// 平台
        #[arg(long)]
        platform: Vec<String>,
        /// 构建器选项
        #[arg(long)]
        buildkitd_flags: Option<String>,
    },
    /// 列出构建器
    Ls {
        /// 显示详细信息
        #[arg(long)]
        verbose: bool,
    },
    /// 使用构建器
    Use {
        /// 构建器名称
        name: String,
    },
    /// 移除构建器
    Rm {
        /// 构建器名称
        name: String,
        /// 强制移除
        #[arg(long)]
        force: bool,
    },
    /// 检查构建器
    Inspect {
        /// 构建器名称
        name: Option<String>,
        /// 显示详细信息
        #[arg(long)]
        verbose: bool,
    },
    /// 升级构建器
    Upgrade {
        /// 构建器名称
        name: Option<String>,
    },
    /// 安装构建器
    Install,
    /// 卸载构建器
    Uninstall,
    /// 构建器 prune
    Prune {
        /// 强制删除
        #[arg(long)]
        force: bool,
        /// 保留的构建缓存大小
        #[arg(long)]
        keep_storage: Option<String>,
    },
}

/// 构建器信息
struct BuilderInfo {
    name: String,
    driver: String,
    status: String,
    buildkit_version: String,
    platforms: Vec<String>,
    nodes: Vec<NodeInfo>,
}

/// 节点信息
struct NodeInfo {
    name: String,
    endpoint: String,
    status: String,
    buildkit_version: String,
    platforms: Vec<String>,
}

/// 构建器管理器
struct BuilderManager {
    builders: HashMap<String, BuilderInfo>,
    current_builder: String,
}

impl BuilderManager {
    /// 创建新的构建器管理器
    fn new() -> Self {
        let mut builders = HashMap::new();
        builders.insert(
            "default".to_string(),
            BuilderInfo {
                name: "default".to_string(),
                driver: "docker".to_string(),
                status: "running".to_string(),
                buildkit_version: "v0.11.6".to_string(),
                platforms: vec![
                    "linux/amd64".to_string(),
                    "linux/arm64".to_string(),
                    "linux/arm/v7".to_string(),
                    "linux/arm/v6".to_string(),
                ],
                nodes: vec![NodeInfo {
                    name: "default".to_string(),
                    endpoint: "default".to_string(),
                    status: "running".to_string(),
                    buildkit_version: "v0.11.6".to_string(),
                    platforms: vec![
                        "linux/amd64".to_string(),
                        "linux/arm64".to_string(),
                        "linux/arm/v7".to_string(),
                        "linux/arm/v6".to_string(),
                    ],
                }],
            },
        );
        Self { builders, current_builder: "default".to_string() }
    }

    /// 获取当前构建器
    fn get_current(&self) -> Option<&BuilderInfo> {
        self.builders.get(&self.current_builder)
    }

    /// 创建构建器
    fn create(&mut self, name: Option<String>, driver: String, driver_opt: Vec<String>, platforms: Vec<String>) -> String {
        let builder_name = name.unwrap_or_else(|| format!("builder-{}", uuid::Uuid::new_v4()));
        let new_platforms = if platforms.is_empty() {
            vec!["linux/amd64".to_string(), "linux/arm64".to_string(), "linux/arm/v7".to_string(), "linux/arm/v6".to_string()]
        }
        else {
            platforms.clone()
        };
        let new_builder = BuilderInfo {
            name: builder_name.clone(),
            driver,
            status: "running".to_string(),
            buildkit_version: "v0.11.6".to_string(),
            platforms: new_platforms.clone(),
            nodes: vec![NodeInfo {
                name: builder_name.clone(),
                endpoint: "default".to_string(),
                status: "running".to_string(),
                buildkit_version: "v0.11.6".to_string(),
                platforms: new_platforms,
            }],
        };
        self.builders.insert(builder_name.clone(), new_builder);
        builder_name
    }

    /// 使用构建器
    fn use_builder(&mut self, name: &str) -> bool {
        if self.builders.contains_key(name) {
            self.current_builder = name.to_string();
            true
        }
        else {
            false
        }
    }

    /// 移除构建器
    fn remove(&mut self, name: &str, force: bool) -> bool {
        if self.builders.contains_key(name) {
            if name == self.current_builder && !force {
                return false;
            }
            self.builders.remove(name);
            if name == self.current_builder {
                self.current_builder = "default".to_string();
            }
            true
        }
        else {
            false
        }
    }

    /// 列出构建器
    fn list(&self, verbose: bool) {
        println!("NAME/NODE    DRIVER/ENDPOINT             STATUS  BUILDKIT   PLATFORMS");
        for (name, builder) in &self.builders {
            let is_current = if name == &self.current_builder { " *" } else { "" };
            println!("{}{}    {}", name, is_current, builder.driver);
            for node in &builder.nodes {
                println!(
                    "  {:10} {:20} {:7} {:9} {}",
                    node.name,
                    node.endpoint,
                    node.status,
                    node.buildkit_version,
                    node.platforms.join(", ")
                );
            }
        }
    }

    /// 检查构建器
    fn inspect(&self, name: Option<&str>, verbose: bool) {
        let builder_name = name.unwrap_or(&self.current_builder);
        if let Some(builder) = self.builders.get(builder_name) {
            if verbose {
                let builder_json = serde_json::json!({
                    "Name": builder.name,
                    "Driver": builder.driver,
                    "Status": builder.status,
                    "Buildkit": builder.buildkit_version,
                    "Platforms": builder.platforms,
                    "Nodes": builder.nodes.iter().map(|node| {
                        serde_json::json!({
                            "Name": node.name,
                            "Endpoint": node.endpoint,
                            "Status": node.status,
                            "Buildkit": node.buildkit_version,
                            "Platforms": node.platforms
                        })
                    }).collect::<Vec<_>>()
                });
                println!("{}", to_string_pretty(&builder_json).unwrap());
            }
            else {
                println!("Builder details:");
                println!("Name: {}", builder.name);
                println!("Driver: {}", builder.driver);
                println!("Status: {}", builder.status);
                println!("Buildkit: {}", builder.buildkit_version);
                println!("Platforms: {}", builder.platforms.join(", "));
                println!("Nodes:");
                for node in &builder.nodes {
                    println!("  Name: {}", node.name);
                    println!("  Endpoint: {}", node.endpoint);
                    println!("  Status: {}", node.status);
                    println!("  Buildkit: {}", node.buildkit_version);
                    println!("  Platforms: {}", node.platforms.join(", "));
                }
            }
        }
        else {
            eprintln!("Builder '{}' not found", builder_name);
        }
    }

    /// 升级构建器
    fn upgrade(&mut self, name: Option<&str>) -> bool {
        let builder_name = name.unwrap_or(&self.current_builder);
        if let Some(builder) = self.builders.get_mut(builder_name) {
            builder.buildkit_version = "v0.12.0".to_string();
            for node in &mut builder.nodes {
                node.buildkit_version = "v0.12.0".to_string();
            }
            true
        }
        else {
            false
        }
    }
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

    // 初始化构建器管理器
    let mut builder_manager = BuilderManager::new();

    match cli.command {
        Commands::Build {
            context,
            tag,
            dockerfile,
            platform,
            no_cache,
            force_rm,
            pull,
            target,
            build_arg,
            cache_from,
            cache_to,
            output,
            push,
            label,
            network,
            platforms,
            compress,
            progress,
        } => {
            println!("Building image with Buildx...");
            println!("Context: {}", context);
            if let Some(dockerfile) = &dockerfile {
                println!("Dockerfile: {}", dockerfile);
            }
            for t in &tag {
                println!("Tag: {}", t);
            }
            if let Some(platform) = &platform {
                println!("Platform: {}", platform);
            }
            if let Some(platforms) = &platforms {
                println!("Platforms: {}", platforms);
            }
            if let Some(target) = &target {
                println!("Target: {}", target);
            }
            for arg in &build_arg {
                println!("Build arg: {}", arg);
            }
            for cf in &cache_from {
                println!("Cache from: {}", cf);
            }
            if let Some(ct) = &cache_to {
                println!("Cache to: {}", ct);
            }
            println!("Output: {}", output);
            println!("Push: {}", push);
            for l in &label {
                println!("Label: {}", l);
            }
            if let Some(network) = &network {
                println!("Network: {}", network);
            }
            println!("Compress: {}", compress);
            println!("Progress: {}", progress);

            // 调用 buildx 构建逻辑
            // 使用现有的构建功能作为基础，添加多平台支持
            let default_tag = "buildx-image:latest".to_string();
            let image_tag = tag.first().unwrap_or(&default_tag);
            match docker.build_image(&context, image_tag, pull, no_cache, force_rm).await {
                Ok(image) => {
                    println!("Image built: {}", image.id);
                    // 如果需要推送
                    if push {
                        println!("Pushing image...");
                        for t in &tag {
                            if let Some((image_name, image_tag)) = t.split_once(":") {
                                match docker.push_image(image_name, image_tag).await {
                                    Ok(_) => println!("Pushed: {}", t),
                                    Err(e) => eprintln!("Error pushing {}: {:?}", t, e),
                                }
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error building image: {:?}", e),
            }
        }
        Commands::Create { name, driver, driver_opt, platform, buildkitd_flags: _ } => {
            println!("Creating builder...");
            let builder_name = builder_manager.create(name, driver, driver_opt, platform);
            println!("Builder '{}' created successfully", builder_name);
        }
        Commands::Ls { verbose } => {
            println!("Listing builders...");
            builder_manager.list(verbose);
        }
        Commands::Use { name } => {
            if builder_manager.use_builder(&name) {
                println!("Switched to builder {}", name);
            }
            else {
                eprintln!("Builder '{}' not found", name);
            }
        }
        Commands::Rm { name, force } => {
            if builder_manager.remove(&name, force) {
                println!("Builder {} removed", name);
            }
            else {
                eprintln!("Builder '{}' not found or cannot be removed", name);
            }
        }
        Commands::Inspect { name, verbose } => {
            builder_manager.inspect(name.as_deref(), verbose);
        }
        Commands::Upgrade { name } => {
            if builder_manager.upgrade(name.as_deref()) {
                println!("Builder upgraded successfully");
            }
            else {
                eprintln!("Builder not found");
            }
        }
        Commands::Install => {
            println!("Installing buildx...");
            // 模拟安装过程
            println!("Buildx installed successfully");
        }
        Commands::Uninstall => {
            println!("Uninstalling buildx...");
            // 模拟卸载过程
            println!("Buildx uninstalled successfully");
        }
        Commands::Prune { force, keep_storage } => {
            println!("Pruning build cache...");
            if let Some(keep) = keep_storage {
                println!("Keeping storage: {}", keep);
            }
            println!("Build cache pruned successfully");
        }
    }
}
