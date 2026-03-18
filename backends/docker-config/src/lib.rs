#![warn(missing_docs)]

//! Docker 核心运行时
//!
//! 包含容器运行时、命名空间管理、控制组管理等核心功能。

pub mod config;
pub mod runtime;

use std::collections::HashMap;
use std::sync::Arc;

use config::ConfigManager;
use docker_types::{ConfigInfo, DockerConfig, SecretInfo};
use runtime::ContainerRuntime;

/// Rusty Docker 服务
pub struct RustyDocker {
    /// 容器运行时
    runtime: Arc<ContainerRuntime>,
    /// 配置管理器
    config_manager: Arc<ConfigManager>,
    /// 全局配置
    config: Arc<DockerConfig>,
}

/// Docker 服务别名
pub type Docker = RustyDocker;

impl RustyDocker {
    /// 创建新的 Rusty Docker 服务
    pub fn new() -> docker_types::Result<Self> {
        // 初始化配置管理器
        let config_manager = Arc::new(ConfigManager::new()?);
        let config = Arc::new(config_manager.get_config()?);

        // 使用配置初始化容器运行时
        let runtime = Arc::new(ContainerRuntime::new_with_config(config.clone())?);

        Ok(Self {
            runtime,
            config_manager,
            config,
        })
    }

    /// 获取全局配置
    pub fn get_config(&self) -> Arc<DockerConfig> {
        self.config.clone()
    }

    /// 重新加载配置
    pub fn reload_config(&mut self) -> docker_types::Result<()> {
        let new_config = self.config_manager.get_config()?;
        self.config = Arc::new(new_config);
        Ok(())
    }

    /// 启动服务
    pub async fn start(&self) {
        self.runtime.start().await;
    }

    /// 获取容器运行时
    pub fn get_runtime(&self) -> Arc<ContainerRuntime> {
        self.runtime.clone()
    }

    /// 运行容器
    pub async fn run(
        &mut self,
        image: String,
        name: Option<String>,
        ports: Vec<String>,
    ) -> docker_types::Result<docker_types::ContainerInfo> {
        self.runtime.run_container(image, name, ports).await
    }

    /// 列出容器
    pub async fn list_containers(
        &self,
        all: bool,
    ) -> docker_types::Result<Vec<docker_types::ContainerInfo>> {
        self.runtime.list_containers(all).await
    }

    /// 停止容器
    pub async fn stop_container(&mut self, container_id: &str) -> docker_types::Result<()> {
        self.runtime.stop_container(container_id).await
    }

    /// 删除容器
    pub async fn remove_container(&mut self, container_id: &str) -> docker_types::Result<()> {
        self.runtime.remove_container(container_id).await
    }

    /// 克隆 Docker 实例
    pub fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            config_manager: self.config_manager.clone(),
            config: self.config.clone(),
        }
    }

    /// 创建配置
    pub fn create_config(
        &self,
        name: &str,
        data: &str,
        labels: HashMap<String, String>,
    ) -> docker_types::Result<ConfigInfo> {
        self.config_manager.create_config(name, data, labels)
    }

    /// 更新配置
    pub fn update_config(
        &self,
        config_id: &str,
        data: &str,
        labels: HashMap<String, String>,
    ) -> docker_types::Result<ConfigInfo> {
        self.config_manager.update_config(config_id, data, labels)
    }

    /// 删除配置
    pub fn delete_config(&self, config_id: &str) -> docker_types::Result<()> {
        self.config_manager.delete_config(config_id)
    }

    /// 获取配置详细信息
    pub fn get_config_info(&self, config_id: &str) -> docker_types::Result<ConfigInfo> {
        self.config_manager.get_config_info(config_id)
    }

    /// 列出所有配置
    pub fn list_configs(&self) -> docker_types::Result<Vec<ConfigInfo>> {
        self.config_manager.list_configs()
    }

    /// 创建密钥
    pub fn create_secret(
        &self,
        name: &str,
        data: &str,
        labels: HashMap<String, String>,
    ) -> docker_types::Result<SecretInfo> {
        self.config_manager.create_secret(name, data, labels)
    }

    /// 删除密钥
    pub fn delete_secret(&self, secret_id: &str) -> docker_types::Result<()> {
        self.config_manager.delete_secret(secret_id)
    }

    /// 获取密钥详细信息
    pub fn get_secret_info(&self, secret_id: &str) -> docker_types::Result<SecretInfo> {
        self.config_manager.get_secret_info(secret_id)
    }

    /// 列出所有密钥
    pub fn list_secrets(&self) -> docker_types::Result<Vec<SecretInfo>> {
        self.config_manager.list_secrets()
    }
}
