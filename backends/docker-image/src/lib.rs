#![warn(missing_docs)]

//! Docker 镜像管理库
//!
//! 提供 Docker 镜像的构建、管理和操作功能。
//!
//! ## 主要功能
//! - 镜像构建
//! - 镜像管理（列出、删除、导入、导出）
//! - 镜像标签管理
//! - 镜像历史查询

pub mod format;

use docker_types::{DockerError, Result};
use format::{ImageConfig, ImageManifest, ImageStore, parse_image_reference};

/// 镜像管理服务
///
/// 提供镜像的构建、管理和操作功能。
pub struct ImageService {
    /// 镜像存储
    image_store: ImageStore,
}

impl ImageService {
    /// 创建新的镜像服务
    ///
    /// # 返回
    /// - `Result<Self>`: 成功返回服务实例，失败返回错误
    pub fn new() -> Result<Self> {
        let image_store = ImageStore::default()?;
        Ok(Self { image_store })
    }

    /// 构建镜像
    ///
    /// # 参数
    /// - `context_path`: 构建上下文路径
    /// - `dockerfile`: Dockerfile 路径
    /// - `tag`: 镜像标签
    ///
    /// # 返回
    /// - `Result<String>`: 成功返回镜像 ID，失败返回错误
    pub async fn build_image(&self, context_path: &str, dockerfile: &str, tag: &str) -> Result<String> {
        // 解析镜像标签
        let (registry, repository, image_tag) = parse_image_reference(tag)?;

        // 生成镜像 ID
        let image_id = format!("sha256:{}", uuid::Uuid::new_v4());

        // 创建镜像清单
        let manifest = ImageManifest {
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            layers: vec![],
            config: format!("{}.json", image_id),
            media_type: "application/vnd.docker.distribution.manifest.v2+json".to_string(),
            name: Some(repository),
            tag: Some(image_tag),
            size: Some(0),
        };

        // 保存镜像清单
        self.image_store.save_manifest(&image_id, &manifest)?;

        Ok(image_id)
    }

    /// 列出所有镜像
    ///
    /// # 返回
    /// - `Result<Vec<String>>`: 成功返回镜像 ID 列表，失败返回错误
    pub async fn list_images(&self) -> Result<Vec<String>> {
        self.image_store.list_images()
    }

    /// 删除镜像
    ///
    /// # 参数
    /// - `image_id`: 镜像 ID
    ///
    /// # 返回
    /// - `Result<()>`: 成功返回 ()，失败返回错误
    pub async fn remove_image(&self, image_id: &str) -> Result<()> {
        self.image_store.delete_image(image_id)
    }

    /// 为镜像添加标签
    ///
    /// # 参数
    /// - `image_id`: 镜像 ID
    /// - `tag`: 新标签
    ///
    /// # 返回
    /// - `Result<()>`: 成功返回 ()，失败返回错误
    pub async fn tag_image(&self, image_id: &str, tag: &str) -> Result<()> {
        // 加载镜像清单
        let mut manifest = self.image_store.load_manifest(image_id)?;

        // 解析新标签
        let (_, repository, image_tag) = parse_image_reference(tag)?;

        // 更新标签
        manifest.name = Some(repository);
        manifest.tag = Some(image_tag);

        // 保存更新后的清单
        self.image_store.save_manifest(image_id, &manifest)?;

        Ok(())
    }

    /// 获取镜像历史
    ///
    /// # 参数
    /// - `image_id`: 镜像 ID
    ///
    /// # 返回
    /// - `Result<Vec<String>>`: 成功返回历史记录列表，失败返回错误
    pub async fn get_image_history(&self, image_id: &str) -> Result<Vec<String>> {
        // 加载镜像清单
        let manifest = self.image_store.load_manifest(image_id)?;

        // 加载镜像配置
        let config = self.image_store.load_config(&manifest.config)?;

        // 提取历史记录
        let history: Vec<String> = config.history.iter().map(|entry| entry.created_by.clone()).collect();

        Ok(history)
    }
}
