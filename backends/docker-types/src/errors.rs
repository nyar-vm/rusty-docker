#![warn(missing_docs)]

//! 错误类型定义
//!
//! 提供统一的错误处理机制，支持国际化和 HTTP 状态码映射。

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fmt;

/// 结果类型
pub type Result<T> = std::result::Result<T, DockerError>;

/// 中心化错误类型
///
/// 包含错误类型标识，支持国际化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerError {
    /// 错误类型
    pub kind: Box<DockerErrorKind>,
}

/// 错误分类
///
/// 用于将错误映射到 HTTP 状态码等场景。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// 验证错误 (400)
    Validation,
    /// 认证错误 (401)
    Auth,
    /// 权限错误 (403)
    Permission,
    /// 资源未找到 (404)
    NotFound,
    /// 请求冲突 (409)
    Conflict,
    /// 请求过多 (429)
    RateLimited,
    /// 网络/服务错误 (502/503)
    Network,
    /// 存储错误 (500)
    Storage,
    /// 数据库错误 (500)
    Database,
    /// 缓存错误 (500)
    Cache,
    /// 配置错误 (500)
    Config,
    /// 超时错误 (408/504)
    Timeout,
    /// 内部错误 (500)
    Internal,
}

impl ErrorCategory {
    /// 获取对应的 HTTP 状态码
    pub fn http_status(&self) -> u16 {
        match self {
            ErrorCategory::Validation => 400,
            ErrorCategory::Auth => 401,
            ErrorCategory::Permission => 403,
            ErrorCategory::NotFound => 404,
            ErrorCategory::Conflict => 409,
            ErrorCategory::RateLimited => 429,
            ErrorCategory::Network => 502,
            ErrorCategory::Storage => 500,
            ErrorCategory::Database => 500,
            ErrorCategory::Cache => 500,
            ErrorCategory::Config => 500,
            ErrorCategory::Timeout => 408,
            ErrorCategory::Internal => 500,
        }
    }
}

