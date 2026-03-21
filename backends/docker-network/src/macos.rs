use super::{NetworkConfig, NetworkInfo, NetworkDriver, NetworkManager, Result, BridgeNetworkDriver};
use docker_types::DockerError;
use rand;
use std::collections::HashMap;
use tokio::sync::{Arc, Mutex};

pub struct MacOSNetworkManager {
    networks: Arc<Mutex<HashMap<String, NetworkInfo>>>,
    drivers: Arc<Mutex<HashMap<String, Arc<Mutex<Box<dyn NetworkDriver>>>>>>,
}

impl MacOSNetworkManager {
    pub fn new() -> Self {
        let manager = Self {
            networks: Arc::new(Mutex::new(HashMap::new())),
            drivers: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Register default bridge driver
        manager.register_driver("bridge", Box::new(BridgeNetworkDriver::new())).unwrap();
        
        manager
    }
}

impl NetworkManager for MacOSNetworkManager {
    async fn create_network(
        &self,
        config: &NetworkConfig,
    ) -> Result<NetworkInfo> {
        let config = config.clone();
        let driver_name = config.driver.clone();
        let drivers = self.drivers.clone();
        let networks = self.networks.clone();
        
        // Get driver
        let driver = {
            let mut drivers_guard = drivers.lock().await;
            drivers_guard.get(&driver_name).ok_or_else(|| {
                DockerError::not_found("network driver", driver_name.clone())
            })?
            .clone()
        };
        
        // Create network using driver
        let network_info = {
            let mut driver_guard = driver.lock().await;
            driver_guard.create_network(&config).await
        }?;
        
        // Store network info
        let mut networks_guard = networks.lock().await;
        networks_guard.insert(network_info.id.clone(), network_info.clone());
        
        Ok(network_info)
    }

    async fn connect_container(
        &self,
        network_id: &str,
        container_id: &str,
        aliases: Option<Vec<String>>,
    ) -> Result<()> {
        let network_id = network_id.to_string();
        let container_id = container_id.to_string();
        let aliases = aliases;
        let networks = self.networks.clone();
        let drivers = self.drivers.clone();
        
        // Get driver name
        let driver_name = {
            let mut networks_guard = networks.lock().await;
            let network = networks_guard.get(&network_id).ok_or_else(|| {
                DockerError::not_found("network", network_id.clone())
            })?;
            network.driver.clone()
        };
        
        // Get driver
        let driver = {
            let mut drivers_guard = drivers.lock().await;
            drivers_guard.get(&driver_name).ok_or_else(|| {
                DockerError::not_found("network driver", driver_name)
            })?
            .clone()
        };
        
        // Connect container using driver
        let mut driver_guard = driver.lock().await;
        driver_guard.connect_container(&network_id, &container_id, aliases).await
    }

    async fn disconnect_container(
        &self,
        network_id: &str,
        container_id: &str,
    ) -> Result<()> {
        let network_id = network_id.to_string();
        let container_id = container_id.to_string();
        let networks = self.networks.clone();
        let drivers = self.drivers.clone();
        
        // Get driver name
        let driver_name = {
            let mut networks_guard = networks.lock().await;
            let network = networks_guard.get(&network_id).ok_or_else(|| {
                DockerError::not_found("network", network_id.clone())
            })?;
            network.driver.clone()
        };
        
        // Get driver
        let driver = {
            let mut drivers_guard = drivers.lock().await;
            drivers_guard.get(&driver_name).ok_or_else(|| {
                DockerError::not_found("network driver", driver_name)
            })?
            .clone()
        };
        
        // Disconnect container using driver
        let mut driver_guard = driver.lock().await;
        driver_guard.disconnect_container(&network_id, &container_id).await
    }

