use std::path::{Path, PathBuf};

/// 文件共享管理器
#[derive(Debug)]
pub struct FileShareManager {
    /// 主机路径到容器路径的映射
    pub mounts: Vec<(PathBuf, PathBuf)>,
}

impl FileShareManager {
    /// 创建新的文件共享管理器
    pub fn new() -> Self {
        Self { mounts: Vec::new() }
    }

    /// 添加挂载点
    pub fn add_mount(&mut self, host_path: &str, container_path: &str) -> Result<(), String> {
        let host_path = Path::new(host_path).to_path_buf();
        if !host_path.exists() {
            return Err(format!("Host path does not exist: {}", host_path.display()));
        }

        let container_path = Path::new(container_path).to_path_buf();
        self.mounts.push((host_path, container_path));
        Ok(())
    }

    /// 获取容器路径对应的主机路径
    pub fn get_host_path(&self, container_path: &str) -> Option<PathBuf> {
        for (host_path, cont_path) in &self.mounts {
            if cont_path == Path::new(container_path) {
                return Some(host_path.clone());
            }
        }
        None
    }

    /// 获取主机路径对应的容器路径
    pub fn get_container_path(&self, host_path: &str) -> Option<PathBuf> {
        for (host_path, cont_path) in &self.mounts {
            if host_path == Path::new(host_path) {
                return Some(cont_path.clone());
            }
        }
        None
    }

    /// 将 Windows 路径转换为 WSL 路径
    pub fn windows_to_wsl_path(&self, windows_path: &str) -> String {
        let path = windows_path.replace('\\', "/");
        let path = path.replace(":", "");
        format!("/mnt/{}", path.to_lowercase())
    }

    /// 将 WSL 路径转换为 Windows 路径
    pub fn wsl_to_windows_path(&self, wsl_path: &str) -> String {
        if wsl_path.starts_with("/mnt/") {
            let path = wsl_path.strip_prefix("/mnt/").unwrap();
            let drive = &path[0..1].to_uppercase();
            let rest = &path[1..];
            format!("{}:{}", drive, rest.replace('/', "\\"))
        }
        else {
            wsl_path.to_string()
        }
    }
}
