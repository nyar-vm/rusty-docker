#![warn(missing_docs)]

//! 存储模块
//! 
//! 提供容器存储管理功能，包括：
//! - Overlay2 文件系统支持
//! - 容器文件系统管理
//! - 镜像存储管理

pub mod overlay2;
pub mod service;

pub use overlay2::Overlay2Driver;
pub use service::StorageService;