    async fn add_port_mapping(
        &self,
        container_id: &str,
        container_ip: &str,
        port_mapping: &super::PortMapping,
    ) -> Result<()> {
        // 默认使用 bridge 驱动
        let driver_name = "bridge";
        let drivers = self.drivers.clone();
        
        // Get driver
        let driver = {
            let mut drivers_guard = drivers.lock().await;
            drivers_guard.get(driver_name).ok_or_else(|| {
                DockerError::not_found("network driver", driver_name.to_string())
            })?
            .clone()
        };
        
        // Add port mapping using driver
        let mut driver_guard = driver.lock().await;
        driver_guard.add_port_mapping(container_id, container_ip, port_mapping).await
    }

    async fn remove_port_mapping(
        &self,
        port_mapping: &super::PortMapping,
    ) -> Result<()> {
        // 默认使用 bridge 驱动
        let driver_name = "bridge";
        let drivers = self.drivers.clone();
        
        // Get driver
        let driver = {
            let mut drivers_guard = drivers.lock().await;
            drivers_guard.get(driver_name).ok_or_else(|| {
                DockerError::not_found("network driver", driver_name.to_string())
            })?
            .clone()
        };
        
        // Remove port mapping using driver
        let mut driver_guard = driver.lock().await;
        driver_guard.remove_port_mapping(port_mapping).await
    }

    async fn remove_network(
        &self,
        network_id: &str,
    ) -> Result<()> {
        let network_id = network_id.to_string();
        let networks = self.networks.clone();
        let drivers = self.drivers.clone();
        
        // Get driver name
        let driver_name = {
            let mut networks_guard = networks.lock().await;
            let network = networks_guard.get(&network_id).ok_or_else(|| {
                DockerError::not_found("network", network_id.clone())
            })?;
            network.driver.clone()
        };
        
        // Get driver
        let driver = {
            let mut drivers_guard = drivers.lock().await;
            drivers_guard.get(&driver_name).ok_or_else(|| {
                DockerError::not_found("network driver", driver_name)
            })?
            .clone()
        };
        
        // Remove network using driver
        let mut driver_guard = driver.lock().await;
        driver_guard.remove_network(&network_id).await?;
        
        // Remove from networks map
        let mut networks_guard = networks.lock().await;
        networks_guard.remove(&network_id);
        
        Ok(())
    }

    async fn list_networks(&self) -> Result<Vec<NetworkInfo>> {
        let networks = self.networks.clone();
        let networks_guard = networks.lock().await;
        Ok(networks_guard.values().cloned().collect())
    }

    async fn inspect_network(
        &self,
        network_id: &str,
    ) -> Result<NetworkInfo> {
        let network_id = network_id.to_string();
        let networks = self.networks.clone();
        let drivers = self.drivers.clone();
        
        // Get driver name
        let driver_name = {
            let mut networks_guard = networks.lock().await;
            let network = networks_guard.get(&network_id).ok_or_else(|| {
                DockerError::not_found("network", network_id.clone())
            })?;
            network.driver.clone()
        };
        
        // Get driver
        let driver = {
            let mut drivers_guard = drivers.lock().await;
            drivers_guard.get(&driver_name).ok_or_else(|| {
                DockerError::not_found("network driver", driver_name)
            })?
            .clone()
        };
        
        // Inspect network using driver
        let mut driver_guard = driver.lock().await;
        driver_guard.inspect_network(&network_id).await
    }

    fn register_driver(
        &self,
        name: &str,
        driver: Box<dyn NetworkDriver>,
    ) -> Result<()> {
        tokio::runtime::Handle::current().block_on(async {
            let mut drivers = self.drivers.lock().await;
            drivers.insert(name.to_string(), Arc::new(Mutex::new(driver)));
            Ok(())
        })
    }

    fn get_driver(&self, name: &str) -> Option<Arc<Mutex<Box<dyn NetworkDriver>>>> {
        tokio::runtime::Handle::current().block_on(async {
            let drivers = self.drivers.lock().await;
            drivers.get(name).cloned()
        })
    }
}