/// 统一错误类型枚举
///
/// 整合所有模块的错误变体，每个变体对应一个 i18n key。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockerErrorKind {
    // ========== 验证错误 ==========
    /// 无效格式
    InvalidFormat {
        /// 字段名
        field: String,
        /// 期望格式
        expected: String,
    },
    /// 值超出范围
    OutOfRange {
        /// 字段名
        field: String,
        /// 最小值
        min: Option<String>,
        /// 最大值
        max: Option<String>,
    },
    /// 必填字段缺失
    Required {
        /// 字段名
        field: String,
    },
    /// 值已存在
    AlreadyExists {
        /// 字段名
        field: String,
        /// 值
        value: String,
    },
    /// 值不允许
    NotAllowed {
        /// 字段名
        field: String,
        /// 允许的值
        allowed: Vec<String>,
    },
    /// 无效参数
    InvalidParams {
        /// 参数名
        param: String,
        /// 原因
        reason: String,
    },

    // ========== 认证错误 (Auth) ==========
    /// 无效凭证
    InvalidCredentials,
    /// 账户已锁定
    AccountLocked,
    /// 用户未找到
    UserNotFound {
        /// 用户标识
        identifier: String,
    },
    /// 用户已存在
    UserAlreadyExists {
        /// 用户名
        username: String,
    },
    /// 无效令牌
    InvalidToken {
        /// 原因
        reason: String,
    },
    /// 令牌已过期
    TokenExpired,

    // ========== 权限错误 ==========
    /// 权限拒绝
    PermissionDenied {
        /// 操作
        action: String,
    },
    /// 禁止访问
    Forbidden {
        /// 资源
        resource: String,
    },

    // ========== 资源未找到 ==========
    /// 资源未找到
    ResourceNotFound {
        /// 资源类型
        resource_type: String,
        /// 资源标识
        identifier: String,
    },

    // ========== 冲突错误 ==========
    /// 资源冲突
    ResourceConflict {
        /// 资源
        resource: String,
        /// 原因
        reason: String,
    },

    // ========== 限流错误 ==========
    /// 限流超出
    RateLimited {
        /// 限制
        limit: u64,
    },

    // ========== 网络错误 ==========
    /// 连接失败
    ConnectionFailed {
        /// 目标
        target: String,
    },
    /// DNS 解析失败
    DnsResolutionFailed {
        /// 主机
        host: String,
    },
    /// 服务不可用
    ServiceUnavailable {
        /// 服务名
        service: String,
    },

    // ========== 存储错误 ==========
    /// 读取失败
    StorageReadFailed {
        /// 路径
        path: String,
    },
    /// 写入失败
    StorageWriteFailed {
        /// 路径
        path: String,
    },
    /// 删除失败
    StorageDeleteFailed {
        /// 路径
        path: String,
    },
    /// 容量不足
    InsufficientCapacity {
        /// 需要
        required: u64,
        /// 可用
        available: u64,
    },
    /// 文件未找到
    StorageFileNotFound {
        /// 路径
        path: String,
    },

    // ========== 配置错误 ==========
    /// 配置缺失
    ConfigMissing {
        /// 配置键
        key: String,
    },
    /// 配置无效
    ConfigInvalid {
        /// 配置键
        key: String,
        /// 原因
        reason: String,
    },

    // ========== 内部错误 ==========
    /// 内部错误
    InternalError {
        /// 原因
        reason: String,
    },
    /// 未实现
    NotImplemented {
        /// 功能
        feature: String,
    },
    /// IO 错误
    IoError {
        /// 操作
        operation: String,
        /// 原因
        reason: String,
    },
    /// JSON 错误
    JsonError {
        /// 原因
        reason: String,
    },
    /// 解析错误
    ParseError {
        /// 类型
        type_name: String,
        /// 原因
        reason: String,
    },
    /// 请求错误
    RequestError {
        /// URL
        url: String,
        /// 原因
        reason: String,
    },

    // ========== Docker 特定错误 ==========
    /// 容器错误
    ContainerError {
        /// 原因
        reason: String,
    },
    /// 镜像错误
    ImageError {
        /// 原因
        reason: String,
    },
    /// 网络错误
    NetworkError {
        /// 原因
        reason: String,
    },
    /// 运行时错误
    RuntimeError {
        /// 原因
        reason: String,
    },
    /// 注册表错误
    RegistryError {
        /// 原因
        reason: String,
    },
    /// Etcd 错误
    EtcdError {
        /// 原因
        reason: String,
    },
    /// 监控错误
    MonitorError {
        /// 原因
        reason: String,
    },
    /// Kubernetes 错误
    KubernetesError {
        /// 原因
        reason: String,
    },
}

