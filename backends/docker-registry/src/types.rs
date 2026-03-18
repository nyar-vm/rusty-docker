#![warn(missing_docs)]

//! Docker Hub 类型定义
//!
//! 定义了与 Docker Hub 交互所需的各种数据结构。

use serde::{Deserialize, Serialize};

/// Docker Hub 镜像 Manifest
///
/// 表示 Docker 镜像的元数据，包含镜像的架构、操作系统、配置和层信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageManifest {
    /// 镜像的架构，如 amd64、arm64 等
    pub architecture: String,
    /// 镜像的操作系统，如 linux、windows 等
    pub os: String,
    /// 镜像的配置信息
    pub config: ManifestConfig,
    /// 镜像的层列表
    pub layers: Vec<ManifestLayer>,
}

/// Manifest 配置
///
/// 表示镜像配置的元数据，包含配置文件的媒体类型、大小和摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestConfig {
    /// 配置文件的媒体类型
    pub media_type: String,
    /// 配置文件的大小（字节）
    pub size: u64,
    /// 配置文件的摘要
    pub digest: String,
}

/// Manifest 层
///
/// 表示镜像层的元数据，包含层的媒体类型、大小和摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestLayer {
    /// 层文件的媒体类型
    pub media_type: String,
    /// 层文件的大小（字节）
    pub size: u64,
    /// 层文件的摘要
    pub digest: String,
}

/// Docker Hub 认证响应
///
/// 表示 Docker Hub 认证 API 的响应，包含认证令牌和相关信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// 认证令牌，用于后续 API 请求的授权
    pub token: String,
    /// 访问令牌（可选）
    pub access_token: Option<String>,
    /// 令牌过期时间（可选）
    pub expires_in: Option<u32>,
    /// 令牌作用域（可选）
    pub scope: Option<String>,
    /// 令牌类型（可选）
    pub token_type: Option<String>,
}

/// 下载进度
///
/// 表示镜像下载的进度信息，包含已下载大小、总大小和完成百分比。
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// 已下载的大小（字节）
    pub downloaded: u64,
    /// 总大小（字节）
    pub total: u64,
    /// 完成百分比（0-100）
    pub percentage: f64,
}
