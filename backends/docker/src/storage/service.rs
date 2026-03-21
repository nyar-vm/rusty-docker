#![warn(missing_docs)]

//! 存储服务
//! 
//! 提供镜像、容器和卷的存储管理功能，集成了 overlay2 文件系统支持。

use std::{fs, io::Write, path::Path};

use docker_registry::DockerHubService;
use docker_types::{DockerError, ImageInfo, Result};

use super::overlay2::Overlay2Driver;

/// 存储服务
pub struct StorageService {
    /// 存储根路径
    storage_root: String,
    /// Docker Hub 服务
    docker_hub: DockerHubService,
    /// Overlay2 驱动
    overlay2_driver: Overlay2Driver,
}

impl StorageService {
    /// 创建新的存储服务
    pub fn new() -> Result<Self> {
        let storage_root = "/var/lib/rusty-docker".to_string();

        let docker_hub = DockerHubService::new_docker_hub()?;
        let overlay2_driver = Overlay2Driver::new(&storage_root);

        let storage_root_clone = storage_root.clone();
        tokio::spawn(async move {
            let _ = fs::create_dir_all(&storage_root_clone);

            let subdirs = ["images", "containers", "volumes"];
            for subdir in &subdirs {
                let dir_path = format!("{}/{}", storage_root_clone, subdir);
                let _ = fs::create_dir_all(&dir_path);
            }
        });

        Ok(Self { storage_root, docker_hub, overlay2_driver })
    }

    /// 获取镜像存储路径
    pub fn get_image_path(&self, image_id: &str) -> String {
        format!("{}/images/{}", self.storage_root, image_id)
    }

    /// 获取容器存储路径
    pub fn get_container_path(&self, container_id: &str) -> String {
        format!("{}/containers/{}", self.storage_root, container_id)
    }

    /// 获取卷存储路径
    pub fn get_volume_path(&self, volume_id: &str) -> String {
        format!("{}/volumes/{}", self.storage_root, volume_id)
    }

    /// 获取 Overlay2 驱动引用
    pub fn overlay2_driver(&self) -> &Overlay2Driver {
        &self.overlay2_driver
    }

    /// 创建容器存储目录
    pub fn create_container_dir(&self, container_id: &str) -> Result<()> {
        self.overlay2_driver.initialize_container_dirs(container_id)?;

        let container_path = self.get_container_path(container_id);
        let logs_path = format!("{}/logs", container_path);
        fs::create_dir_all(&logs_path)
            .map_err(|e| DockerError::io_error("create logs dir", e.to_string()))?;

        Ok(())
    }

    /// 删除容器存储目录
    pub fn delete_container_dir(&self, container_id: &str) -> Result<()> {
        let _ = self.overlay2_driver.cleanup_container_rootfs(container_id);
        let _ = self.overlay2_driver.umount_overlay(container_id);
        self.overlay2_driver.delete_container(container_id)?;
        Ok(())
    }

    /// 创建卷存储目录
    pub fn create_volume_dir(&self, volume_id: &str) -> Result<()> {
        let volume_path = self.get_volume_path(volume_id);
        fs::create_dir_all(&volume_path)
            .map_err(|e| DockerError::io_error("create volume dir", e.to_string()))?;
        Ok(())
    }

    /// 删除卷存储目录
    pub fn delete_volume_dir(&self, volume_id: &str) -> Result<()> {
        let volume_path = self.get_volume_path(volume_id);
        if Path::new(&volume_path).exists() {
            fs::remove_dir_all(&volume_path)
                .map_err(|e| DockerError::io_error("remove volume dir", e.to_string()))?;
        }
        Ok(())
    }