impl DockerErrorKind {
    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        match self {
            DockerErrorKind::InvalidFormat { .. }
            | DockerErrorKind::OutOfRange { .. }
            | DockerErrorKind::Required { .. }
            | DockerErrorKind::AlreadyExists { .. }
            | DockerErrorKind::NotAllowed { .. }
            | DockerErrorKind::InvalidParams { .. } => ErrorCategory::Validation,

            DockerErrorKind::InvalidCredentials
            | DockerErrorKind::AccountLocked
            | DockerErrorKind::UserNotFound { .. }
            | DockerErrorKind::UserAlreadyExists { .. }
            | DockerErrorKind::InvalidToken { .. }
            | DockerErrorKind::TokenExpired => ErrorCategory::Auth,

            DockerErrorKind::PermissionDenied { .. } | DockerErrorKind::Forbidden { .. } => {
                ErrorCategory::Permission
            }

            DockerErrorKind::ResourceNotFound { .. } => ErrorCategory::NotFound,

            DockerErrorKind::ResourceConflict { .. } => ErrorCategory::Conflict,

            DockerErrorKind::RateLimited { .. } => ErrorCategory::RateLimited,

            DockerErrorKind::ConnectionFailed { .. }
            | DockerErrorKind::DnsResolutionFailed { .. }
            | DockerErrorKind::ServiceUnavailable { .. } => ErrorCategory::Network,

            DockerErrorKind::StorageReadFailed { .. }
            | DockerErrorKind::StorageWriteFailed { .. }
            | DockerErrorKind::StorageDeleteFailed { .. }
            | DockerErrorKind::InsufficientCapacity { .. }
            | DockerErrorKind::StorageFileNotFound { .. } => ErrorCategory::Storage,

            DockerErrorKind::ConfigMissing { .. } | DockerErrorKind::ConfigInvalid { .. } => {
                ErrorCategory::Config
            }

            DockerErrorKind::InternalError { .. }
            | DockerErrorKind::NotImplemented { .. }
            | DockerErrorKind::IoError { .. }
            | DockerErrorKind::JsonError { .. }
            | DockerErrorKind::ParseError { .. }
            | DockerErrorKind::RequestError { .. }
            | DockerErrorKind::ContainerError { .. }
            | DockerErrorKind::ImageError { .. }
            | DockerErrorKind::NetworkError { .. }
            | DockerErrorKind::RuntimeError { .. }
            | DockerErrorKind::RegistryError { .. }
            | DockerErrorKind::EtcdError { .. }
            | DockerErrorKind::MonitorError { .. }
            | DockerErrorKind::KubernetesError { .. } => ErrorCategory::Internal,
        }
    }

    /// 获取国际化键
    pub fn i18n_key(&self) -> &'static str {
        match self {
            DockerErrorKind::InvalidFormat { .. } => "docker.error.validation.invalid_format",
            DockerErrorKind::OutOfRange { .. } => "docker.error.validation.out_of_range",
            DockerErrorKind::Required { .. } => "docker.error.validation.required",
            DockerErrorKind::AlreadyExists { .. } => "docker.error.validation.already_exists",
            DockerErrorKind::NotAllowed { .. } => "docker.error.validation.not_allowed",
            DockerErrorKind::InvalidParams { .. } => "docker.error.validation.invalid_params",

            DockerErrorKind::InvalidCredentials => "docker.error.auth.invalid_credentials",
            DockerErrorKind::AccountLocked => "docker.error.auth.account_locked",
            DockerErrorKind::UserNotFound { .. } => "docker.error.auth.user_not_found",
            DockerErrorKind::UserAlreadyExists { .. } => "docker.error.auth.user_already_exists",
            DockerErrorKind::InvalidToken { .. } => "docker.error.auth.invalid_token",
            DockerErrorKind::TokenExpired => "docker.error.auth.token_expired",

            DockerErrorKind::PermissionDenied { .. } => "docker.error.permission.denied",
            DockerErrorKind::Forbidden { .. } => "docker.error.permission.forbidden",

            DockerErrorKind::ResourceNotFound { .. } => "docker.error.not_found.resource",

            DockerErrorKind::ResourceConflict { .. } => "docker.error.conflict.resource",

            DockerErrorKind::RateLimited { .. } => "docker.error.rate_limited",

            DockerErrorKind::ConnectionFailed { .. } => "docker.error.network.connection_failed",
            DockerErrorKind::DnsResolutionFailed { .. } => {
                "docker.error.network.dns_resolution_failed"
            }
            DockerErrorKind::ServiceUnavailable { .. } => {
                "docker.error.network.service_unavailable"
            }

            DockerErrorKind::StorageReadFailed { .. } => "docker.error.storage.read_failed",
            DockerErrorKind::StorageWriteFailed { .. } => "docker.error.storage.write_failed",
            DockerErrorKind::StorageDeleteFailed { .. } => "docker.error.storage.delete_failed",
            DockerErrorKind::InsufficientCapacity { .. } => {
                "docker.error.storage.insufficient_capacity"
            }
            DockerErrorKind::StorageFileNotFound { .. } => "docker.error.storage.file_not_found",

            DockerErrorKind::ConfigMissing { .. } => "docker.error.config.missing",
            DockerErrorKind::ConfigInvalid { .. } => "docker.error.config.invalid",

            DockerErrorKind::InternalError { .. } => "docker.error.internal.error",
            DockerErrorKind::NotImplemented { .. } => "docker.error.internal.not_implemented",
            DockerErrorKind::IoError { .. } => "docker.error.internal.io_error",
            DockerErrorKind::JsonError { .. } => "docker.error.internal.json_error",
            DockerErrorKind::ParseError { .. } => "docker.error.internal.parse_error",
            DockerErrorKind::RequestError { .. } => "docker.error.internal.request_error",

            DockerErrorKind::ContainerError { .. } => "docker.error.container.error",
            DockerErrorKind::ImageError { .. } => "docker.error.image.error",
            DockerErrorKind::NetworkError { .. } => "docker.error.network.error",
            DockerErrorKind::RuntimeError { .. } => "docker.error.runtime.error",
            DockerErrorKind::RegistryError { .. } => "docker.error.registry.error",
            DockerErrorKind::EtcdError { .. } => "docker.error.etcd.error",
            DockerErrorKind::MonitorError { .. } => "docker.error.monitor.error",
            DockerErrorKind::KubernetesError { .. } => "docker.error.kubernetes.error",
        }
    }

    /// 获取国际化数据
    pub fn i18n_data(&self) -> Value {
        match self {
            DockerErrorKind::InvalidFormat { field, expected } => {
                json!({ "field": field, "expected": expected })
            }
            DockerErrorKind::OutOfRange { field, min, max } => {
                json!({ "field": field, "min": min, "max": max })
            }
            DockerErrorKind::Required { field } => json!({ "field": field }),
            DockerErrorKind::AlreadyExists { field, value } => {
                json!({ "field": field, "value": value })
            }
            DockerErrorKind::NotAllowed { field, allowed } => {
                json!({ "field": field, "allowed": allowed })
            }
            DockerErrorKind::InvalidParams { param, reason } => {
                json!({ "param": param, "reason": reason })
            }

            DockerErrorKind::InvalidCredentials => json!({}),
            DockerErrorKind::AccountLocked => json!({}),
            DockerErrorKind::UserNotFound { identifier } => json!({ "identifier": identifier }),
            DockerErrorKind::UserAlreadyExists { username } => json!({ "username": username }),
            DockerErrorKind::InvalidToken { reason } => json!({ "reason": reason }),
            DockerErrorKind::TokenExpired => json!({}),

            DockerErrorKind::PermissionDenied { action } => json!({ "action": action }),
            DockerErrorKind::Forbidden { resource } => json!({ "resource": resource }),

            DockerErrorKind::ResourceNotFound {
                resource_type,
                identifier,
            } => {
                json!({ "resource_type": resource_type, "identifier": identifier })
            }

            DockerErrorKind::ResourceConflict { resource, reason } => {
                json!({ "resource": resource, "reason": reason })
            }

            DockerErrorKind::RateLimited { limit } => json!({ "limit": limit }),

            DockerErrorKind::ConnectionFailed { target } => json!({ "target": target }),
            DockerErrorKind::DnsResolutionFailed { host } => json!({ "host": host }),
            DockerErrorKind::ServiceUnavailable { service } => json!({ "service": service }),

            DockerErrorKind::StorageReadFailed { path } => json!({ "path": path }),
            DockerErrorKind::StorageWriteFailed { path } => json!({ "path": path }),
            DockerErrorKind::StorageDeleteFailed { path } => json!({ "path": path }),
            DockerErrorKind::InsufficientCapacity {
                required,
                available,
            } => {
                json!({ "required": required, "available": available })
            }
            DockerErrorKind::StorageFileNotFound { path } => json!({ "path": path }),

            DockerErrorKind::ConfigMissing { key } => json!({ "key": key }),
            DockerErrorKind::ConfigInvalid { key, reason } => {
                json!({ "key": key, "reason": reason })
            }

            DockerErrorKind::InternalError { reason } => json!({ "reason": reason }),
            DockerErrorKind::NotImplemented { feature } => json!({ "feature": feature }),
            DockerErrorKind::IoError { operation, reason } => {
                json!({ "operation": operation, "reason": reason })
            }
            DockerErrorKind::JsonError { reason } => json!({ "reason": reason }),
            DockerErrorKind::ParseError { type_name, reason } => {
                json!({ "type": type_name, "reason": reason })
            }
            DockerErrorKind::RequestError { url, reason } => {
                json!({ "url": url, "reason": reason })
            }

            DockerErrorKind::ContainerError { reason } => json!({ "reason": reason }),
            DockerErrorKind::ImageError { reason } => json!({ "reason": reason }),
            DockerErrorKind::NetworkError { reason } => json!({ "reason": reason }),
            DockerErrorKind::RuntimeError { reason } => json!({ "reason": reason }),
            DockerErrorKind::RegistryError { reason } => json!({ "reason": reason }),
            DockerErrorKind::EtcdError { reason } => json!({ "reason": reason }),
            DockerErrorKind::MonitorError { reason } => json!({ "reason": reason }),
            DockerErrorKind::KubernetesError { reason } => json!({ "reason": reason }),
        }
    }
}

