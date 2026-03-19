#![warn(missing_docs)]

use docker_types::DockerError;
use serde::{Deserialize, Serialize};

/// Container 结构体表示一个 Docker 容器
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Container {
    /// 容器 ID
    pub id: String,
    /// 容器名称
    pub name: Option<String>,
    /// 容器使用的镜像
    pub image: String,
    /// 容器状态
    pub status: ContainerStatus,
    /// 容器端口映射
    pub ports: Vec<String>,
    /// 容器环境变量
    pub environment: Vec<String>,
    /// 容器卷挂载
    pub volumes: Vec<String>,
    /// 容器 secrets
    pub secrets: Vec<String>,
    /// 容器添加的 capabilities
    pub cap_add: Vec<String>,
    /// 容器移除的 capabilities
    pub cap_drop: Vec<String>,
    /// 容器是否为特权模式
    pub privileged: bool,
    /// 容器是否为只读模式
    pub read_only: bool,
}

/// ContainerStatus 枚举表示容器的各种状态
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ContainerStatus {
    /// 容器已创建
    Created,
    /// 容器运行中
    Running,
    /// 容器已暂停
    Paused,
    /// 容器已停止
    Stopped,
    /// 容器已退出
    Exited,
    /// 容器已死亡
    Dead,
}

/// 结果类型
pub type Result<T> = std::result::Result<T, DockerError>;

/// ContainerManager trait 定义了容器管理的核心方法
pub trait ContainerManager: Send + Sync {
    /// 创建一个新容器
    ///
    /// # 参数
    /// * `image` - 容器使用的镜像名称
    /// * `name` - 容器的可选名称
    /// * `ports` - 端口映射列表
    /// * `environment` - 环境变量列表
    /// * `volumes` - 卷挂载列表
    /// * `restart_policy` - 重启策略
    /// * `healthcheck` - 健康检查配置
    /// * `deploy` - 部署配置
    /// * `secrets` - 容器 secrets 列表
    /// * `cap_add` - 容器添加的 capabilities 列表
    /// * `cap_drop` - 容器移除的 capabilities 列表
    /// * `privileged` - 容器是否为特权模式
    /// * `read_only` - 容器是否为只读模式
    ///
    /// # 返回值
    /// * `Ok(Container)` - 创建成功的容器信息
    /// * `Err(DockerError)` - 创建失败的错误信息
    fn create(
        &mut self,
        image: String,
        name: Option<String>,
        ports: Vec<String>,
        environment: Vec<String>,
        volumes: Vec<String>,
        restart_policy: Option<String>,
        healthcheck: Option<String>,
        deploy: Option<String>,
        secrets: Vec<String>,
        cap_add: Vec<String>,
        cap_drop: Vec<String>,
        privileged: bool,
        read_only: bool,
    ) -> Result<Container>;

    /// 启动指定容器
    ///
    /// # 参数
    /// * `container_id` - 容器 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 启动成功
    /// * `Err(DockerError)` - 启动失败的错误信息
    fn start(&mut self, container_id: &str) -> Result<()>;

    /// 停止指定容器
    ///
    /// # 参数
    /// * `container_id` - 容器 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 停止成功
    /// * `Err(DockerError)` - 停止失败的错误信息
    fn stop(&mut self, container_id: &str) -> Result<()>;

    /// 删除指定容器
    ///
    /// # 参数
    /// * `container_id` - 容器 ID
    ///
    /// # 返回值
    /// * `Ok(())` - 删除成功
    /// * `Err(DockerError)` - 删除失败的错误信息
    fn delete(&mut self, container_id: &str) -> Result<()>;

    /// 列出容器列表
    ///
    /// # 参数
    /// * `all` - 是否列出所有容器（包括已停止的）
    ///
    /// # 返回值
    /// * `Ok(Vec<Container>)` - 容器列表
    /// * `Err(DockerError)` - 列出失败的错误信息
    fn list(&mut self, all: bool) -> Result<Vec<Container>>;

