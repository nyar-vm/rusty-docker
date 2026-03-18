//! Etcd server main entry point

use clap::Parser;
use docker_etcd::{EtcdServer, Storage};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Address to listen on
    #[arg(short = 'a', long, default_value = "127.0.0.1:2379")]
    listen_client_urls: String,

    /// Peer address to listen on
    #[arg(long, default_value = "127.0.0.1:2380")]
    listen_peer_urls: String,

    /// Initial cluster
    #[arg(long, default_value = "default=http://127.0.0.1:2380")]
    initial_cluster: String,

    /// Initial cluster token
    #[arg(long, default_value = "etcd-cluster-1")]
    initial_cluster_token: String,

    /// Initial advertise peer URLs
    #[arg(long, default_value = "http://127.0.0.1:2380")]
    initial_advertise_peer_urls: String,

    /// Name of this member
    #[arg(long, default_value = "default")]
    name: String,
}

/// API请求
#[derive(Debug, Deserialize)]
enum ApiRequest {
    Get { key: String },
    Put { key: String, value: String, lease: Option<i64> },
    Delete { key: String },
    List {},
}

/// API响应
#[derive(Debug, Serialize)]
enum ApiResponse {
    Ok(serde_json::Value),
    Error(String),
}

async fn handle_client(mut stream: tokio::net::TcpStream, storage: std::sync::Arc<Storage>) {
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

    let response = match serde_json::from_str::<ApiRequest>(&request) {
        Ok(req) => match req {
            ApiRequest::Get { key } => match storage.get(key.as_bytes()) {
                Ok(kv) => {
                    let value = serde_json::json!({
                        "key": base64::encode(&kv.key),
                        "value": base64::encode(&kv.value),
                        "create_revision": kv.create_revision,
                        "mod_revision": kv.mod_revision,
                        "version": kv.version,
                        "lease": kv.lease,
                    });
                    ApiResponse::Ok(value)
                }
                Err(e) => ApiResponse::Error(format!("{:?}", e)),
            },
            ApiRequest::Put { key, value, lease } => {
                match storage.put(key.as_bytes().to_vec(), value.as_bytes().to_vec(), lease.unwrap_or(0)) {
                    Ok(kv) => {
                        let value = serde_json::json!({
                            "key": base64::encode(&kv.key),
                            "value": base64::encode(&kv.value),
                            "create_revision": kv.create_revision,
                            "mod_revision": kv.mod_revision,
                            "version": kv.version,
                            "lease": kv.lease,
                        });
                        ApiResponse::Ok(value)
                    }
                    Err(e) => ApiResponse::Error(format!("{:?}", e)),
                }
            }
            ApiRequest::Delete { key } => match storage.delete(key.as_bytes()) {
                Ok(prev_kv) => {
                    let value = serde_json::json!({
                        "prev_kv": prev_kv.map(|kv| serde_json::json!({
                            "key": base64::encode(&kv.key),
                            "value": base64::encode(&kv.value),
                            "create_revision": kv.create_revision,
                            "mod_revision": kv.mod_revision,
                            "version": kv.version,
                            "lease": kv.lease,
                        })),
                    });
                    ApiResponse::Ok(value)
                }
                Err(e) => ApiResponse::Error(format!("{:?}", e)),
            },
            ApiRequest::List {} => {
                let kvs = storage.list();
                let value = serde_json::json!({
                    "kvs": kvs.into_iter().map(|kv| serde_json::json!({
                        "key": base64::encode(&kv.key),
                        "value": base64::encode(&kv.value),
                        "create_revision": kv.create_revision,
                        "mod_revision": kv.mod_revision,
                        "version": kv.version,
                        "lease": kv.lease,
                    })).collect::<Vec<_>>(),
                });
                ApiResponse::Ok(value)
            }
        },
        Err(e) => ApiResponse::Error(format!("Invalid request: {:?}", e)),
    };

    let response_str = serde_json::to_string(&response).unwrap();
    if let Err(e) = stream.write_all(response_str.as_bytes()).await {
        eprintln!("Error writing to stream: {:?}", e);
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("Starting etcd server");
    println!("Name: {}", cli.name);
    println!("Client URLs: {}", cli.listen_client_urls);
    println!("Peer URLs: {}", cli.listen_peer_urls);
    println!("Initial cluster: {}", cli.initial_cluster);

    let server = EtcdServer::new(cli.listen_client_urls.clone());
    let storage = server.storage();

    // 启动HTTP服务器
    let listener = match TcpListener::bind(&cli.listen_client_urls).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error binding to address: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("Etcd server started on {}", cli.listen_client_urls);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let storage_clone = storage.clone();
                tokio::spawn(async move {
                    handle_client(stream, storage_clone).await;
                });
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
