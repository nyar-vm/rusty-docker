#![warn(missing_docs)]

//! 配置管理
//!
//! 负责读取和管理 Docker 全局配置、配置和密钥。

use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use docker_types::{
    ConfigInfo, DockerConfig, DockerError, EndpointConfig, EndpointInfo, EndpointStatus,
    EndpointType, LogConfig, ResourceLimits, SecretInfo,
};

/// 结果类型
pub type Result<T> = std::result::Result<T, DockerError>;

/// 配置管理器
pub struct ConfigManager {
    /// 配置文件路径
    config_path: String,
    /// 配置存储目录
    configs_dir: String,
    /// 密钥存储目录
    secrets_dir: String,
    /// 端点存储目录
    endpoints_dir: String,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self> {
        // 确定配置文件路径
        let config_path = Self::get_config_path()?;
        let data_dir = Self::get_default_data_dir();
        let configs_dir = format!("{}/configs", data_dir);
        let secrets_dir = format!("{}/secrets", data_dir);
        let endpoints_dir = format!("{}/endpoints", data_dir);

        // 确保配置、密钥和端点目录存在
        fs::create_dir_all(&configs_dir)
            .map_err(|e| DockerError::io_error("create_dir_all", e.to_string()))?;
        fs::create_dir_all(&secrets_dir)
            .map_err(|e| DockerError::io_error("create_dir_all", e.to_string()))?;
        fs::create_dir_all(&endpoints_dir)
            .map_err(|e| DockerError::io_error("create_dir_all", e.to_string()))?;

        Ok(Self {
            config_path,
            configs_dir,
            secrets_dir,
            endpoints_dir,
        })
    }

    /// 获取配置文件路径
    fn get_config_path() -> Result<String> {
        // 在不同操作系统上的默认配置路径
        #[cfg(target_os = "windows")]
        let default_path = "C:\\ProgramData\\rusty-docker\\config.toml";

        #[cfg(target_os = "linux")]
        let default_path = "/etc/rusty-docker/config.toml";

        #[cfg(target_os = "macos")]
        let default_path = "/usr/local/etc/rusty-docker/config.toml";

        // 检查配置文件是否存在
        if Path::new(default_path).exists() {
            Ok(default_path.to_string())
        } else {
            // 如果配置文件不存在，创建默认配置
            Self::create_default_config(default_path)?;
            Ok(default_path.to_string())
        }
    }

    /// 创建默认配置文件
    fn create_default_config(path: &str) -> Result<()> {
        // 确保目录存在
        if let Some(dir) = Path::new(path).parent() {
            fs::create_dir_all(dir)
                .map_err(|e| DockerError::io_error("create_dir_all", e.to_string()))?;
        }

        // 创建默认配置
        let default_config = Self::get_default_config();
        let config_content = toml::to_string(&default_config)
            .map_err(|e| DockerError::parse_error("DockerConfig", e.to_string()))?;

        // 写入配置文件
        fs::write(path, config_content)
            .map_err(|e| DockerError::io_error("write", e.to_string()))?;
        Ok(())
    }

    /// 获取默认配置
    fn get_default_config() -> DockerConfig {
        DockerConfig {
            data_dir: Self::get_default_data_dir(),
            image_dir: format!("{}/images", Self::get_default_data_dir()),
            container_dir: format!("{}/containers", Self::get_default_data_dir()),
            network_dir: format!("{}/networks", Self::get_default_data_dir()),
            default_network: "default".to_string(),
            default_resources: ResourceLimits {
                cpu_limit: 1.0,
                memory_limit: 512,
                storage_limit: 10,
                network_limit: 10,
            },
            log_config: LogConfig {
                log_level: "info".to_string(),
                log_file: format!("{}/logs/docker.log", Self::get_default_data_dir()),
                max_log_size: 100,
            },
        }
    }

    /// 获取默认数据目录
    fn get_default_data_dir() -> String {
        #[cfg(target_os = "windows")]
        return "C:\\ProgramData\\rusty-docker\\data".to_string();

        #[cfg(target_os = "linux")]
        return "/var/lib/rusty-docker".to_string();

        #[cfg(target_os = "macos")]
        return "/usr/local/var/rusty-docker".to_string();
    }

    /// 获取配置
    pub fn get_config(&self) -> Result<DockerConfig> {
        // 读取配置文件
        let config_content = fs::read_to_string(&self.config_path)
            .map_err(|e| DockerError::io_error("read_to_string", e.to_string()))?;

        // 解析配置文件
        let config: DockerConfig = toml::from_str(&config_content)
            .map_err(|e| DockerError::parse_error("DockerConfig", e.to_string()))?;

        Ok(config)
    }

