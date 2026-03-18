use clap::Parser;
use docker::Docker;
use docker_types::{ContainerInfo, ImageInfo, Result as DockerResult};
use serde_json::{Value, from_str, to_string};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Address to listen on
    #[arg(short = 'a', long, default_value = "127.0.0.1:2375")]
    host: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum DockerCommand {
    Run { image: String, name: Option<String>, ports: Vec<String> },
    Ps { all: bool },
    Stop { container: String },
    Rm { container: String },
    Build { path: String, tag: String },
    Images,
    Pull { image: String, tag: String },
    Rmi { image: String },
    Inspect { container: String },
    Start { container: String },
    Pause { container: String },
    Unpause { container: String },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum DockerResponse {
    Ok(Value),
    Error(String),
}

async fn handle_http_request(request: &str, docker: Arc<Mutex<Docker>>) -> String {
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
        ("POST", "/containers/create") => {
            // 解析请求体
            if let Ok(req) = from_str::<serde_json::Value>(body) {
                let image = req["Image"].as_str().unwrap_or("").to_string();
                let name = req["Name"].as_str().map(|s| s.to_string());
                let mut ports = Vec::new();

                if let Some(ports_obj) = req["HostConfig"]["PortBindings"].as_object() {
                    for (container_port, host_ports) in ports_obj {
                        if let Some(host_ports_array) = host_ports.as_array() {
                            for host_port in host_ports_array {
                                if let Some(host_port_str) = host_port["HostPort"].as_str() {
                                    ports.push(format!("{}:{}", host_port_str, container_port.split("/").next().unwrap()));
                                }
                            }
                        }
                    }
                }

                let mut docker = docker.lock().await;
                match docker.run(image, name, ports, None, None, None, false, false).await {
                    Ok(container) => {
                        let container_json = to_string(&container).unwrap();
                        let container_value: Value = from_str(&container_json).unwrap();
                        DockerResponse::Ok(container_value)
                    }
                    Err(e) => DockerResponse::Error(format!("{:?}", e)),
                }
            }
            else {
                DockerResponse::Error("Invalid request body".to_string())
            }
        }
        ("GET", "/containers/json") => {
            let all = path.contains("all=true");
            let mut docker = docker.lock().await;
            match docker.list_containers(all).await {
                Ok(containers) => {
                    let containers_json = to_string(&containers).unwrap();
                    let containers_value: Value = from_str(&containers_json).unwrap();
                    DockerResponse::Ok(containers_value)
                }
                Err(e) => DockerResponse::Error(format!("{:?}", e)),
            }
        }
        ("POST", "/containers/*/stop") => {
            let container_id = path.split("/").nth(2).unwrap_or("");
            let mut docker = docker.lock().await;
            match docker.stop_container(container_id).await {
                Ok(_) => {
                    let response_json = to_string(&format!("Container {} stopped", container_id)).unwrap();
                    let response_value: Value = from_str(&response_json).unwrap();
                    DockerResponse::Ok(response_value)
                }
                Err(e) => DockerResponse::Error(format!("{:?}", e)),
            }
        }
        ("DELETE", "/containers/*") => {
            let container_id = path.split("/").nth(2).unwrap_or("");
            let mut docker = docker.lock().await;
            match docker.remove_container(container_id).await {
                Ok(_) => {
                    let response_json = to_string(&format!("Container {} removed", container_id)).unwrap();
                    let response_value: Value = from_str(&response_json).unwrap();
                    DockerResponse::Ok(response_value)
                }
                Err(e) => DockerResponse::Error(format!("{:?}", e)),
            }
        }
        ("POST", "/build") => {
            // 解析请求体
            if let Ok(req) = from_str::<serde_json::Value>(body) {
                let path = req["path"].as_str().unwrap_or("").to_string();
                let tag = req["tag"].as_str().unwrap_or("").to_string();
                let mut docker = docker.lock().await;
                match docker.build_image(&path, &tag, false, false, false).await {
                    Ok(image) => {
                        let image_json = to_string(&image).unwrap();
                        let image_value: Value = from_str(&image_json).unwrap();
                        DockerResponse::Ok(image_value)
                    }
                    Err(e) => DockerResponse::Error(format!("{:?}", e)),
                }
            }
            else {
                DockerResponse::Error("Invalid request body".to_string())
            }
        }
        ("GET", "/images/json") => {
            let mut docker = docker.lock().await;
            match docker.list_images().await {
                Ok(images) => {
                    let images_json = to_string(&images).unwrap();
                    let images_value: Value = from_str(&images_json).unwrap();
                    DockerResponse::Ok(images_value)
                }
                Err(e) => DockerResponse::Error(format!("{:?}", e)),
            }
        }
        ("POST", "/images/create") => {
            // 解析请求体
            if let Ok(req) = from_str::<serde_json::Value>(body) {
                let image = req["fromImage"].as_str().unwrap_or("").to_string();
                let tag = req["tag"].as_str().unwrap_or("latest").to_string();
                let mut docker = docker.lock().await;
                match docker.pull_image(&image, &tag).await {
                    Ok(image_info) => {
                        let image_json = to_string(&image_info).unwrap();
                        let image_value: Value = from_str(&image_json).unwrap();
                        DockerResponse::Ok(image_value)
                    }
                    Err(e) => DockerResponse::Error(format!("{:?}", e)),
                }
            }
            else {
                DockerResponse::Error("Invalid request body".to_string())
            }
        }
        ("DELETE", "/images/*") => {
            let image_id = path.split("/").nth(2).unwrap_or("");
            let mut docker = docker.lock().await;
            match docker.delete_image(image_id).await {
                Ok(_) => {
                    let response_json = to_string(&format!("Image {} deleted", image_id)).unwrap();
                    let response_value: Value = from_str(&response_json).unwrap();
                    DockerResponse::Ok(response_value)
                }
                Err(e) => DockerResponse::Error(format!("{:?}", e)),
            }
        }
        _ => DockerResponse::Error("Not found".to_string()),
    };

    // 构建 HTTP 响应
    let response_str = to_string(&response).unwrap();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        response_str.len(),
        response_str
    )
}

async fn handle_client(mut stream: TcpStream, docker: Arc<Mutex<Docker>>) {
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

    let response = handle_http_request(&request, docker).await;

    if let Err(e) = stream.write_all(response.as_bytes()).await {
        eprintln!("Error writing to stream: {:?}", e);
    }
    else {
        println!("Sent response: {}", response);
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

    let docker = match Docker::new() {
        Ok(docker) => docker,
        Err(e) => {
            eprintln!("Warning: Failed to initialize Docker: {:?}", e);
            eprintln!("Starting in mock mode...");
            // 在实际环境中，这里应该根据平台选择不同的初始化方式
            // 现在我们先使用一个简单的模拟实现
            panic!("Docker initialization failed. Please ensure Docker is installed and running.");
        }
    };

    let docker = Arc::new(Mutex::new(docker));
    println!("Docker daemon listening on {}", cli.host);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let docker_clone = docker.clone();
                tokio::spawn(async move {
                    handle_client(stream, docker_clone).await;
                });
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