impl fmt::Display for DockerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.i18n_key())
    }
}

impl DockerError {
    /// 创建新的错误
    pub fn new(kind: DockerErrorKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }

    /// 获取国际化键
    pub fn i18n_key(&self) -> &'static str {
        self.kind.i18n_key()
    }

    /// 获取国际化数据
    pub fn i18n_data(&self) -> Value {
        self.kind.i18n_data()
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        self.kind.category()
    }

    /// 获取 HTTP 状态码
    pub fn http_status(&self) -> u16 {
        self.category().http_status()
    }

    // ========== 验证错误便捷方法 ==========

    /// 创建无效格式错误
    pub fn invalid_format(field: impl Into<String>, expected: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::InvalidFormat {
            field: field.into(),
            expected: expected.into(),
        })
    }

    /// 创建超出范围错误
    pub fn out_of_range(
        field: impl Into<String>,
        min: Option<String>,
        max: Option<String>,
    ) -> Self {
        Self::new(DockerErrorKind::OutOfRange {
            field: field.into(),
            min,
            max,
        })
    }

    /// 创建必填字段缺失错误
    pub fn required(field: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::Required {
            field: field.into(),
        })
    }

    /// 创建已存在错误
    pub fn already_exists(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::AlreadyExists {
            field: field.into(),
            value: value.into(),
        })
    }

    /// 创建不允许错误
    pub fn not_allowed(field: impl Into<String>, allowed: Vec<String>) -> Self {
        Self::new(DockerErrorKind::NotAllowed {
            field: field.into(),
            allowed,
        })
    }

    /// 创建无效参数错误
    pub fn invalid_params(param: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::InvalidParams {
            param: param.into(),
            reason: reason.into(),
        })
    }

    // ========== 认证错误便捷方法 ==========

    /// 创建无效凭证错误
    pub fn invalid_credentials() -> Self {
        Self::new(DockerErrorKind::InvalidCredentials)
    }

    /// 创建账户已锁定错误
    pub fn account_locked() -> Self {
        Self::new(DockerErrorKind::AccountLocked)
    }

    /// 创建用户未找到错误
    pub fn user_not_found(identifier: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::UserNotFound {
            identifier: identifier.into(),
        })
    }

    /// 创建用户已存在错误
    pub fn user_already_exists(username: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::UserAlreadyExists {
            username: username.into(),
        })
    }

    /// 创建无效令牌错误
    pub fn invalid_token(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::InvalidToken {
            reason: reason.into(),
        })
    }

    /// 创建令牌过期错误
    pub fn token_expired() -> Self {
        Self::new(DockerErrorKind::TokenExpired)
    }

    // ========== 权限错误便捷方法 ==========

    /// 创建权限拒绝错误
    pub fn permission_denied(action: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::PermissionDenied {
            action: action.into(),
        })
    }

    /// 创建禁止访问错误
    pub fn forbidden(resource: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::Forbidden {
            resource: resource.into(),
        })
    }

    // ========== 资源未找到便捷方法 ==========

    /// 创建资源未找到错误
    pub fn not_found(resource_type: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::ResourceNotFound {
            resource_type: resource_type.into(),
            identifier: identifier.into(),
        })
    }

    // ========== 网络错误便捷方法 ==========

    /// 创建连接失败错误
    pub fn connection_failed(target: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::ConnectionFailed {
            target: target.into(),
        })
    }

    /// 创建服务不可用错误
    pub fn service_unavailable(service: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::ServiceUnavailable {
            service: service.into(),
        })
    }

    // ========== 存储错误便捷方法 ==========

    /// 创建存储读取失败错误
    pub fn storage_read_failed(path: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::StorageReadFailed { path: path.into() })
    }

    /// 创建存储写入失败错误
    pub fn storage_write_failed(path: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::StorageWriteFailed { path: path.into() })
    }

    /// 创建存储文件未找到错误
    pub fn storage_file_not_found(path: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::StorageFileNotFound { path: path.into() })
    }

    // ========== 配置错误便捷方法 ==========

    /// 创建配置缺失错误
    pub fn config_missing(key: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::ConfigMissing { key: key.into() })
    }

    /// 创建配置无效错误
    pub fn config_invalid(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::ConfigInvalid {
            key: key.into(),
            reason: reason.into(),
        })
    }

    // ========== 内部错误便捷方法 ==========

    /// 创建内部错误
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::InternalError {
            reason: reason.into(),
        })
    }

    /// 创建未实现错误
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::NotImplemented {
            feature: feature.into(),
        })
    }

    /// 创建 IO 错误
    pub fn io_error(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::IoError {
            operation: operation.into(),
            reason: reason.into(),
        })
    }

    /// 创建 JSON 错误
    pub fn json_error(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::JsonError {
            reason: reason.into(),
        })
    }

    /// 创建解析错误
    pub fn parse_error(type_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::ParseError {
            type_name: type_name.into(),
            reason: reason.into(),
        })
    }

    /// 创建请求错误
    pub fn request_error(url: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::RequestError {
            url: url.into(),
            reason: reason.into(),
        })
    }

    // ========== Docker 特定错误便捷方法 ==========

    /// 创建容器错误
    pub fn container_error(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::ContainerError {
            reason: reason.into(),
        })
    }

    /// 创建镜像错误
    pub fn image_error(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::ImageError {
            reason: reason.into(),
        })
    }

    /// 创建网络错误
    pub fn network_error(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::NetworkError {
            reason: reason.into(),
        })
    }

    /// 创建运行时错误
    pub fn runtime_error(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::RuntimeError {
            reason: reason.into(),
        })
    }

    /// 创建注册表错误
    pub fn registry_error(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::RegistryError {
            reason: reason.into(),
        })
    }

    /// 创建 Etcd 错误
    pub fn etcd_error(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::EtcdError {
            reason: reason.into(),
        })
    }

    /// 创建监控错误
    pub fn monitor_error(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::MonitorError {
            reason: reason.into(),
        })
    }

    /// 创建 Kubernetes 错误
    pub fn kubernetes_error(reason: impl Into<String>) -> Self {
        Self::new(DockerErrorKind::KubernetesError {
            reason: reason.into(),
        })
    }
}

impl fmt::Display for DockerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind.category(), self.kind.i18n_key())
    }
}

impl std::error::Error for DockerError {}

impl From<std::io::Error> for DockerError {
    fn from(err: std::io::Error) -> Self {
        Self::io_error("unknown", err.to_string())
    }
}

impl From<serde_json::Error> for DockerError {
    fn from(err: serde_json::Error) -> Self {
        Self::json_error(err.to_string())
    }
}

/// 向后兼容：RustyDockerError 类型别名
#[deprecated(note = "请使用 DockerError 替代")]
pub type RustyDockerError = DockerError;
