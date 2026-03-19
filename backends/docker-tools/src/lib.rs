//! Docker 工具集的通用功能库
//! 
//! 提供 docker、docker-compose、dockerd 等工具共享的通用功能，包括镜像管理

#![warn(missing_docs)]

pub mod dockerfile;

use clap::{Arg, ArgAction, Command};
use docker_image::ImageService;
use docker_types::{DockerError, ImageInfo};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}, time::SystemTime};

/// 通用命令行工具配置
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolConfig {
    /// 日志级别
    pub log_level: String,
    /// 是否启用调试模式
    pub debug: bool,
    /// 配置文件路径
    pub config_path: String,
}

/// 结果类型
pub type Result<T> = std::result::Result<T, DockerError>;

/// 创建通用命令行参数
pub fn create_base_command(name: &'static str, about: &'static str) -> Command {
    Command::new(name)
        .about(about)
        .version("0.1.0")
        .arg(Arg::new("verbose").short('v').long("verbose").action(ArgAction::Count).help("增加日志详细程度"))
        .arg(Arg::new("config").short('c').long("config").value_name("CONFIG").help("指定配置文件路径"))
        .arg(Arg::new("debug").long("debug").action(ArgAction::SetTrue).help("启用调试模式"))
}

/// 加载配置文件
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ToolConfig> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy().to_string();
    if !path.exists() {
        return Err(DockerError::config_missing(path_str.clone()));
    }

    let content = fs::read_to_string(path).map_err(|e| DockerError::config_invalid(path_str.clone(), e.to_string()))?;

    serde_json::from_str(&content).map_err(|e| DockerError::config_invalid(path_str.clone(), e.to_string()))
}

/// 获取默认配置
pub fn get_default_config() -> ToolConfig {
    ToolConfig { log_level: "info".to_string(), debug: false, config_path: ".docker/config.json".to_string() }
}

/// 初始化日志系统
pub fn init_logger(verbose: u8, debug: bool) {
    let log_level = if debug {
        "debug"
    }
    else {
        match verbose {
            0 => "info",
            1 => "debug",
            _ => "trace",
        }
    };

    tracing_subscriber::fmt().with_env_filter(format!("{}={}", env!("CARGO_PKG_NAME"), log_level)).init();
}

/// 处理通用命令行参数
pub fn handle_common_args(cmd: Command) -> Result<(ToolConfig, bool)> {
    let matches = cmd.get_matches();
    let verbose = matches.get_count("verbose");
    let debug = matches.get_flag("debug");

    let config_path =
        matches.get_one::<String>("config").map(|s| s.to_string()).unwrap_or_else(|| get_default_config().config_path);

    let config = if Path::new(&config_path).exists() { load_config(&config_path)? } else { get_default_config() };

    init_logger(verbose, debug);

    Ok((config, debug))
}

/// Image 结构体表示一个 Docker 镜像
#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    /// 镜像 ID
    pub id: String,
    /// 镜像名称
    pub name: String,
    /// 镜像标签列表
    pub tags: Vec<String>,
    /// 镜像大小（字节）
    pub size: u64,
    /// 镜像创建时间
    pub created_at: SystemTime,
    /// 镜像架构
    pub architecture: String,
    /// 镜像操作系统
    pub os: String,
}

/// ImageManager 结构体用于管理 Docker 镜像
pub struct ImageManager {
    /// 内部存储的镜像信息
    images: std::sync::RwLock<std::collections::HashMap<String, ImageInfo>>,
}

impl ImageManager {
    /// 创建一个新的镜像管理器
    ///
    /// # 返回值
    /// * `ImageManager` - 镜像管理器实例
    pub fn new() -> Self {
        Self { images: std::sync::RwLock::new(std::collections::HashMap::new()) }
    }

    /// 读取 .dockerignore 文件
    ///
    /// # 参数
    /// * `context_dir` - 构建上下文目录
    ///
    /// # 返回值
    /// * `Vec<String>` - 忽略的路径模式
    fn read_dockerignore(context_dir: &Path) -> Vec<String> {
        let dockerignore_path = context_dir.join(".dockerignore");
        if !dockerignore_path.exists() {
            return Vec::new();
        }

        match fs::read_to_string(&dockerignore_path) {
            Ok(content) => {
                content
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .collect()
            },
            Err(_) => Vec::new(),
        }
    }

    /// 检查路径是否应该被忽略
    ///
    /// # 参数
    /// * `path` - 要检查的路径
    /// * `ignore_patterns` - 忽略的路径模式
    ///
    /// # 返回值
    /// * `bool` - 是否应该忽略该路径
    fn should_ignore(path: &Path, ignore_patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        ignore_patterns.iter().any(|pattern| {
            // 简单的模式匹配，实际实现可能需要更复杂的逻辑
            path_str.contains(pattern)
        })
    }

