use super::{NetworkConfig, Result};
use docker_types::DockerError;
use std::fs;
use std::path::Path;

/// NetworkConfigManager 用于管理网络配置
#[derive(Debug)]
pub struct NetworkConfigManager {
    config_path: String,
}

impl NetworkConfigManager {
    /// 创建新的网络配置管理器
    ///
    /// # 参数
    /// * `config_path` - 配置文件路径
    ///
    /// # 返回值
    /// * `NetworkConfigManager` - 网络配置管理器实例
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
        }
    }

    /// 加载网络配置
    ///
    /// # 参数
    /// * `name` - 网络名称
    ///
    /// # 返回值
    /// * `Ok(NetworkConfig)` - 加载成功的网络配置
    /// * `Err(DockerError)` - 加载失败的错误信息
    pub fn load_config(&self, name: &str) -> Result<NetworkConfig> {
        let config_file = Path::new(&self.config_path).join(format!("{}.json", name));
        
        if !config_file.exists() {
            return Err(DockerError::not_found("network config", name.to_string()));
        }
        
        let content = fs::read_to_string(config_file).map_err(|e| {
            DockerError::internal(format!("Failed to read config file: {}", e))
        })?;
        
        let config: NetworkConfig = serde_json::from_str(&content).map_err(|e| {
            DockerError::internal(format!("Failed to parse config file: {}", e))
        })?;
        
        Ok(config)
    }

    /// 保存网络配置
    ///
    /// # 参数
    /// * `config` - 网络配置
    ///
    /// # 返回值
    /// * `Ok(())` - 保存成功
    /// * `Err(DockerError)` - 保存失败的错误信息
    pub fn save_config(&self, config: &NetworkConfig) -> Result<()> {
        let config_file = Path::new(&self.config_path).join(format!("{}.json", config.name));
        
        // Create directory if it doesn't exist
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                DockerError::internal(format!("Failed to create config directory: {}", e))
            })?;
        }
        
        let content = serde_json::to_string_pretty(config).map_err(|e| {
            DockerError::internal(format!("Failed to serialize config: {}", e))
        })?;
        
        fs::write(config_file, content).map_err(|e| {
            DockerError::internal(format!("Failed to write config file: {}", e))
        })?;
        
        Ok(())
    }

    /// 删除网络配置
    ///
    /// # 参数
    /// * `name` - 网络名称
    ///
    /// # 返回值
    /// * `Ok(())` - 删除成功
    /// * `Err(DockerError)` - 删除失败的错误信息
    pub fn delete_config(&self, name: &str) -> Result<()> {
        let config_file = Path::new(&self.config_path).join(format!("{}.json", name));
        
        if !config_file.exists() {
            return Err(DockerError::not_found("network config", name.to_string()));
        }
        
        fs::remove_file(config_file).map_err(|e| {
            DockerError::internal(format!("Failed to delete config file: {}", e))
        })?;
        
        Ok(())
    }

    /// 验证网络配置
    ///
    /// # 参数
    /// * `config` - 网络配置
    ///
    /// # 返回值
    /// * `Ok(())` - 验证成功
    /// * `Err(DockerError)` - 验证失败的错误信息
    pub fn validate_config(&self, config: &NetworkConfig) -> Result<()> {
        // Validate network name
        if config.name.is_empty() {
            return Err(DockerError::invalid_params("name", "Network name cannot be empty"));
        }
        
        // Validate driver name
        if config.driver.is_empty() {
            return Err(DockerError::invalid_params("driver", "Network driver cannot be empty"));
        }
        
        // Validate IPAM config if provided
        if let Some(ipam) = &config.ipam {
            if ipam.driver.is_empty() {
                return Err(DockerError::invalid_params(
                    "ipam.driver", 
                    "IPAM driver cannot be empty"
                ));
            }
            
            for (i, subnet_config) in ipam.config.iter().enumerate() {
                if subnet_config.subnet.is_empty() {
                    return Err(DockerError::invalid_params(
                        &format!("ipam.config[{}].subnet", i),
                        "Subnet CIDR cannot be empty"
                    ));
                }
            }
        }
        
        Ok(())
    }
}
