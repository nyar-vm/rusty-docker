#![warn(missing_docs)]

//! Kubernetes API 接口

use crate::{models::*, runtime::KubernetesRuntime};
use docker_types::Result as DockerResult;

/// Kubernetes API 服务
pub struct KubernetesApi {
    /// Kubernetes 运行时
    runtime: std::sync::Arc<KubernetesRuntime>,
}

impl KubernetesApi {
    /// 创建新的 Kubernetes API 服务
    pub fn new(runtime: std::sync::Arc<KubernetesRuntime>) -> Self {
        Self { runtime }
    }

    /// 部署应用
    pub async fn deploy(
        &self,
        name: String,
        image: String,
        replicas: u32,
        ports: Vec<String>,
        env: Vec<String>,
    ) -> DockerResult<DeploymentInfo> {
        self.runtime.deploy(name, image, replicas, ports, env).await
    }

    /// 列出部署
    pub async fn list_deployments(&self) -> DockerResult<Vec<DeploymentInfo>> {
        self.runtime.list_deployments().await
    }

    /// 删除部署
    pub async fn delete_deployment(&self, name: &str) -> DockerResult<()> {
        self.runtime.delete_deployment(name).await
    }

    /// 扩缩容部署
    pub async fn scale_deployment(&self, name: &str, replicas: u32) -> DockerResult<DeploymentInfo> {
        self.runtime.scale_deployment(name, replicas).await
    }

    /// 创建服务
    pub async fn create_service(
        &self,
        name: String,
        selector: std::collections::HashMap<String, String>,
        ports: Vec<ServicePort>,
        service_type: ServiceType,
    ) -> DockerResult<ServiceInfo> {
        self.runtime.create_service(name, selector, ports, service_type).await
    }

    /// 列出服务
    pub async fn list_services(&self) -> DockerResult<Vec<ServiceInfo>> {
        self.runtime.list_services().await
    }

    /// 删除服务
    pub async fn delete_service(&self, name: &str) -> DockerResult<()> {
        self.runtime.delete_service(name).await
    }

    /// 创建配置映射
    pub async fn create_config_map(
        &self,
        name: String,
        data: std::collections::HashMap<String, String>,
    ) -> DockerResult<ConfigMapInfo> {
        self.runtime.create_config_map(name, data).await
    }

    /// 列出配置映射
    pub async fn list_config_maps(&self) -> DockerResult<Vec<ConfigMapInfo>> {
        self.runtime.list_config_maps().await
    }

    /// 删除配置映射
    pub async fn delete_config_map(&self, name: &str) -> DockerResult<()> {
        self.runtime.delete_config_map(name).await
    }

    /// 创建秘密
    pub async fn create_secret(
        &self,
        name: String,
        data: std::collections::HashMap<String, String>,
    ) -> DockerResult<SecretInfo> {
        self.runtime.create_secret(name, data).await
    }

    /// 列出秘密
    pub async fn list_secrets(&self) -> DockerResult<Vec<SecretInfo>> {
        self.runtime.list_secrets().await
    }

    /// 删除秘密
    pub async fn delete_secret(&self, name: &str) -> DockerResult<()> {
        self.runtime.delete_secret(name).await
    }

    /// 获取集群信息
    pub async fn get_cluster_info(&self) -> DockerResult<ClusterInfo> {
        self.runtime.get_cluster_info().await
    }

    /// 列出节点
    pub async fn list_nodes(&self) -> DockerResult<Vec<NodeInfo>> {
        self.runtime.list_nodes().await
    }

    /// 获取节点详情
    pub async fn get_node(&self, name: &str) -> DockerResult<NodeInfo> {
        self.runtime.get_node(name).await
    }
}