    /// 保存配置
    pub fn save_config(&self, config: &DockerConfig) -> Result<()> {
        // 序列化配置
        let config_content = toml::to_string(config)
            .map_err(|e| DockerError::parse_error("DockerConfig", e.to_string()))?;

        // 写入配置文件
        fs::write(&self.config_path, config_content)
            .map_err(|e| DockerError::io_error("write", e.to_string()))?;
        Ok(())
    }

    /// 生成唯一 ID
    fn generate_id(data: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        );
        format!("{:x}", hasher.finalize())
    }

    /// 创建配置
    pub fn create_config(
        &self,
        name: &str,
        data: &str,
        labels: HashMap<String, String>,
    ) -> Result<ConfigInfo> {
        let id = Self::generate_id(&format!("{}{}", name, data));
        let config_info = ConfigInfo {
            id: id.clone(),
            name: name.to_string(),
            data: data.to_string(),
            created_at: SystemTime::now(),
            labels,
        };

        // 序列化配置信息
        let config_content = toml::to_string(&config_info)
            .map_err(|e| DockerError::parse_error("ConfigInfo", e.to_string()))?;

        // 写入配置文件
        let config_path = format!("{}/{}.toml", self.configs_dir, id);
        fs::write(&config_path, config_content)
            .map_err(|e| DockerError::io_error("write", e.to_string()))?;

        Ok(config_info)
    }

    /// 更新配置
    pub fn update_config(
        &self,
        config_id: &str,
        data: &str,
        labels: HashMap<String, String>,
    ) -> Result<ConfigInfo> {
        // 检查配置是否存在
        let config_path = format!("{}/{}.toml", self.configs_dir, config_id);
        if !Path::new(&config_path).exists() {
            return Err(DockerError::not_found(
                "config",
                format!("Config {} not found", config_id),
            ));
        }

        // 读取现有配置
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| DockerError::io_error("read_to_string", e.to_string()))?;
        let mut config_info: ConfigInfo = toml::from_str(&config_content)
            .map_err(|e| DockerError::parse_error("ConfigInfo", e.to_string()))?;

        // 更新配置
        config_info.data = data.to_string();
        config_info.labels = labels;

        // 序列化并保存
        let updated_content = toml::to_string(&config_info)
            .map_err(|e| DockerError::parse_error("ConfigInfo", e.to_string()))?;
        fs::write(&config_path, updated_content)
            .map_err(|e| DockerError::io_error("write", e.to_string()))?;

        Ok(config_info)
    }

    /// 删除配置
    pub fn delete_config(&self, config_id: &str) -> Result<()> {
        let config_path = format!("{}/{}.toml", self.configs_dir, config_id);
        if !Path::new(&config_path).exists() {
            return Err(DockerError::not_found(
                "config",
                format!("Config {} not found", config_id),
            ));
        }

        fs::remove_file(&config_path)
            .map_err(|e| DockerError::io_error("remove_file", e.to_string()))?;
        Ok(())
    }

    /// 获取配置详细信息
    pub fn get_config_info(&self, config_id: &str) -> Result<ConfigInfo> {
        let config_path = format!("{}/{}.toml", self.configs_dir, config_id);
        if !Path::new(&config_path).exists() {
            return Err(DockerError::not_found(
                "config",
                format!("Config {} not found", config_id),
            ));
        }

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| DockerError::io_error("read_to_string", e.to_string()))?;
        let config_info: ConfigInfo = toml::from_str(&config_content)
            .map_err(|e| DockerError::parse_error("ConfigInfo", e.to_string()))?;

        Ok(config_info)
    }

    /// 列出所有配置
    pub fn list_configs(&self) -> Result<Vec<ConfigInfo>> {
        let mut configs = Vec::new();

        // 遍历配置目录
        for entry in fs::read_dir(&self.configs_dir)
            .map_err(|e| DockerError::io_error("read_dir", e.to_string()))?
        {
            let entry = entry.map_err(|e| DockerError::io_error("entry", e.to_string()))?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "toml" {
                let config_content = fs::read_to_string(&path)
                    .map_err(|e| DockerError::io_error("read_to_string", e.to_string()))?;
                let config_info: ConfigInfo = toml::from_str(&config_content)
                    .map_err(|e| DockerError::parse_error("ConfigInfo", e.to_string()))?;
                configs.push(config_info);
            }
        }

        Ok(configs)
    }

    /// 创建密钥
    pub fn create_secret(
        &self,
        name: &str,
        data: &str,
        labels: HashMap<String, String>,
    ) -> Result<SecretInfo> {
        let id = Self::generate_id(&format!("{}{}", name, data));
        let digest = Self::generate_id(data);

        let secret_info = SecretInfo {
            id: id.clone(),
            name: name.to_string(),
            created_at: SystemTime::now(),
            labels,
            digest,
        };

        // 序列化密钥信息
        let secret_content = toml::to_string(&secret_info)
            .map_err(|e| DockerError::parse_error("SecretInfo", e.to_string()))?;

        // 写入密钥文件
        let secret_path = format!("{}/{}.toml", self.secrets_dir, id);
        fs::write(&secret_path, secret_content)
            .map_err(|e| DockerError::io_error("write", e.to_string()))?;

        // 写入密钥数据（实际应用中应该加密存储）
        let data_path = format!("{}/{}.data", self.secrets_dir, id);
        fs::write(&data_path, data).map_err(|e| DockerError::io_error("write", e.to_string()))?;

        Ok(secret_info)
    }

    /// 删除密钥
    pub fn delete_secret(&self, secret_id: &str) -> Result<()> {
        let secret_path = format!("{}/{}.toml", self.secrets_dir, secret_id);
        let data_path = format!("{}/{}.data", self.secrets_dir, secret_id);

        if !Path::new(&secret_path).exists() {
            return Err(DockerError::not_found(
                "secret",
                format!("Secret {} not found", secret_id),
            ));
        }

        // 删除密钥文件和数据文件
        fs::remove_file(&secret_path)
            .map_err(|e| DockerError::io_error("remove_file", e.to_string()))?;
        if Path::new(&data_path).exists() {
            fs::remove_file(&data_path)
                .map_err(|e| DockerError::io_error("remove_file", e.to_string()))?;
        }

        Ok(())
    }

    /// 获取密钥详细信息
    pub fn get_secret_info(&self, secret_id: &str) -> Result<SecretInfo> {
        let secret_path = format!("{}/{}.toml", self.secrets_dir, secret_id);
        if !Path::new(&secret_path).exists() {
            return Err(DockerError::not_found(
                "secret",
                format!("Secret {} not found", secret_id),
            ));
        }

        let secret_content = fs::read_to_string(&secret_path)
            .map_err(|e| DockerError::io_error("read_to_string", e.to_string()))?;
        let secret_info: SecretInfo = toml::from_str(&secret_content)
            .map_err(|e| DockerError::parse_error("SecretInfo", e.to_string()))?;

        Ok(secret_info)
    }

    /// 列出所有密钥
    pub fn list_secrets(&self) -> Result<Vec<SecretInfo>> {
        let mut secrets = Vec::new();

        // 遍历密钥目录
        for entry in fs::read_dir(&self.secrets_dir)
            .map_err(|e| DockerError::io_error("read_dir", e.to_string()))?
        {
            let entry = entry.map_err(|e| DockerError::io_error("entry", e.to_string()))?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "toml" {
                let secret_content = fs::read_to_string(&path)
                    .map_err(|e| DockerError::io_error("read_to_string", e.to_string()))?;
                let secret_info: SecretInfo = toml::from_str(&secret_content)
                    .map_err(|e| DockerError::parse_error("SecretInfo", e.to_string()))?;
                secrets.push(secret_info);
            }
        }

        Ok(secrets)
    }

    /// 创建端点
    pub fn create_endpoint(
        &self,
        name: &str,
        endpoint_type: EndpointType,
        url: &str,
        use_tls: bool,
        tls_cert_path: Option<String>,
        tls_key_path: Option<String>,
        tls_ca_path: Option<String>,
        auth_token: Option<String>,
        labels: HashMap<String, String>,
    ) -> Result<EndpointInfo> {
        let endpoint_type_str = match endpoint_type {
            EndpointType::Local => "local",
            EndpointType::Remote => "remote",
            EndpointType::Cloud => "cloud",
        };
        let id = Self::generate_id(&format!("{}{}{}", name, url, endpoint_type_str));
        let config = EndpointConfig {
            id: id.clone(),
            name: name.to_string(),
            endpoint_type,
            url: url.to_string(),
            use_tls,
            tls_cert_path,
            tls_key_path,
            tls_ca_path,
            auth_token,
            labels,
        };

        let endpoint_info = EndpointInfo {
            config,
            status: EndpointStatus::Disconnected,
            created_at: SystemTime::now(),
            last_connected_at: None,
            connection_info: None,
        };

        // 序列化端点信息
        let endpoint_content = toml::to_string(&endpoint_info)
            .map_err(|e| DockerError::parse_error("EndpointInfo", e.to_string()))?;

        // 写入端点文件
        let endpoint_path = format!("{}/{}.toml", self.endpoints_dir, id);
        fs::write(&endpoint_path, endpoint_content)
            .map_err(|e| DockerError::io_error("write", e.to_string()))?;

        Ok(endpoint_info)
    }

    /// 更新端点
    pub fn update_endpoint(
        &self,
        endpoint_id: &str,
        name: &str,
        url: &str,
        use_tls: bool,
        tls_cert_path: Option<String>,
        tls_key_path: Option<String>,
        tls_ca_path: Option<String>,
        auth_token: Option<String>,
        labels: HashMap<String, String>,
    ) -> Result<EndpointInfo> {
        // 检查端点是否存在
        let endpoint_path = format!("{}/{}.toml", self.endpoints_dir, endpoint_id);
        if !Path::new(&endpoint_path).exists() {
            return Err(DockerError::not_found(
                "endpoint",
                format!("Endpoint {} not found", endpoint_id),
            ));
        }

        // 读取现有端点
        let endpoint_content = fs::read_to_string(&endpoint_path)
            .map_err(|e| DockerError::io_error("read_to_string", e.to_string()))?;
        let mut endpoint_info: EndpointInfo = toml::from_str(&endpoint_content)
            .map_err(|e| DockerError::parse_error("EndpointInfo", e.to_string()))?;

        // 更新端点配置
        endpoint_info.config.name = name.to_string();
        endpoint_info.config.url = url.to_string();
        endpoint_info.config.use_tls = use_tls;
        endpoint_info.config.tls_cert_path = tls_cert_path;
        endpoint_info.config.tls_key_path = tls_key_path;
        endpoint_info.config.tls_ca_path = tls_ca_path;
        endpoint_info.config.auth_token = auth_token;
        endpoint_info.config.labels = labels;

        // 序列化并保存
        let updated_content = toml::to_string(&endpoint_info)
            .map_err(|e| DockerError::parse_error("EndpointInfo", e.to_string()))?;
        fs::write(&endpoint_path, updated_content)
            .map_err(|e| DockerError::io_error("write", e.to_string()))?;

        Ok(endpoint_info)
    }

    /// 删除端点
    pub fn delete_endpoint(&self, endpoint_id: &str) -> Result<()> {
        let endpoint_path = format!("{}/{}.toml", self.endpoints_dir, endpoint_id);
        if !Path::new(&endpoint_path).exists() {
            return Err(DockerError::not_found(
                "endpoint",
                format!("Endpoint {} not found", endpoint_id),
            ));
        }

        fs::remove_file(&endpoint_path)
            .map_err(|e| DockerError::io_error("remove_file", e.to_string()))?;
        Ok(())
    }

    /// 获取端点详细信息
    pub fn get_endpoint_info(&self, endpoint_id: &str) -> Result<EndpointInfo> {
        let endpoint_path = format!("{}/{}.toml", self.endpoints_dir, endpoint_id);
        if !Path::new(&endpoint_path).exists() {
            return Err(DockerError::not_found(
                "endpoint",
                format!("Endpoint {} not found", endpoint_id),
            ));
        }

        let endpoint_content = fs::read_to_string(&endpoint_path)
            .map_err(|e| DockerError::io_error("read_to_string", e.to_string()))?;
        let endpoint_info: EndpointInfo = toml::from_str(&endpoint_content)
            .map_err(|e| DockerError::parse_error("EndpointInfo", e.to_string()))?;

        Ok(endpoint_info)
    }

    /// 列出所有端点
    pub fn list_endpoints(&self) -> Result<Vec<EndpointInfo>> {
        let mut endpoints = Vec::new();

        // 遍历端点目录
        for entry in fs::read_dir(&self.endpoints_dir)
            .map_err(|e| DockerError::io_error("read_dir", e.to_string()))?
        {
            let entry = entry.map_err(|e| DockerError::io_error("entry", e.to_string()))?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "toml" {
                let endpoint_content = fs::read_to_string(&path)
                    .map_err(|e| DockerError::io_error("read_to_string", e.to_string()))?;
                let endpoint_info: EndpointInfo = toml::from_str(&endpoint_content)
                    .map_err(|e| DockerError::parse_error("EndpointInfo", e.to_string()))?;
                endpoints.push(endpoint_info);
            }
        }

        Ok(endpoints)
    }

    /// 测试端点连接
    pub fn test_endpoint_connection(&self, endpoint_id: &str) -> Result<EndpointStatus> {
        // 这里应该实现实际的连接测试逻辑
        // 目前只是模拟连接成功
        Ok(EndpointStatus::Connected)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create config manager")
    }
}
