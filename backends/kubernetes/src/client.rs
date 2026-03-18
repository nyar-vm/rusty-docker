#![warn(missing_docs)]

//! Kubernetes 客户端实现

use crate::{api::KubernetesApi, runtime::KubernetesRuntime};
use docker_types::Result as DockerResult;

/// Kubernetes 客户端
pub struct KubernetesClient {
    /// API 服务
    api: KubernetesApi,
}

impl KubernetesClient {
    /// 创建新的 Kubernetes 客户端
    pub fn new() -> DockerResult<Self> {
        let runtime = std::sync::Arc::new(KubernetesRuntime::new()?);
        let api = KubernetesApi::new(runtime);

        Ok(Self { api })
    }

    /// 获取 API 服务
    pub fn api(&self) -> &KubernetesApi {
        &self.api
    }

    /// 部署应用
    pub async fn deploy(
        &self,
        name: String,
        image: String,
        replicas: u32,
        ports: Vec<String>,
        env: Vec<String>,
    ) -> DockerResult<crate::models::DeploymentInfo> {
        self.api.deploy(name, image, replicas, ports, env).await
    }

    /// 列出部署
    pub async fn list_deployments(&self) -> DockerResult<Vec<crate::models::DeploymentInfo>> {
        self.api.list_deployments().await
    }

    /// 删除部署
    pub async fn delete_deployment(&self, name: &str) -> DockerResult<()> {
        self.api.delete_deployment(name).await
    }

    /// 扩缩容部署
    pub async fn scale_deployment(&self, name: &str, replicas: u32) -> DockerResult<crate::models::DeploymentInfo> {
        self.api.scale_deployment(name, replicas).await
    }

    /// 创建服务
    pub async fn create_service(
        &self,
        name: String,
        selector: std::collections::HashMap<String, String>,
        ports: Vec<crate::models::ServicePort>,
        service_type: crate::models::ServiceType,
    ) -> DockerResult<crate::models::ServiceInfo> {
        self.api.create_service(name, selector, ports, service_type).await
    }

    /// 列出服务
    pub async fn list_services(&self) -> DockerResult<Vec<crate::models::ServiceInfo>> {
        self.api.list_services().await
    }

    /// 删除服务
    pub async fn delete_service(&self, name: &str) -> DockerResult<()> {
        self.api.delete_service(name).await
    }

    /// 创建配置映射
    pub async fn create_config_map(
        &self,
        name: String,
        data: std::collections::HashMap<String, String>,
    ) -> DockerResult<crate::models::ConfigMapInfo> {
        self.api.create_config_map(name, data).await
    }

    /// 列出配置映射
    pub async fn list_config_maps(&self) -> DockerResult<Vec<crate::models::ConfigMapInfo>> {
        self.api.list_config_maps().await
    }

    /// 删除配置映射
    pub async fn delete_config_map(&self, name: &str) -> DockerResult<()> {
        self.api.delete_config_map(name).await
    }

    /// 创建秘密
    pub async fn create_secret(
        &self,
        name: String,
        data: std::collections::HashMap<String, String>,
    ) -> DockerResult<crate::models::SecretInfo> {
        self.api.create_secret(name, data).await
    }

    /// 列出秘密
    pub async fn list_secrets(&self) -> DockerResult<Vec<crate::models::SecretInfo>> {
        self.api.list_secrets().await
    }

    /// 删除秘密
    pub async fn delete_secret(&self, name: &str) -> DockerResult<()> {
        self.api.delete_secret(name).await
    }

    /// 获取集群信息
    pub async fn get_cluster_info(&self) -> DockerResult<crate::models::ClusterInfo> {
        self.api.get_cluster_info().await
    }

    /// 列出节点
    pub async fn list_nodes(&self) -> DockerResult<Vec<crate::models::NodeInfo>> {
        self.api.list_nodes().await
    }

    /// 获取节点详情
    pub async fn get_node(&self, name: &str) -> DockerResult<crate::models::NodeInfo> {
        self.api.get_node(name).await
    }
}
