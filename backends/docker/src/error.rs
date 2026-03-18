//! 错误处理模块

use docker_types::DockerError;

/// 错误类型重导出
pub type Error = DockerError;

/// 结果类型别名
pub type Result<T> = std::result::Result<T, Error>;
