#![warn(missing_docs)]

//! 镜像下载器
//!
//! 提供镜像的并行下载和续点续传功能，支持多线程下载以提高速度。

use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    sync::Arc,
    time::Duration,
};

use futures::StreamExt;
use tokio::{
    sync::{Mutex, Semaphore},
    task,
};

use crate::{
    client::{DockerHubClient, RegistryClient},
    types::{DownloadProgress, ImageManifest},
};
use docker_types::{DockerError, ImageInfo, Result};

/// 镜像下载器
///
/// 负责下载镜像及其层，支持并行下载和续点续传。
pub struct ImageDownloader {
    /// 镜像仓库客户端，用于与仓库 API 交互
    client: Arc<DockerHubClient>,
    /// 存储根路径，用于存放下载的镜像和层
    storage_root: String,
    /// 并发下载限制，控制同时下载的层数
    max_concurrency: usize,
}

impl ImageDownloader {
    /// 创建新的镜像下载器
    ///
    /// # 参数
    /// - `client`: 镜像仓库客户端
    ///
    /// # 返回
    /// - `Result<Self>`: 成功返回下载器实例，失败返回错误
    pub fn new(client: Arc<DockerHubClient>) -> Result<Self> {
        let storage_root = "/var/lib/rusty-docker".to_string();

        // 创建存储目录
        fs::create_dir_all(&storage_root).map_err(|e| DockerError::storage_write_failed(&storage_root))?;

        // 创建子目录
        let subdirs = ["images", "layers"];
        for subdir in &subdirs {
            let dir_path = format!("{}/{}", storage_root, subdir);
            fs::create_dir_all(&dir_path).map_err(|e| DockerError::storage_write_failed(&dir_path))?;
        }

        Ok(Self { client, storage_root, max_concurrency: num_cpus::get() })
    }

    /// 下载镜像
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `tag`: 镜像标签
    ///
    /// # 返回
    /// - `Result<ImageInfo>`: 成功返回镜像信息，失败返回错误
    pub async fn download_image(&self, image: &str, tag: &str) -> Result<ImageInfo> {
        // 获取镜像 manifest
        let manifest = self.client.get_manifest(image, tag).await?;

        // 并行下载所有层
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));
        let layer_tasks = manifest.layers.iter().map(|layer| {
            let self_clone = Arc::new(self.clone());
            let semaphore = semaphore.clone();
            let image = image.to_string();
            let layer_digest = layer.digest.clone();
            let layer_size = layer.size;

            task::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                self_clone.download_layer(&image, &layer_digest, layer_size).await
            })
        });

        let results = futures::future::join_all(layer_tasks).await;
        for result in results {
            result.map_err(|e| DockerError::registry_error(e.to_string()))?;
        }

        // 创建镜像信息
        let image_info = ImageInfo {
            id: manifest.config.digest.clone(),
            name: format!("{}:{}", image, tag),
            tags: vec![format!("{}:{}", image, tag)],
            size: manifest.layers.iter().map(|l| l.size).sum(),
            created_at: std::time::SystemTime::now(),
            architecture: manifest.architecture,
            os: manifest.os,
        };

        Ok(image_info)
    }

    /// 下载单个层
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `digest`: 镜像层摘要
    /// - `size`: 镜像层大小
    ///
    /// # 返回
    /// - `Result<()>`: 成功返回 Ok(())，失败返回错误
    async fn download_layer(&self, image: &str, digest: &str, size: u64) -> Result<()> {
        let layers_dir = format!("{}/layers", self.storage_root);
        let layer_file = format!("{}/{}", layers_dir, digest.split(':').nth(1).unwrap());
        let layer_path = Path::new(&layer_file);

        // 检查文件是否已存在，计算已下载的大小
        let mut downloaded = 0;
        if layer_path.exists() {
            let metadata = fs::metadata(layer_path).map_err(|e| DockerError::storage_read_failed(&layer_file))?;
            downloaded = metadata.len();
        }

        // 如果已下载完成，直接返回
        if downloaded >= size {
            return Ok(());
        }

        // 下载层（支持续点续传）
        let response = if downloaded > 0 {
            self.client.download_layer_with_range(image, digest, downloaded).await?
        }
        else {
            self.client.download_layer(image, digest).await?
        };

        // 打开文件，追加模式
        let mut file = if downloaded > 0 {
            File::options().append(true).open(layer_path).map_err(|e| DockerError::storage_write_failed(&layer_file))?
        }
        else {
            File::create(layer_path).map_err(|e| DockerError::storage_write_failed(&layer_file))?
        };

        // 读取响应体并写入文件
        let body = response.body;
        file.write_all(&body).map_err(|e| DockerError::storage_write_failed(&layer_file))?;
        let downloaded = body.len() as u64;

        // 验证下载大小
        if downloaded != size {
            return Err(
                DockerError::registry_error(format!("Download size mismatch: expected {}, got {}", size, downloaded)).into()
            );
        }

        Ok(())
    }

    /// 获取下载进度
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `tag`: 镜像标签
    ///
    /// # 返回
    /// - `Result<DownloadProgress>`: 成功返回下载进度，失败返回错误
    pub async fn get_download_progress(&self, image: &str, tag: &str) -> Result<DownloadProgress> {
        // 获取镜像 manifest
        let manifest = self.client.get_manifest(image, tag).await?;

        let total_size = manifest.layers.iter().map(|l| l.size).sum::<u64>();
        let mut downloaded_size = 0;

        // 检查已下载的层
        let layers_dir = format!("{}/layers", self.storage_root);
        for layer in &manifest.layers {
            let layer_file = format!("{}/{}", layers_dir, layer.digest.split(':').nth(1).unwrap());
            let layer_path = Path::new(&layer_file);
            if layer_path.exists() {
                let metadata = fs::metadata(layer_path).map_err(|e| DockerError::storage_read_failed(&layer_file))?;
                downloaded_size += metadata.len();
            }
        }

        let percentage = if total_size > 0 { (downloaded_size as f64 / total_size as f64) * 100.0 } else { 0.0 };

        Ok(DownloadProgress { downloaded: downloaded_size, total: total_size, percentage })
    }
}

impl Clone for ImageDownloader {
    /// 克隆镜像下载器
    fn clone(&self) -> Self {
        Self { client: self.client.clone(), storage_root: self.storage_root.clone(), max_concurrency: self.max_concurrency }
    }
}

impl Default for ImageDownloader {
    /// 创建默认的镜像下载器
    ///
    /// # 注意
    /// 如果创建失败，会直接 panic
    fn default() -> Self {
        let client = Arc::new(DockerHubClient::default());
        Self::new(client).expect("Failed to create ImageDownloader")
    }
}