    /// 查看容器详细信息
    ///
    /// # 参数
    /// * `container_id` - 容器 ID
    ///
    /// # 返回值
    /// * `Ok(Container)` - 容器详细信息
    /// * `Err(DockerError)` - 查看失败的错误信息
    fn inspect(&mut self, container_id: &str) -> Result<Container>;

    /// 获取容器日志
    ///
    /// # 参数
    /// * `container_id` - 容器 ID
    /// * `lines` - 要获取的日志行数
    /// * `follow` - 是否跟随日志输出
    ///
    /// # 返回值
    /// * `Ok(String)` - 容器日志
    /// * `Err(DockerError)` - 获取失败的错误信息
    fn get_logs(&mut self, container_id: &str, lines: Option<u32>, follow: bool) -> Result<String>;

    /// 执行容器内命令
    ///
    /// # 参数
    /// * `container_id` - 容器 ID
    /// * `command` - 要执行的命令
    /// * `shell` - 是否在 shell 中执行
    ///
    /// # 返回值
    /// * `Ok(String)` - 命令执行结果
    /// * `Err(DockerError)` - 执行失败的错误信息
    fn exec_command(&mut self, container_id: &str, command: &str, shell: bool) -> Result<String>;
}

/// RuntimeManager trait 定义了运行时管理的核心方法
pub trait RuntimeManager: Send + Sync {
    /// 初始化运行时
    ///
    /// # 返回值
    /// * `Ok(())` - 初始化成功
    /// * `Err(DockerError)` - 初始化失败的错误信息
    fn initialize(&mut self) -> Result<()>;

    /// 关闭运行时
    ///
    /// # 返回值
    /// * `Ok(())` - 关闭成功
    /// * `Err(DockerError)` - 关闭失败的错误信息
    fn shutdown(&mut self) -> Result<()>;

    /// 获取运行时状态
    ///
    /// # 返回值
    /// * `Ok(String)` - 运行时状态
    /// * `Err(DockerError)` - 获取失败的错误信息
    fn status(&mut self) -> Result<String>;

    /// 获取运行时版本
    ///
    /// # 返回值
    /// * `Ok(String)` - 运行时版本
    /// * `Err(DockerError)` - 获取失败的错误信息
    fn version(&mut self) -> Result<String>;
}

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod hyperv;

#[cfg(target_os = "windows")]
mod wsl2;

#[cfg(target_os = "windows")]
mod file_share;

#[cfg(target_os = "windows")]
mod network_manager;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
/// Linux 平台的容器管理器
pub use linux::LinuxContainerManager;

#[cfg(target_os = "macos")]
/// macOS 平台的容器管理器
pub use macos::MacOSContainerManager;

#[cfg(target_os = "windows")]
/// Windows 平台的容器管理器
pub use windows::WindowsContainerManager;

use once_cell::sync::Lazy;

/// 容器管理器单例
static CONTAINER_MANAGER: Lazy<Box<dyn ContainerManager>> = Lazy::new(|| {
    #[cfg(target_os = "linux")]
    return Box::new(LinuxContainerManager::new());
    
    #[cfg(target_os = "macos")]
    return Box::new(MacOSContainerManager::new());
    
    #[cfg(target_os = "windows")]
    return Box::new(WindowsContainerManager::new());
});

/// 运行时管理器单例
static RUNTIME_MANAGER: Lazy<Box<dyn RuntimeManager>> = Lazy::new(|| {
    #[cfg(target_os = "linux")]
    return Box::new(LinuxContainerManager::new());
    
    #[cfg(target_os = "macos")]
    return Box::new(MacOSContainerManager::new());
    
    #[cfg(target_os = "windows")]
    return Box::new(WindowsContainerManager::new());
});

/// 获取适合当前平台的容器管理器
///
/// # 返回值
/// * `&'static dyn ContainerManager` - 容器管理器实例
pub fn get_container_manager() -> &'static dyn ContainerManager {
    CONTAINER_MANAGER.as_ref()
}

/// 获取适合当前平台的运行时管理器
///
/// # 返回值
/// * `&'static dyn RuntimeManager` - 运行时管理器实例
pub fn get_runtime_manager() -> &'static dyn RuntimeManager {
    RUNTIME_MANAGER.as_ref()
}
