#![warn(missing_docs)]

//! rusty-kubernetes - Rust 实现的轻量级 Kubernetes 工具集
//!
//! 实现基于 Rust 的 Kubernetes 工具集，提供：
//! - 轻量级 Kubernetes 组件
//! - 与 Docker 容器运行时集成
//! - 简化的集群管理
//! - 与 MOS 平台集成

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod runtime;

use std::sync::{Arc, Mutex};

use docker_types::Result as DockerResult;
use runtime::KubernetesRuntime;

/// Kubernetes 服务
pub struct RustyKubernetes {
    /// Kubernetes 运行时
    runtime: Arc<KubernetesRuntime>,
}

/// Kubernetes 服务别名
pub type Kubernetes = RustyKubernetes;

impl RustyKubernetes {
    /// 创建新的 Rusty Kubernetes 服务
    pub fn new() -> DockerResult<Self> {
        let runtime = Arc::new(KubernetesRuntime::new()?);

        Ok(Self { runtime })
    }

    /// 启动服务
    pub async fn start(&self) {
        // 直接启动运行时
        self.runtime.start().await;
    }

    /// 获取 Kubernetes 运行时
    pub fn get_runtime(&self) -> Arc<KubernetesRuntime> {
        self.runtime.clone()
    }

    /// 部署应用
    pub async fn deploy(
        &mut self,
        name: String,
        image: String,
        replicas: u32,
        ports: Vec<String>,
        env: Vec<String>,
    ) -> DockerResult<models::DeploymentInfo> {
        self.runtime.deploy(name, image, replicas, ports, env).await
    }

    /// 列出部署
    pub async fn list_deployments(&self) -> DockerResult<Vec<models::DeploymentInfo>> {
        self.runtime.list_deployments().await
    }

    /// 删除部署
    pub async fn delete_deployment(&mut self, name: &str) -> DockerResult<()> {
        self.runtime.delete_deployment(name).await
    }

    /// 扩缩容部署
    pub async fn scale_deployment(&mut self, name: &str, replicas: u32) -> DockerResult<models::DeploymentInfo> {
        self.runtime.scale_deployment(name, replicas).await
    }

    /// 创建服务
    pub async fn create_service(
        &mut self,
        name: String,
        selector: std::collections::HashMap<String, String>,
        ports: Vec<models::ServicePort>,
        service_type: models::ServiceType,
    ) -> DockerResult<models::ServiceInfo> {
        self.runtime.create_service(name, selector, ports, service_type).await
    }

    /// 列出服务
    pub async fn list_services(&self) -> DockerResult<Vec<models::ServiceInfo>> {
        self.runtime.list_services().await
    }

    /// 删除服务
    pub async fn delete_service(&mut self, name: &str) -> DockerResult<()> {
        self.runtime.delete_service(name).await
    }

    /// 创建配置映射
    pub async fn create_config_map(
        &mut self,
        name: String,
        data: std::collections::HashMap<String, String>,
    ) -> DockerResult<models::ConfigMapInfo> {
        self.runtime.create_config_map(name, data).await
    }

    /// 列出配置映射
    pub async fn list_config_maps(&self) -> DockerResult<Vec<models::ConfigMapInfo>> {
        self.runtime.list_config_maps().await
    }

    /// 删除配置映射
    pub async fn delete_config_map(&mut self, name: &str) -> DockerResult<()> {
        self.runtime.delete_config_map(name).await
    }

    /// 创建秘密
    pub async fn create_secret(
        &mut self,
        name: String,
        data: std::collections::HashMap<String, String>,
    ) -> DockerResult<models::SecretInfo> {
        self.runtime.create_secret(name, data).await
    }

    /// 列出秘密
    pub async fn list_secrets(&self) -> DockerResult<Vec<models::SecretInfo>> {
        self.runtime.list_secrets().await
    }

    /// 删除秘密
    pub async fn delete_secret(&mut self, name: &str) -> DockerResult<()> {
        self.runtime.delete_secret(name).await
    }

    /// 获取集群信息
    pub async fn get_cluster_info(&self) -> DockerResult<models::ClusterInfo> {
        self.runtime.get_cluster_info().await
    }

    /// 列出节点
    pub async fn list_nodes(&self) -> DockerResult<Vec<models::NodeInfo>> {
        self.runtime.list_nodes().await
    }

    /// 获取节点详情
    pub async fn get_node(&self, name: &str) -> DockerResult<models::NodeInfo> {
        self.runtime.get_node(name).await
    }

    /// 克隆 Kubernetes 实例
    pub fn clone(&self) -> Self {
        Self { runtime: self.runtime.clone() }
    }
}
