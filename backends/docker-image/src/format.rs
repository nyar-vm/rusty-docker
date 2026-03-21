#![warn(missing_docs)]

//! Docker 镜像格式
//!
//! 实现 Docker 镜像格式的解析和管理，包括镜像清单、配置和分层存储。

use std::{collections::HashMap, fs, path::Path};

use serde::{Serialize, Deserialize};
use docker_types::{DockerError, Result};

/// 镜像清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageManifest {
    /// 架构
    pub architecture: String,
    /// 操作系统
    pub os: String,
    /// 层
    pub layers: Vec<String>,
    /// 配置
    pub config: String,
    /// 媒体类型
    pub media_type: String,
    /// 名称
    pub name: Option<String>,
    /// 标签
    pub tag: Option<String>,
    /// 大小
    pub size: Option<u64>,
}

/// 镜像配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// 架构
    pub architecture: String,
    /// 操作系统
    pub os: String,
    /// 操作系统版本
    pub os_version: Option<String>,
    /// 配置
    pub config: ContainerConfig,
    /// 容器配置
    pub container_config: Option<ContainerConfig>,
    /// 创建时间
    pub created: String,
    /// 历史
    pub history: Vec<HistoryEntry>,
    /// 层
    pub rootfs: RootfsConfig,
    /// 元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 容器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    /// 主机名
    pub Hostname: String,
    /// 域名
    pub Domainname: String,
    /// 用户
    pub User: String,
    /// 附加组
    pub AttachStdin: bool,
    /// 附加标准输出
    pub AttachStdout: bool,
    /// 附加标准错误
    pub AttachStderr: bool,
    /// 暴露端口
    pub ExposedPorts: Option<HashMap<String, serde_json::Value>>,
    /// 发布服务
    pub PublishService: Option<String>,
    /// TTY
    pub Tty: bool,
    /// 打开标准输入
    pub OpenStdin: bool,
    /// 标准输入保持打开
    pub StdinOnce: bool,
    /// 环境变量
    pub Env: Option<Vec<String>>,
    /// 命令
    pub Cmd: Option<Vec<String>>,
    /// 健康检查
    pub Healthcheck: Option<HealthcheckConfig>,
    /// 参数
    pub ArgsEscaped: Option<bool>,
    /// 镜像
    pub Image: String,
    /// 卷
    pub Volumes: Option<HashMap<String, serde_json::Value>>,
    /// 工作目录
    pub WorkingDir: String,
    /// 入口点
    pub Entrypoint: Option<Vec<String>>,
    /// 网络模式
    pub NetworkDisabled: Option<bool>,
    /// 标签
    pub Labels: Option<HashMap<String, String>>,
}

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcheckConfig {
    /// 测试
    pub Test: Vec<String>,
    /// 间隔
    pub Interval: Option<u64>,
    /// 超时
    pub Timeout: Option<u64>,
    /// 重试次数
    pub Retries: Option<u32>,
    /// 启动周期
    pub StartPeriod: Option<u64>,
}

/// 历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 创建时间
    pub created: String,
    /// 创建者
    pub created_by: String,
    /// 评论
    pub comment: Option<String>,
    /// 空层
    pub empty_layer: Option<bool>,
}

/// 根文件系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootfsConfig {
    /// 类型
    pub type_: String,
    /// 层
    pub diff_ids: Vec<String>,
}

/// 镜像层
#[derive(Debug, Clone)]
pub struct ImageLayer {
    /// 层 ID
    pub id: String,
    /// 父层 ID
    pub parent_id: Option<String>,
    /// 大小
    pub size: u64,
    /// 路径
    pub path: String,
    /// 内容哈希
    pub digest: String,
}

/// 镜像存储
pub struct ImageStore {
    /// 存储根路径
    store_path: String,
}

impl ImageStore {
    /// 创建新的镜像存储
    pub fn new(store_path: &str) -> Result<Self> {
        // 创建存储目录
        fs::create_dir_all(store_path).map_err(|e| DockerError::io_error("create_image_store", e.to_string()))?;

        // 创建子目录
        let subdirs = ["blobs", "manifests", "layers"];
        for subdir in &subdirs {
            let dir_path = format!("{}/{}", store_path, subdir);
            fs::create_dir_all(&dir_path).map_err(|e| DockerError::io_error("create_subdir", e.to_string()))?;
        }

        Ok(Self { store_path: store_path.to_string() })
    }

    /// 从默认路径创建镜像存储
    pub fn default() -> Result<Self> {
        Self::new("/var/lib/rusty-docker/images")
    }

