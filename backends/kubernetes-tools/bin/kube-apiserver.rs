//! Kubernetes API Server
//!
//! 提供Kubernetes集群的RESTful API接口，是集群的核心组件

use clap::Parser;
use docker_tools::create_base_command;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Address to listen on
    #[arg(short = 'a', long, default_value = "127.0.0.1:8080")]
    host: String,

    /// Etcd server URL
    #[arg(long, default_value = "http://127.0.0.1:2379")]
    etcd_servers: String,

    /// Secure port
    #[arg(long, default_value = "6443")]
    secure_port: u16,
}

/// Kubernetes API资源类型
#[derive(Debug, Serialize, Deserialize)]
enum ApiResource {
    Pod,
    Service,
    Deployment,
    ReplicaSet,
    ConfigMap,
    Secret,
    Node,
    Namespace,
    PersistentVolume,
    PersistentVolumeClaim,
}

/// API请求
#[derive(Debug, Serialize, Deserialize)]
struct ApiRequest {
    verb: String,
    resource: ApiResource,
    namespace: Option<String>,
    name: Option<String>,
    body: Option<serde_json::Value>,
}

/// API响应
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    kind: String,
    api_version: String,
    metadata: serde_json::Value,
    spec: Option<serde_json::Value>,
    status: Option<serde_json::Value>,
}

/// API服务器状态
struct ApiServerState {
    resources: std::collections::HashMap<String, serde_json::Value>,
}

async fn handle_http_request(request: &str, state: Arc<Mutex<ApiServerState>>) -> String {
    // 解析 HTTP 请求
    let lines: Vec<&str> = request.split("\r\n").collect();
    if lines.is_empty() {
        return "HTTP/1.1 400 Bad Request\r\n\r\n".to_string();
    }

    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return "HTTP/1.1 400 Bad Request\r\n\r\n".to_string();
    }

    let method = parts[0];
    let path = parts[1];
    let _version = parts[2];

    // 提取请求体
    let mut body = "";
    if let Some(idx) = lines.iter().position(|&line| line.is_empty()) {
        if idx + 1 < lines.len() {
            body = lines[idx + 1];
        }
    }

    // 处理不同的 API 端点
    let response = match (method, path) {
        // 健康检查
        ("GET", "/healthz") => "HTTP/1.1 200 OK\r\n\r\nok".to_string(),
        // 版本信息
        ("GET", "/version") => {
            let version_info = serde_json::json!({
                "major": "1",
                "minor": "28",
                "gitVersion": "v1.28.0",
                "gitCommit": "abc123",
                "gitTreeState": "clean",
                "buildDate": "2026-03-17T00:00:00Z",
                "goVersion": "go1.20",
                "compiler": "gc",
                "platform": "windows/amd64"
            });
            let response_str = serde_json::to_string(&version_info).unwrap();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response_str.len(),
                response_str
            )
        }
        // API 资源
        _ if path.starts_with("/api/") || path.starts_with("/apis/") => {
            handle_api_request(method, path, body, state).await
        }
        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    };

    response.to_string()
}

async fn handle_api_request(
    method: &str,
    path: &str,
    body: &str,
    state: Arc<Mutex<ApiServerState>>,
) -> String {
    // 解析API路径
    let parts: Vec<&str> = path.split("/").filter(|s| !s.is_empty()).collect();
    if parts.len() < 2 {
        return "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
    }

    let api_version = parts[1];
    let resource_type = if parts.len() > 2 { parts[2] } else { "" };
    let namespace = if parts.len() > 3 && parts[2] == "namespaces" {
        Some(parts[3])
    } else {
        None
    };
    let resource_name = if parts.len() > 4 {
        Some(parts[4])
    } else if parts.len() > 3 && parts[2] != "namespaces" {
        Some(parts[3])
    } else {
        None
    };

    // 构建etcd键
    let mut etcd_key = format!("/kubernetes/{}/{}", api_version, resource_type);
    if let Some(ns) = namespace {
        etcd_key.push_str(format!("/namespaces/{}", ns).as_str());
    }
    if let Some(name) = resource_name {
        etcd_key.push_str(format!("/{}", name).as_str());
    }

    let mut state = state.lock().await;

    match method {
        "GET" => {
            // 从HashMap获取资源
            if let Some(value) = state.resources.get(&etcd_key) {
                let value_str = serde_json::to_string(value).unwrap();
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    value_str.len(),
                    value_str
                )
            } else {
                "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
            }
        }
        "POST" => {
            // 向HashMap写入资源
            if body.is_empty() {
                return "HTTP/1.1 400 Bad Request\r\n\r\n".to_string();
            }

            match serde_json::from_str(body) {
                Ok(value) => {
                    state.resources.insert(etcd_key, value);
                    format!(
                        "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    )
                }
                Err(e) => {
                    eprintln!("Error parsing JSON: {:?}", e);
                    "HTTP/1.1 400 Bad Request\r\n\r\n".to_string()
                }
            }
        }
        "PUT" => {
            // 更新HashMap中的资源
            if body.is_empty() {
                return "HTTP/1.1 400 Bad Request\r\n\r\n".to_string();
            }

            match serde_json::from_str(body) {
                Ok(value) => {
                    state.resources.insert(etcd_key, value);
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    )
                }
                Err(e) => {
                    eprintln!("Error parsing JSON: {:?}", e);
                    "HTTP/1.1 400 Bad Request\r\n\r\n".to_string()
                }
            }
        }
        "DELETE" => {
            // 从HashMap删除资源
            if state.resources.remove(&etcd_key).is_some() {
                "HTTP/1.1 204 No Content\r\n\r\n".to_string()
            } else {
                "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
            }
        }
        _ => "HTTP/1.1 405 Method Not Allowed\r\n\r\n".to_string(),
    }
}

async fn handle_client(mut stream: TcpStream, state: Arc<Mutex<ApiServerState>>) {
    let mut buffer = [0; 4096];
    let n = match stream.read(&mut buffer).await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error reading from stream: {:?}", e);
            return;
        }
    };

    let request = String::from_utf8_lossy(&buffer[..n]);
    println!("Received request: {}", request);

    let response = handle_http_request(&request, state).await;

    if let Err(e) = stream.write_all(response.as_bytes()).await {
        eprintln!("Error writing to stream: {:?}", e);
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let listener = match TcpListener::bind(&cli.host).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error binding to address: {:?}", e);
            std::process::exit(1);
        }
    };

    // 初始化API服务器状态
    let state = ApiServerState {
        resources: std::collections::HashMap::new(),
    };

    let state = Arc::new(Mutex::new(state));
    println!("Kubernetes API Server listening on {}", cli.host);
    println!("Secure port: {}", cli.secure_port);
    println!("Etcd servers: {}", cli.etcd_servers);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    handle_client(stream, state_clone).await;
                });
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
