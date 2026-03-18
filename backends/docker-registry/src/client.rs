#![warn(missing_docs)]

//! 镜像仓库 API 客户端
//!
//! 提供与镜像仓库 API 交互的客户端实现，支持 Docker Hub 和其他仓库类型。

use std::{sync::Arc, time::Duration};

use wae_request::{HttpClient, HttpClientConfig, HttpResponse, RequestBuilder};

use crate::types::{AuthResponse, ImageManifest};
use docker_types::{DockerError, Result};

/// 镜像仓库客户端 trait
///
/// 定义了与镜像仓库交互的核心方法，包括获取镜像 manifest 和下载镜像层。
pub trait RegistryClient {
    /// 获取镜像 Manifest
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `tag`: 镜像标签
    ///
    /// # 返回
    /// - `Result<ImageManifest>`: 成功返回镜像 manifest，失败返回错误
    async fn get_manifest(&self, image: &str, tag: &str) -> Result<ImageManifest>;

    /// 下载镜像层
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `digest`: 镜像层摘要
    ///
    /// # 返回
    /// - `Result<HttpResponse>`: 成功返回 HTTP 响应，失败返回错误
    async fn download_layer(&self, image: &str, digest: &str) -> Result<HttpResponse>;

    /// 下载镜像层（支持续点续传）
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `digest`: 镜像层摘要
    /// - `start`: 开始下载的位置
    ///
    /// # 返回
    /// - `Result<HttpResponse>`: 成功返回 HTTP 响应，失败返回错误
    async fn download_layer_with_range(&self, image: &str, digest: &str, start: u64) -> Result<HttpResponse>;

    /// 获取基本 URL
    ///
    /// # 返回
    /// - `&str`: 仓库的基本 URL
    fn get_base_url(&self) -> &str;
}

/// Docker Hub 客户端
///
/// 实现了与 Docker Hub 仓库交互的客户端，支持获取镜像 manifest 和下载镜像层。
pub struct DockerHubClient {
    /// HTTP 客户端，用于发送 API 请求
    client: HttpClient,
    /// Docker Hub 基本 URL
    base_url: String,
}

impl DockerHubClient {
    /// 创建新的 Docker Hub 客户端
    ///
    /// # 返回
    /// - `Result<Self>`: 成功返回客户端实例，失败返回错误
    pub fn new() -> Result<Self> {
        let config = HttpClientConfig {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            ..Default::default()
        };

        let client = HttpClient::new(config);

        Ok(Self { client, base_url: "https://registry-1.docker.io".to_string() })
    }

    /// 获取镜像 Manifest（内部实现）
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `tag`: 镜像标签
    ///
    /// # 返回
    /// - `Result<ImageManifest>`: 成功返回镜像 manifest，失败返回错误
    async fn get_manifest_internal(&self, image: &str, tag: &str) -> Result<ImageManifest> {
        let url = format!("{}/v2/{}/manifests/{}", self.base_url, image, tag);

        // 重试逻辑
        for attempt in 0..3 {
            let response = match self
                .client
                .get_with_headers(
                    &url,
                    [("Accept".to_string(), "application/vnd.docker.distribution.manifest.v2+json".to_string())].into(),
                )
                .await
            {
                Ok(response) => response,
                Err(e) => {
                    if attempt < 2 {
                        tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                        continue;
                    }
                    return Err(DockerError::request_error(&url, e.to_string()).into());
                }
            };

            if !response.is_success() {
                // 如果需要认证，获取认证令牌
                if response.status == 401 {
                    if let Some(www_auth) = response.headers.get("www-authenticate") {
                        let auth_header = www_auth;
                        let token = self.get_auth_token(auth_header, image).await?;

                        // 使用令牌重新请求
                        let response = self
                            .client
                            .get_with_headers(
                                &url,
                                [
                                    ("Accept".to_string(), "application/vnd.docker.distribution.manifest.v2+json".to_string()),
                                    ("Authorization".to_string(), format!("Bearer {}", token)),
                                ]
                                .into(),
                            )
                            .await
                            .map_err(|e| DockerError::request_error(&url, e.to_string()))?;

                        let manifest =
                            response.json::<ImageManifest>().map_err(|e| DockerError::request_error(&url, e.to_string()))?;
                        return Ok(manifest);
                    }
                }
                return Err(DockerError::registry_error(format!("Failed to get manifest: {}", response.status)).into());
            }

            let manifest = response.json::<ImageManifest>().map_err(|e| DockerError::json_error(e.to_string()))?;
            return Ok(manifest);
        }

        Err(DockerError::registry_error("Failed to get manifest after multiple attempts".to_string()).into())
    }

    /// 获取认证令牌
    ///
    /// # 参数
    /// - `auth_header`: WWW-Authenticate 头信息
    /// - `image`: 镜像名称
    ///
    /// # 返回
    /// - `Result<String>`: 成功返回认证令牌，失败返回错误
    async fn get_auth_token(&self, auth_header: &str, image: &str) -> Result<String> {
        // 解析 WWW-Authenticate 头
        let parts: Vec<&str> = auth_header.split(' ').collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Err(DockerError::registry_error("Invalid WWW-Authenticate header".to_string()).into());
        }

        let params: Vec<&str> = parts[1].split(',').collect();
        let mut realm = "";
        let mut service = "";
        let mut scope = "";