    /// 保存镜像清单
    pub fn save_manifest(&self, image_id: &str, manifest: &ImageManifest) -> Result<()> {
        let manifest_path = format!("{}/manifests/{}.json", self.store_path, image_id);
        let content = serde_json::to_string_pretty(manifest).map_err(|e| DockerError::json_error(e.to_string()))?;

        fs::write(&manifest_path, content).map_err(|e| DockerError::io_error("save_manifest", e.to_string()))?;

        Ok(())
    }

    /// 加载镜像清单
    pub fn load_manifest(&self, image_id: &str) -> Result<ImageManifest> {
        let manifest_path = format!("{}/manifests/{}.json", self.store_path, image_id);
        let content = fs::read_to_string(&manifest_path).map_err(|e| DockerError::io_error("load_manifest", e.to_string()))?;

        let manifest: ImageManifest = serde_json::from_str(&content).map_err(|e| DockerError::json_error(e.to_string()))?;

        Ok(manifest)
    }

    /// 保存镜像配置
    pub fn save_config(&self, config_id: &str, config: &ImageConfig) -> Result<()> {
        let config_path = format!("{}/blobs/{}.json", self.store_path, config_id);
        let content = serde_json::to_string_pretty(config).map_err(|e| DockerError::json_error(e.to_string()))?;

        fs::write(&config_path, content).map_err(|e| DockerError::io_error("save_config", e.to_string()))?;

        Ok(())
    }

    /// 加载镜像配置
    pub fn load_config(&self, config_id: &str) -> Result<ImageConfig> {
        let config_path = format!("{}/blobs/{}.json", self.store_path, config_id);
        let content = fs::read_to_string(&config_path).map_err(|e| DockerError::io_error("load_config", e.to_string()))?;

        let config: ImageConfig = serde_json::from_str(&content).map_err(|e| DockerError::json_error(e.to_string()))?;

        Ok(config)
    }

    /// 保存镜像层
    pub fn save_layer(&self, layer_id: &str, layer_data: &[u8]) -> Result<()> {
        let layer_path = format!("{}/layers/{}", self.store_path, layer_id);
        fs::write(&layer_path, layer_data).map_err(|e| DockerError::io_error("save_layer", e.to_string()))?;

        Ok(())
    }

    /// 加载镜像层
    pub fn load_layer(&self, layer_id: &str) -> Result<Vec<u8>> {
        let layer_path = format!("{}/layers/{}", self.store_path, layer_id);
        let data = fs::read(&layer_path).map_err(|e| DockerError::io_error("load_layer", e.to_string()))?;

        Ok(data)
    }

    /// 列出所有镜像
    pub fn list_images(&self) -> Result<Vec<String>> {
        let manifests_dir = format!("{}/manifests", self.store_path);
        let entries = fs::read_dir(&manifests_dir).map_err(|e| DockerError::io_error("list_manifests", e.to_string()))?;

        let mut image_ids = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".json") {
                        let image_id = filename.trim_end_matches(".json");
                        image_ids.push(image_id.to_string());
                    }
                }
            }
        }

        Ok(image_ids)
    }

    /// 删除镜像
    pub fn delete_image(&self, image_id: &str) -> Result<()> {
        // 删除清单
        let manifest_path = format!("{}/manifests/{}.json", self.store_path, image_id);
        if Path::new(&manifest_path).exists() {
            fs::remove_file(&manifest_path).map_err(|e| DockerError::io_error("delete_manifest", e.to_string()))?;
        }

        // 删除相关的配置和层（这里简化处理）
        // 实际实现中需要解析清单，找到相关的配置和层并删除

        Ok(())
    }
}

/// 解析镜像引用
pub fn parse_image_reference(ref_str: &str) -> Result<(String, String, String)> {
    // 解析镜像引用格式：[registry/][repository][:tag]
    let parts: Vec<&str> = ref_str.split('/').collect();

    let mut registry = "docker.io".to_string();
    let mut repository = "".to_string();
    let mut tag = "latest".to_string();

    if parts.len() > 1 {
        // 检查第一个部分是否是注册表
        let first_part = parts[0];
        if first_part.contains('.') || first_part.contains(':') {
            registry = first_part.to_string();
            repository = parts[1..].join("/");
        }
        else {
            repository = ref_str.to_string();
        }
    }
    else {
        repository = ref_str.to_string();
    }

    // 提取标签
    let (repo, t) = repository.split_once(':').unwrap_or((&repository, "latest"));
    let repository = repo.to_string();
    let tag = t.to_string();

    Ok((registry, repository, tag))
}

/// 计算内容哈希
pub fn calculate_digest(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}


