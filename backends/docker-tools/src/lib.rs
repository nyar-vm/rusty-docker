//! Docker 工具集的通用功能库
//!
//! 提供 docker、docker-compose、dockerd 等工具共享的通用功能，包括镜像管理

#![warn(missing_docs)]

use clap::{Arg, ArgAction, Command};
use docker_types::{DockerError, ImageInfo};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

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
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::Count)
                .help("增加日志详细程度"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("CONFIG")
                .help("指定配置文件路径"),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("启用调试模式"),
        )
}

/// 加载配置文件
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ToolConfig> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy().to_string();
    if !path.exists() {
        return Err(DockerError::config_missing(path_str.clone()));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| DockerError::config_invalid(path_str.clone(), e.to_string()))?;

    serde_json::from_str(&content)
        .map_err(|e| DockerError::config_invalid(path_str.clone(), e.to_string()))
}

/// 获取默认配置
pub fn get_default_config() -> ToolConfig {
    ToolConfig {
        log_level: "info".to_string(),
        debug: false,
        config_path: ".docker/config.json".to_string(),
    }
}

/// 初始化日志系统
pub fn init_logger(verbose: u8, debug: bool) {
    let log_level = if debug {
        "debug"
    } else {
        match verbose {
            0 => "info",
            1 => "debug",
            _ => "trace",
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(format!("{}={}", env!("CARGO_PKG_NAME"), log_level))
        .init();
}

/// 处理通用命令行参数
pub fn handle_common_args(cmd: Command) -> Result<(ToolConfig, bool)> {
    let matches = cmd.get_matches();
    let verbose = matches.get_count("verbose");
    let debug = matches.get_flag("debug");

    let config_path = matches
        .get_one::<String>("config")
        .map(|s| s.to_string())
        .unwrap_or_else(|| get_default_config().config_path);

    let config = if Path::new(&config_path).exists() {
        load_config(&config_path)?
    } else {
        get_default_config()
    };

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
    docker: BollardDocker,
}

impl ImageManager {
    /// 创建一个新的镜像管理器
    ///
    /// # 返回值
    /// * `ImageManager` - 镜像管理器实例
    pub fn new() -> Self {
        let docker = BollardDocker::connect_with_defaults().expect("Failed to connect to Docker");
        Self { docker }
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
        use futures_util::StreamExt;

        let options = BuildImageOptions {
            t: Some(tag.to_string()),
            dockerfile: dockerfile.unwrap_or_default().to_string(),
            nocache: no_cache,
            target: target.unwrap_or_default().to_string(),
            ..Default::default()
        };

        // 创建构建上下文 tar 归档
        let context_dir = Path::new(context);
        let mut tar = tar::Builder::new(Vec::new());
        tar.append_dir_all(".", context_dir).map_err(|e| {
            DockerError::container_error(format!("Failed to create build context: {:?}", e))
        })?;
        let _tar_data = tar.into_inner().map_err(|e| {
            DockerError::container_error(format!("Failed to create build context: {:?}", e))
        })?;

        // 传递 tar 数据作为构建上下文
        let mut stream = self.docker.build_image(options, None, None);

        // 处理构建输出
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Some(error_detail) = chunk.error_detail {
                        return Err(DockerError::container_error(
                            error_detail.message.unwrap_or_default(),
                        ));
                    }
                }
                Err(e) => {
                    return Err(DockerError::container_error(format!(
                        "Failed to build image: {:?}",
                        e
                    )));
                }
            }
        }

        let image_name = tag.split(':').next().unwrap_or("unknown");
        let image_tag = tag.split(':').nth(1).unwrap_or("latest");

        self.inspect_image(&format!("{}:{}", image_name, image_tag))
            .await
    }

    /// 列出所有镜像
    ///
    /// # 返回值
    /// * `Ok(Vec<ImageInfo>)` - 镜像列表
    /// * `Err(DockerError)` - 列出失败的错误信息
    pub async fn list_images(&self) -> Result<Vec<ImageInfo>> {
        let options = ListImagesOptions {
            all: true,
            ..Default::default()
        };

        let images =
            self.docker.list_images(Some(options)).await.map_err(|e| {
                DockerError::container_error(format!("Failed to list images: {:?}", e))
            })?;

        let images: Vec<ImageInfo> = images
            .into_iter()
            .map(|image| {
                let repo_tags = image.repo_tags;
                let default_tag = "<none>:<none>".to_string();
                let first_tag = repo_tags.first().unwrap_or(&default_tag);
                let parts: Vec<&str> = first_tag.split(':').collect();
                let name = if parts.len() > 1 && parts[0] != "<none>" {
                    parts[0].to_string()
                } else {
                    "<none>".to_string()
                };

                ImageInfo {
                    id: image.id,
                    name,
                    tags: repo_tags,
                    size: image.size as u64,
                    created_at: SystemTime::now(),     // 需要解析时间
                    architecture: "amd64".to_string(), // 需要解析
                    os: "linux".to_string(),           // 需要解析
                }
            })
            .collect();

        Ok(images)
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
        let options = RemoveImageOptions {
            force: true,
            ..Default::default()
        };

        self.docker
            .remove_image(image_id, Some(options), None)
            .await
            .map_err(|e| {
                DockerError::container_error(format!("Failed to remove image: {:?}", e))
            })?;
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
        use futures_util::StreamExt;

        let image_ref = format!("{}:{}", name, tag);
        let options = CreateImageOptions {
            from_image: Some(image_ref.clone()),
            ..Default::default()
        };

        let mut stream = self.docker.create_image(Some(options), None, None);

        // 处理拉取输出
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Some(error_detail) = chunk.error_detail {
                        return Err(DockerError::container_error(
                            error_detail.message.unwrap_or_default(),
                        ));
                    }
                }
                Err(e) => {
                    return Err(DockerError::container_error(format!(
                        "Failed to pull image: {:?}",
                        e
                    )));
                }
            }
        }

        self.inspect_image(&image_ref).await
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
        let _options = InspectImageOptions {
            ..Default::default()
        };

        let image = self.docker.inspect_image(image_id).await.map_err(|e| {
            DockerError::not_found("image", format!("Failed to inspect image: {:?}", e))
        })?;

        let repo_tags = image.repo_tags.unwrap_or_default();
        let default_tag = "<none>:<none>".to_string();
        let first_tag = repo_tags.first().unwrap_or(&default_tag);
        let parts: Vec<&str> = first_tag.split(':').collect();
        let name = if parts.len() > 1 && parts[0] != "<none>" {
            parts[0].to_string()
        } else {
            "<none>".to_string()
        };

        Ok(ImageInfo {
            id: image.id.unwrap_or_default(),
            name,
            tags: repo_tags,
            size: image.size.unwrap_or(0) as u64,
            created_at: SystemTime::now(), // 需要解析时间
            architecture: image.architecture.unwrap_or_default(),
            os: image.os.unwrap_or_default(),
        })
    }
}
