//! Oh My Crab - OpenCrab GUI Client
//!
//! A Tauri-based GUI client for connecting to OpenCrab or OpenCrab servers.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::Manager;

use docker::{Docker, RustyDocker};
use docker_types::{ContainerInfo, ImageInfo, Result as DockerResult};

/// 应用状态管理
struct AppState {
    /// Rusty Docker 实例
    docker: Mutex<Option<Docker>>,
}

impl AppState {
    /// 创建新的应用状态
    fn new() -> Self {
        AppState {
            docker: Mutex::new(None),
        }
    }
}

/// 问候用户
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ==================== 容器管理命令 ====================

/// 初始化 Docker 服务
#[tauri::command]
async fn init_docker(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut docker_lock = state.docker.lock().unwrap();
    if docker_lock.is_none() {
        let docker = RustyDocker::new().map_err(|e| e.to_string())?;
        docker.start().await;
        *docker_lock = Some(docker);
    }
    Ok(())
}

/// 列出容器
#[tauri::command]
async fn list_containers(
    state: tauri::State<'_, AppState>,
    all: bool,
) -> Result<Vec<ContainerInfo>, String> {
    let docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_ref().ok_or("Docker 服务未初始化")?;
    docker.list_containers(all).await.map_err(|e| e.to_string())
}

/// 启动容器
#[tauri::command]
async fn start_container(
    state: tauri::State<'_, AppState>,
    container_id: String,
) -> Result<(), String> {
    let mut docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_mut().ok_or("Docker 服务未初始化")?;
    let runtime = docker.get_runtime();
    runtime
        .start_container(&container_id)
        .await
        .map_err(|e| e.to_string())
}

/// 停止容器
#[tauri::command]
async fn stop_container(
    state: tauri::State<'_, AppState>,
    container_id: String,
) -> Result<(), String> {
    let mut docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_mut().ok_or("Docker 服务未初始化")?;
    docker
        .stop_container(&container_id)
        .await
        .map_err(|e| e.to_string())
}

/// 重启容器
#[tauri::command]
async fn restart_container(
    state: tauri::State<'_, AppState>,
    container_id: String,
) -> Result<(), String> {
    let mut docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_mut().ok_or("Docker 服务未初始化")?;
    let runtime = docker.get_runtime();
    // 先停止容器
    docker
        .stop_container(&container_id)
        .await
        .map_err(|e| e.to_string())?;
    // 再启动容器
    runtime
        .start_container(&container_id)
        .await
        .map_err(|e| e.to_string())
}

/// 删除容器
#[tauri::command]
async fn delete_container(
    state: tauri::State<'_, AppState>,
    container_id: String,
) -> Result<(), String> {
    let mut docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_mut().ok_or("Docker 服务未初始化")?;
    docker
        .remove_container(&container_id)
        .await
        .map_err(|e| e.to_string())
}

/// 运行新容器
#[tauri::command]
async fn run_container(
    state: tauri::State<'_, AppState>,
    image: String,
    name: Option<String>,
    ports: Vec<String>,
    network_name: Option<String>,
    network_mode: Option<String>,
    aliases: Option<Vec<String>>,
    enable_ipv6: bool,
) -> Result<ContainerInfo, String> {
    let mut docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_mut().ok_or("Docker 服务未初始化")?;
    docker
        .run(
            image,
            name,
            ports,
            network_name,
            network_mode,
            aliases,
            enable_ipv6,
        )
        .await
        .map_err(|e| e.to_string())
}

// ==================== 镜像管理命令 ====================

/// 列出镜像
#[tauri::command]
async fn list_images(state: tauri::State<'_, AppState>) -> Result<Vec<ImageInfo>, String> {
    let docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_ref().ok_or("Docker 服务未初始化")?;
    docker.list_images().await.map_err(|e| e.to_string())
}

/// 拉取镜像
#[tauri::command]
async fn pull_image(
    state: tauri::State<'_, AppState>,
    image_name: String,
) -> Result<ImageInfo, String> {
    let docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_ref().ok_or("Docker 服务未初始化")?;
    // 解析镜像名称和标签
    let parts: Vec<&str> = image_name.split(':').collect();
    let (image, tag) = if parts.len() > 1 {
        (parts[0], parts[1])
    } else {
        (image_name.as_str(), "latest")
    };
    docker
        .pull_image(image, tag)
        .await
        .map_err(|e| e.to_string())
}