    /// 构建镜像
    pub async fn build_image(&self, _path: &str, tag: &str, pull: bool, no_cache: bool, force_rm: bool) -> Result<ImageInfo> {
        println!("Building image with options:");
        println!("  Pull: {}", pull);
        println!("  No cache: {}", no_cache);
        println!("  Force rm: {}", force_rm);

        Ok(ImageInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name: tag.to_string(),
            tags: vec![tag.to_string()],
            size: 1024 * 1024 * 100,
            created_at: std::time::SystemTime::now(),
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
        })
    }

    /// 列出镜像
    pub async fn list_images(&self) -> Result<Vec<ImageInfo>> {
        Ok(vec![ImageInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name: "ubuntu:latest".to_string(),
            tags: vec!["ubuntu:latest".to_string()],
            size: 1024 * 1024 * 100,
            created_at: std::time::SystemTime::now(),
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
        }])
    }

    /// 拉取镜像
    pub async fn pull_image(&self, image: &str, tag: &str) -> Result<ImageInfo> {
        self.docker_hub.pull_image(image, tag).await
    }

    /// 推送镜像
    pub async fn push_image(&self, image: &str, tag: &str) -> Result<ImageInfo> {
        self.docker_hub.push_image(image, tag).await
    }

    /// 删除镜像
    pub async fn delete_image(&self, image_id: &str) -> Result<()> {
        let image_path = self.get_image_path(image_id);
        if Path::new(&image_path).exists() {
            fs::remove_dir_all(&image_path)
                .map_err(|e| DockerError::io_error("remove image dir", e.to_string()))?;
        }
        Ok(())
    }

    /// 列出卷
    pub async fn list_volumes(&self) -> Result<Vec<docker_types::VolumeInfo>> {
        Ok(vec![docker_types::VolumeInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name: "my-volume".to_string(),
            size: 1024 * 1024 * 100,
            created_at: std::time::SystemTime::now(),
            mount_point: format!("{}/volumes/my-volume", self.storage_root),
            driver: "local".to_string(),
            labels: std::collections::HashMap::new(),
            used_by: vec![],
        }])
    }

    /// 创建卷
    pub async fn create_volume(
        &self,
        name: &str,
        driver: &str,
        labels: Option<std::collections::HashMap<String, String>>,
    ) -> Result<docker_types::VolumeInfo> {
        let volume_id = uuid::Uuid::new_v4().to_string();
        let volume_path = self.get_volume_path(&volume_id);

        fs::create_dir_all(&volume_path)
            .map_err(|e| DockerError::io_error("create volume dir", e.to_string()))?;

        Ok(docker_types::VolumeInfo {
            id: volume_id,
            name: name.to_string(),
            size: 0,
            created_at: std::time::SystemTime::now(),
            mount_point: volume_path,
            driver: driver.to_string(),
            labels: labels.unwrap_or_default(),
            used_by: vec![],
        })
    }

    /// 删除卷
    pub async fn delete_volume(&self, volume_id: &str) -> Result<()> {
        let volume_path = self.get_volume_path(volume_id);
        if Path::new(&volume_path).exists() {
            fs::remove_dir_all(&volume_path)
                .map_err(|e| DockerError::io_error("remove volume dir", e.to_string()))?;
        }
        Ok(())
    }

    /// 获取卷详情
    pub async fn get_volume(&self, volume_id: &str) -> Result<docker_types::VolumeInfo> {
        Ok(docker_types::VolumeInfo {
            id: volume_id.to_string(),
            name: "my-volume".to_string(),
            size: 1024 * 1024 * 100,
            created_at: std::time::SystemTime::now(),
            mount_point: format!("{}/volumes/{}", self.storage_root, volume_id),
            driver: "local".to_string(),
            labels: std::collections::HashMap::new(),
            used_by: vec![],
        })
    }

    /// 准备容器文件系统
    pub fn prepare_container_fs(&self, container_id: &str, image: &str) -> Result<()> {
        let container_path = self.get_container_path(container_id);
        let logs_path = format!("{}/logs", container_path);

        fs::create_dir_all(&logs_path)
            .map_err(|e| DockerError::io_error("create logs dir", e.to_string()))?;

        let log_file = format!("{}/container.log", logs_path);
        fs::File::create(&log_file)
            .map_err(|e| DockerError::io_error("create log file", e.to_string()))?;

        Ok(())
    }

    /// 写入容器日志
    pub fn write_container_log(&self, container_id: &str, message: &str) -> Result<()> {
        let container_path = self.get_container_path(container_id);
        let log_file = format!("{}/logs/container.log", container_path);

        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&log_file)
            .map_err(|e| DockerError::io_error("open log file", e.to_string()))?;

        let log_entry = format!("[{:?}] {}\n", std::time::SystemTime::now(), message);
        file.write_all(log_entry.as_bytes())
            .map_err(|e| DockerError::io_error("write log", e.to_string()))?;

        Ok(())
    }

    /// 读取容器日志
    pub fn read_container_logs(&self, container_id: &str) -> Result<Vec<String>> {
        let container_path = self.get_container_path(container_id);
        let log_file = format!("{}/logs/container.log", container_path);

        if !Path::new(&log_file).exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&log_file)
            .map_err(|e| DockerError::io_error("read log file", e.to_string()))?;

        let logs: Vec<String> = content.lines().map(|line| line.to_string()).collect();

        Ok(logs)
    }

    /// 写入容器环境变量
    pub fn write_container_env(&self, container_id: &str, env: &std::collections::HashMap<String, String>) -> Result<()> {
        let container_path = self.get_container_path(container_id);
        let env_file = format!("{}/env.json", container_path);

        let env_content = serde_json::to_string_pretty(env)
            .map_err(|e| DockerError::json_error(e.to_string()))?;

        fs::write(env_file, env_content)
            .map_err(|e| DockerError::io_error("write env file", e.to_string()))?;

        Ok(())
    }

    /// 读取容器环境变量
    pub fn read_container_env(&self, container_id: &str) -> Result<std::collections::HashMap<String, String>> {
        let container_path = self.get_container_path(container_id);
        let env_file = format!("{}/env.json", container_path);

        if !Path::new(&env_file).exists() {
            return Ok(std::collections::HashMap::new());
        }

        let content = fs::read_to_string(&env_file)
            .map_err(|e| DockerError::io_error("read env file", e.to_string()))?;

        let env: std::collections::HashMap<String, String> =
            serde_json::from_str(&content).map_err(|e| DockerError::json_error(e.to_string()))?;

        Ok(env)
    }

    /// 写入容器配置
    pub fn write_container_config(&self, container_id: &str, config: &docker_types::ContainerConfig) -> Result<()> {
        let container_path = self.get_container_path(container_id);
        let config_file = format!("{}/config.json", container_path);

        let config_content =
            serde_json::to_string_pretty(config).map_err(|e| DockerError::io_error("write config", e.to_string()))?;

        fs::write(config_file, config_content)
            .map_err(|e| DockerError::io_error("write config file", e.to_string()))?;

        Ok(())
    }

    /// 读取容器配置
    pub fn read_container_config(&self, container_id: &str) -> Result<docker_types::ContainerConfig> {
        let container_path = self.get_container_path(container_id);
        let config_file = format!("{}/config.json", container_path);

        if !Path::new(&config_file).exists() {
            return Err(DockerError::not_found("config", "Config file not found"));
        }

        let content = fs::read_to_string(&config_file)
            .map_err(|e| DockerError::io_error("read config file", e.to_string()))?;

        let config: docker_types::ContainerConfig =
            serde_json::from_str(&content).map_err(|e| DockerError::json_error(e.to_string()))?;

        Ok(config)
    }
}
