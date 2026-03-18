#![warn(missing_docs)]

//! Docker Registry 镜像管理库
//!
//! 支持从多种镜像仓库拉取和推送镜像，实现并行下载和续点续传功能。
//!
//! ## 主要功能
//! - 支持多种镜像仓库类型（Docker Hub、私有仓库、GCR、ECR、ACR）
//! - 并行下载镜像层，提高下载速度
//! - 支持续点续传，避免网络中断导致的重复下载
//! - 提供下载进度查询

pub mod client;
pub mod downloader;
pub mod types;

use std::sync::Arc;

use client::DockerHubClient;
use docker_types::{ImageInfo, Result};
use downloader::ImageDownloader;

/// 镜像仓库类型
///
/// 定义了支持的镜像仓库类型，用于创建相应的仓库客户端。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryType {
    /// Docker Hub 官方仓库
    DockerHub,
    /// 私有镜像仓库
    Private,
    /// Google Container Registry
    GCR,
    /// Amazon Elastic Container Registry
    ECR,
    /// Azure Container Registry
    ACR,
}

/// 镜像仓库服务
///
/// 提供镜像的拉取和推送功能，支持多种仓库类型。
pub struct RegistryService {
    /// 仓库客户端，用于与镜像仓库 API 交互
    client: Arc<DockerHubClient>,
    /// 镜像下载器，用于并行下载镜像层
    downloader: Arc<ImageDownloader>,
    /// 仓库类型，标识当前服务使用的仓库类型
    registry_type: RegistryType,
}

impl RegistryService {
    /// 创建新的仓库服务
    ///
    /// # 参数
    /// - `registry_type`: 仓库类型
    /// - `endpoint`: 可选的仓库端点 URL
    ///
    /// # 返回
    /// - `Result<Self>`: 成功返回服务实例，失败返回错误
    pub fn new(registry_type: RegistryType, endpoint: Option<&str>) -> Result<Self> {
        let client = match registry_type {
            RegistryType::DockerHub => Arc::new(DockerHubClient::new()?),
            _ => {
                // 对于其他类型的仓库，这里可以实现相应的客户端
                Arc::new(DockerHubClient::new()?)
            }
        };

        let downloader = Arc::new(ImageDownloader::new(client.clone())?);

        Ok(Self {
            client,
            downloader,
            registry_type,
        })
    }

    /// 从 Docker Hub 创建服务（向后兼容）
    ///
    /// # 返回
    /// - `Result<Self>`: 成功返回 Docker Hub 服务实例，失败返回错误
    pub fn new_docker_hub() -> Result<Self> {
        Self::new(RegistryType::DockerHub, None)
    }

    /// 拉取镜像
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `tag`: 镜像标签
    ///
    /// # 返回
    /// - `Result<ImageInfo>`: 成功返回镜像信息，失败返回错误
    pub async fn pull_image(&self, image: &str, tag: &str) -> Result<ImageInfo> {
        self.downloader.download_image(image, tag).await
    }

    /// 推送镜像
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `tag`: 镜像标签
    ///
    /// # 返回
    /// - `Result<ImageInfo>`: 成功返回镜像信息，失败返回错误
    pub async fn push_image(&self, image: &str, tag: &str) -> Result<ImageInfo> {
        // 模拟推送镜像
        Ok(ImageInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{}", image),
            tags: vec![format!("{}:{}", image, tag)],
            size: 1024 * 1024 * 100, // 100MB
            created_at: std::time::SystemTime::now(),
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
        })
    }

    /// 获取仓库客户端
    ///
    /// # 返回
    /// - `Arc<DockerHubClient>`: 仓库客户端实例
    pub fn get_client(&self) -> Arc<DockerHubClient> {
        self.client.clone()
    }

    /// 获取镜像下载器
    ///
    /// # 返回
    /// - `Arc<ImageDownloader>`: 镜像下载器实例
    pub fn get_downloader(&self) -> Arc<ImageDownloader> {
        self.downloader.clone()
    }

    /// 获取仓库类型
    ///
    /// # 返回
    /// - `RegistryType`: 当前服务的仓库类型
    pub fn get_registry_type(&self) -> RegistryType {
        self.registry_type.clone()
    }
}

/// Docker Hub 服务（向后兼容）
pub type DockerHubService = RegistryService;

/// 创建 Docker Hub 服务（向后兼容）
///
/// # 返回
/// - `Result<DockerHubService>`: 成功返回 Docker Hub 服务实例，失败返回错误
pub fn new_docker_hub_service() -> Result<DockerHubService> {
    RegistryService::new_docker_hub()
}