/// 删除镜像
#[tauri::command]
async fn delete_image(state: tauri::State<'_, AppState>, image_id: String) -> Result<(), String> {
    let docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_ref().ok_or("Docker 服务未初始化")?;
    docker
        .delete_image(&image_id)
        .await
        .map_err(|e| e.to_string())
}

// ==================== 网络管理命令 ====================

/// 列出网络
#[tauri::command]
async fn list_networks(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<docker_types::NetworkConfigInfo>, String> {
    let docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_ref().ok_or("Docker 服务未初始化")?;
    docker.list_networks().await.map_err(|e| e.to_string())
}

/// 创建网络
#[tauri::command]
async fn create_network(
    state: tauri::State<'_, AppState>,
    name: String,
    driver: String,
    enable_ipv6: bool,
    options: Option<std::collections::HashMap<String, String>>,
) -> Result<docker_types::NetworkConfigInfo, String> {
    let mut docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_mut().ok_or("Docker 服务未初始化")?;
    docker
        .create_network(name, driver, enable_ipv6, options)
        .await
        .map_err(|e| e.to_string())
}

/// 删除网络
#[tauri::command]
async fn delete_network(
    state: tauri::State<'_, AppState>,
    network_id: String,
) -> Result<(), String> {
    let mut docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_mut().ok_or("Docker 服务未初始化")?;
    docker
        .delete_network(&network_id)
        .await
        .map_err(|e| e.to_string())
}

// ==================== 卷管理命令 ====================

/// 列出卷
#[tauri::command]
async fn list_volumes(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<docker_types::VolumeInfo>, String> {
    let docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_ref().ok_or("Docker 服务未初始化")?;
    docker.list_volumes().await.map_err(|e| e.to_string())
}

/// 创建卷
#[tauri::command]
async fn create_volume(
    state: tauri::State<'_, AppState>,
    name: String,
    driver: String,
    labels: Option<std::collections::HashMap<String, String>>,
) -> Result<docker_types::VolumeInfo, String> {
    let mut docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_mut().ok_or("Docker 服务未初始化")?;
    docker
        .create_volume(name, driver, labels)
        .await
        .map_err(|e| e.to_string())
}

/// 删除卷
#[tauri::command]
async fn delete_volume(state: tauri::State<'_, AppState>, volume_id: String) -> Result<(), String> {
    let mut docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_mut().ok_or("Docker 服务未初始化")?;
    docker
        .delete_volume(&volume_id)
        .await
        .map_err(|e| e.to_string())
}

// ==================== 系统状态命令 ====================

/// 获取系统状态
#[tauri::command]
async fn get_system_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_ref().ok_or("Docker 服务未初始化")?;
    let system_status = docker
        .get_system_status()
        .await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(system_status).map_err(|e| e.to_string())?)
}

// ==================== 容器日志和终端命令 ====================

/// 获取容器日志
#[tauri::command]
async fn get_container_logs(
    state: tauri::State<'_, AppState>,
    container_id: String,
    lines: Option<u32>,
    follow: bool,
) -> Result<String, String> {
    let docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_ref().ok_or("Docker 服务未初始化")?;
    let logs = docker
        .get_container_logs(&container_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(logs.join("\n"))
}

/// 执行容器命令
#[tauri::command]
async fn exec_container_command(
    state: tauri::State<'_, AppState>,
    container_id: String,
    command: String,
    shell: bool,
) -> Result<String, String> {
    let docker_lock = state.docker.lock().unwrap();
    let docker = docker_lock.as_ref().ok_or("Docker 服务未初始化")?;
    let result = docker
        .exec_command(&container_id, &[command])
        .await
        .map_err(|e| e.to_string())?;
    Ok(result)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler!(
            greet,
            init_docker,
            list_containers,
            start_container,
            stop_container,
            restart_container,
            delete_container,
            run_container,
            list_images,
            pull_image,
            delete_image,
            list_networks,
            create_network,
            delete_network,
            list_volumes,
            create_volume,
            delete_volume,
            get_system_status,
            get_container_logs,
            exec_container_command,
        ))
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
