use super::{NetworkConfig, NetworkInfo, NetworkManager, Result};
use bollard::Docker;
use bollard::network::{
    ConnectNetworkOptions, DisconnectNetworkOptions, InspectNetworkOptions, ListNetworksOptions,
    NetworkCreateRequest,
};
use docker_types::DockerError;

pub struct MacOSNetworkManager {
    docker: Docker,
}

impl MacOSNetworkManager {
    pub fn new() -> Self {
        let docker = Docker::connect_with_defaults().expect("Failed to connect to Docker");
        Self { docker }
    }
}

impl NetworkManager for MacOSNetworkManager {
    fn create_network(
        &mut self,
        config: &NetworkConfig,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<NetworkInfo>> + Send + '_>> {
        Box::pin(async move {
            let network_create = NetworkCreateRequest {
                name: config.name.clone(),
                driver: Some(config.driver.clone()),
                enable_ipv6: Some(config.enable_ipv6),
                options: config.options.clone(),
                ..Default::default()
            };

            let response = self
                .docker
                .create_network(network_create)
                .await
                .map_err(|e| {
                    DockerError::container_error(format!("Failed to create network: {:?}", e))
                })?;

            let network_id = response.id;
            self.inspect_network(&network_id).await
        })
    }

    fn connect_container(
        &mut self,
        network_id: &str,
        container_id: &str,
        aliases: Option<Vec<String>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let connect_options = ConnectNetworkOptions {
                container: container_id.to_string(),
                aliases,
                ..Default::default()
            };

            self.docker
                .connect_network(network_id, connect_options)
                .await
                .map_err(|e| {
                    DockerError::container_error(format!("Failed to connect container: {:?}", e))
                })?;

            Ok(())
        })
    }

    fn disconnect_container(
        &mut self,
        network_id: &str,
        container_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let disconnect_options = DisconnectNetworkOptions {
                container: container_id.to_string(),
                force: Some(true),
                ..Default::default()
            };

            self.docker
                .disconnect_network(network_id, disconnect_options)
                .await
                .map_err(|e| {
                    DockerError::container_error(format!("Failed to disconnect container: {:?}", e))
                })?;

            Ok(())
        })
    }

    fn remove_network(
        &mut self,
        network_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.docker.remove_network(network_id).await.map_err(|e| {
                DockerError::container_error(format!("Failed to remove network: {:?}", e))
            })?;

            Ok(())
        })
    }

    fn list_networks(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<NetworkInfo>>> + Send + '_>>
    {
        Box::pin(async move {
            let options = ListNetworksOptions::default();
            let networks = self
                .docker
                .list_networks(Some(options))
                .await
                .map_err(|e| {
                    DockerError::container_error(format!("Failed to list networks: {:?}", e))
                })?;

            let network_infos: Vec<NetworkInfo> = networks
                .into_iter()
                .map(|network| NetworkInfo {
                    id: network.id.unwrap_or_default(),
                    name: network.name.unwrap_or_default(),
                    driver: network.driver.unwrap_or_default(),
                    scope: network.scope.unwrap_or_default(),
                    enable_ipv6: network.enable_ipv6.unwrap_or(false),
                    internal: network.internal.unwrap_or(false),
                    attachable: network.attachable.unwrap_or(false),
                    ingress: network.ingress.unwrap_or(false),
                    containers: std::collections::HashMap::new(),
                    options: network.options.unwrap_or_default(),
                    labels: network.labels.unwrap_or_default(),
                })
                .collect();

            Ok(network_infos)
        })
    }

    fn inspect_network(
        &mut self,
        network_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<NetworkInfo>> + Send + '_>> {
        Box::pin(async move {
            let options = InspectNetworkOptions {
                scope: None,
                verbose: None,
            };
            let network = self
                .docker
                .inspect_network(network_id, Some(options))
                .await
                .map_err(|e| {
                    DockerError::container_error(format!("Failed to inspect network: {:?}", e))
                })?;

            let containers = std::collections::HashMap::new();

            Ok(NetworkInfo {
                id: network.id.unwrap_or_default(),
                name: network.name.unwrap_or_default(),
                driver: network.driver.unwrap_or_default(),
                scope: network.scope.unwrap_or_default(),
                enable_ipv6: network.enable_ipv6.unwrap_or(false),
                internal: network.internal.unwrap_or(false),
                attachable: network.attachable.unwrap_or(false),
                ingress: network.ingress.unwrap_or(false),
                containers,
                options: network.options.unwrap_or_default(),
                labels: network.labels.unwrap_or_default(),
            })
        })
    }
}