        for param in params {
            let param_parts: Vec<&str> = param.split('=').collect();
            if param_parts.len() == 2 {
                let key = param_parts[0].trim();
                let value = param_parts[1].trim().trim_matches('"');
                match key {
                    "realm" => realm = value,
                    "service" => service = value,
                    "scope" => scope = value,
                    _ => {}
                }
            }
        }

        if realm.is_empty() || service.is_empty() {
            return Err(DockerError::registry_error("Invalid WWW-Authenticate header".to_string()).into());
        }

        let scope = if scope.is_empty() { format!("repository:{}/pull", image) } else { scope.to_string() };

        let auth_url = format!("{}?service={}&scope={}", realm, service, scope);

        let response = self.client.get(&auth_url).await.map_err(|e| DockerError::request_error(&auth_url, e.to_string()))?;

        if !response.is_success() {
            return Err(DockerError::registry_error(format!("Failed to get auth token: {}", response.status)).into());
        }

        let auth_response = response.json::<AuthResponse>().map_err(|e| DockerError::json_error(e.to_string()))?;
        Ok(auth_response.token)
    }

    /// 下载镜像层（内部实现）
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `digest`: 镜像层摘要
    ///
    /// # 返回
    /// - `Result<HttpResponse>`: 成功返回 HTTP 响应，失败返回错误
    async fn download_layer_internal(&self, image: &str, digest: &str) -> Result<HttpResponse> {
        let url = format!("{}/v2/{}/blobs/{}", self.base_url, image, digest);

        // 重试逻辑
        for attempt in 0..3 {
            let response = match self.client.get(&url).await {
                Ok(response) => response,
                Err(e) => {
                    if attempt < 2 {
                        tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                        continue;
                    }
                    return Err(DockerError::request_error(&url, e.to_string()).into());
                }
            };

            if !response.is_success() {
                // 如果需要认证，获取认证令牌
                if response.status == 401 {
                    if let Some(www_auth) = response.headers.get("www-authenticate") {
                        let auth_header = www_auth;
                        let token = self.get_auth_token(auth_header, image).await?;

                        // 使用令牌重新请求
                        let response = self
                            .client
                            .get_with_headers(&url, [("Authorization".to_string(), format!("Bearer {}", token))].into())
                            .await
                            .map_err(|e| DockerError::request_error(&url, e.to_string()))?;

                        return Ok(response);
                    }
                }
                return Err(DockerError::registry_error(format!("Failed to download layer: {}", response.status)).into());
            }

            return Ok(response);
        }

        Err(DockerError::registry_error("Failed to download layer after multiple attempts".to_string()).into())
    }

    /// 下载镜像层（支持续点续传，内部实现）
    ///
    /// # 参数
    /// - `image`: 镜像名称
    /// - `digest`: 镜像层摘要
    /// - `start`: 开始下载的位置
    ///
    /// # 返回
    /// - `Result<HttpResponse>`: 成功返回 HTTP 响应，失败返回错误
    async fn download_layer_with_range_internal(&self, image: &str, digest: &str, start: u64) -> Result<HttpResponse> {
        let url = format!("{}/v2/{}/blobs/{}", self.base_url, image, digest);

        // 重试逻辑
        for attempt in 0..3 {
            let response =
                match self.client.get_with_headers(&url, [("Range".to_string(), format!("bytes={}-", start))].into()).await {
                    Ok(response) => response,
                    Err(e) => {
                        if attempt < 2 {
                            tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                            continue;
                        }
                        return Err(DockerError::request_error(&url, e.to_string()).into());
                    }
                };

            if !response.is_success() && response.status != 206 {
                // 如果需要认证，获取认证令牌
                if response.status == 401 {
                    if let Some(www_auth) = response.headers.get("www-authenticate") {
                        let auth_header = www_auth;
                        let token = self.get_auth_token(auth_header, image).await?;

                        // 使用令牌重新请求
                        let response = self
                            .client
                            .get_with_headers(
                                &url,
                                [
                                    ("Range".to_string(), format!("bytes={}-", start)),
                                    ("Authorization".to_string(), format!("Bearer {}", token)),
                                ]
                                .into(),
                            )
                            .await
                            .map_err(|e| DockerError::request_error(&url, e.to_string()))?;

                        return Ok(response);
                    }
                }
                return Err(DockerError::registry_error(format!("Failed to download layer: {}", response.status)).into());
            }

            return Ok(response);
        }

        Err(DockerError::registry_error("Failed to download layer after multiple attempts".to_string()).into())
    }
}

impl Default for DockerHubClient {
    /// 创建默认的 Docker Hub 客户端
    ///
    /// # 注意
    /// 如果创建失败，会直接 panic
    fn default() -> Self {
        Self::new().expect("Failed to create DockerHubClient")
    }
}

impl RegistryClient for DockerHubClient {
    async fn get_manifest(&self, image: &str, tag: &str) -> Result<ImageManifest> {
        self.get_manifest_internal(image, tag).await
    }

    async fn download_layer(&self, image: &str, digest: &str) -> Result<HttpResponse> {
        self.download_layer_internal(image, digest).await
    }

    async fn download_layer_with_range(&self, image: &str, digest: &str, start: u64) -> Result<HttpResponse> {
        self.download_layer_with_range_internal(image, digest, start).await
    }

    fn get_base_url(&self) -> &str {
        &self.base_url
    }
}