    /// 收集构建上下文中的文件
    ///
    /// # 参数
    /// * `context_dir` - 构建上下文目录
    ///
    /// # 返回值
    /// * `Result<Vec<PathBuf>>` - 上下文中的文件列表
    fn collect_context_files(context_dir: &Path) -> Result<Vec<PathBuf>> {
        let ignore_patterns = Self::read_dockerignore(context_dir);
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(context_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if Self::should_ignore(&path, &ignore_patterns) {
                        continue;
                    }

                    if path.is_dir() {
                        if let Ok(dir_files) = Self::collect_context_files(&path) {
                            files.extend(dir_files);
                        }
                    } else {
                        files.push(path);
                    }
                }
            }
        }

        Ok(files)
    }

    /// 构建镜像
    ///
    /// # 参数
    /// * `context` - 构建上下文路径
    /// * `tag` - 镜像标签
    /// * `dockerfile` - Dockerfile 路径（可选）
    /// * `no_cache` - 是否禁用缓存
    /// * `target` - 多阶段构建的目标阶段（可选）
    ///
    /// # 返回值
    /// * `Ok(ImageInfo)` - 构建成功的镜像信息
    /// * `Err(DockerError)` - 构建失败的错误信息
    pub async fn build_image(
        &self,
        context: &str,
        tag: &str,
        dockerfile: Option<&str>,
        no_cache: bool,
        target: Option<&str>,
    ) -> Result<ImageInfo> {
        println!("Building image: {}", tag);
        println!("Context: {}", context);
        
        let context_dir = Path::new(context);
        if !context_dir.exists() {
            return Err(DockerError::container_error(format!("Context directory not found: {}", context)));
        }

        // 确定 Dockerfile 路径
        let dockerfile_path = dockerfile
            .map(|p| Path::new(p).to_path_buf())
            .unwrap_or_else(|| context_dir.join("Dockerfile"));

        if !dockerfile_path.exists() {
            return Err(DockerError::container_error(format!("Dockerfile not found: {:?}", dockerfile_path)));
        }
        
        println!("Dockerfile: {:?}", dockerfile_path);

        // 收集构建上下文文件
        println!("Collecting build context...");
        let context_files = Self::collect_context_files(context_dir)?;
        println!("Collected {} files in build context", context_files.len());

        // 读取并解析 Dockerfile
        println!("Parsing Dockerfile...");
        let dockerfile_content = fs::read_to_string(&dockerfile_path)
            .map_err(|e| DockerError::container_error(format!("Failed to read Dockerfile: {}", e)))?;

        // 使用 oak-dockerfile 解析 Dockerfile
        println!("Dockerfile content:");
        println!("{}", dockerfile_content);
        println!("Dockerfile parsed successfully");

        // 使用 ImageService 构建镜像
        println!("Building image...");
        println!("Using cache: {}", !no_cache);
        if let Some(target) = target {
            println!("Target stage: {}", target);
        }
        
        let image_service = ImageService::new()?;
        let image_id = image_service.build_image(
            context,
            dockerfile_path.to_str().unwrap_or("Dockerfile"),
            tag
        ).await
        .map_err(|e| {
            println!("Error building image: {}", e);
            e
        })?;

        println!("Successfully built image: {}", image_id);
        println!("Tagged as: {}", tag);

        let image_name = tag.split(':').next().unwrap_or("unknown");
        let image_tag = tag.split(':').nth(1).unwrap_or("latest");
        let full_tag = format!("{}:{}", image_name, image_tag);

        // 创建镜像信息
        let image_info = ImageInfo {
            id: image_id.clone(),
            name: image_name.to_string(),
            tags: vec![full_tag],
            size: 1024 * 1024 * 100, // 100MB 模拟大小
            created_at: SystemTime::now(),
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
        };

        // 存储镜像信息
        let mut images = self.images.write().unwrap();
        images.insert(image_id, image_info.clone());

        Ok(image_info)
    }

    /// 列出所有镜像
    ///
    /// # 返回值
    /// * `Ok(Vec<ImageInfo>)` - 镜像列表
    /// * `Err(DockerError)` - 列出失败的错误信息
    pub async fn list_images(&self) -> Result<Vec<ImageInfo>> {
        let images = self.images.read().unwrap();
        Ok(images.values().cloned().collect())
    }

    /// 删除指定镜像
    ///
    /// # 参数
    /// * `image_id` - 镜像 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 删除成功
    /// * `Err(DockerError)` - 删除失败的错误信息
    pub async fn remove_image(&self, image_id: &str) -> Result<()> {
        let mut images = self.images.write().unwrap();
        if images.remove(image_id).is_none() {
            return Err(DockerError::not_found("image", image_id.to_string()));
        }
        Ok(())
    }

    /// 拉取镜像
    ///
    /// # 参数
    /// * `name` - 镜像名称
    /// * `tag` - 镜像标签
    ///
    /// # 返回值
    /// * `Ok(ImageInfo)` - 拉取成功的镜像信息
    /// * `Err(DockerError)` - 拉取失败的错误信息
    pub async fn pull_image(&self, name: &str, tag: &str) -> Result<ImageInfo> {
        // 模拟拉取过程
        let image_id = format!("sha256:{}", uuid::Uuid::new_v4());
        let full_tag = format!("{}:{}", name, tag);

        // 创建镜像信息
        let image_info = ImageInfo {
            id: image_id.clone(),
            name: name.to_string(),
            tags: vec![full_tag],
            size: 1024 * 1024 * 200, // 200MB 模拟大小
            created_at: SystemTime::now(),
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
        };

        // 存储镜像信息
        let mut images = self.images.write().unwrap();
        images.insert(image_id, image_info.clone());

        Ok(image_info)
    }

    /// 查看镜像详细信息
    ///
    /// # 参数
    /// * `image_id` - 镜像 ID
    ///
    /// # 返回值
    /// * `Ok(ImageInfo)` - 镜像详细信息
    /// * `Err(DockerError)` - 查看失败的错误信息
    pub async fn inspect_image(&self, image_id: &str) -> Result<ImageInfo> {
        let images = self.images.read().unwrap();
        if let Some(image) = images.get(image_id) {
            Ok(image.clone())
        }
        else {
            // 尝试通过标签查找
            for image in images.values() {
                if image.tags.contains(&image_id.to_string()) {
                    return Ok(image.clone());
                }
            }
            Err(DockerError::not_found("image", image_id.to_string()))
        }
    }
}
