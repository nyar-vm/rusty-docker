# Docker Service

服务发现和负载均衡功能

本模块提供了服务发现、负载均衡和服务间通信功能，支持多种负载均衡策略，
包括轮询、随机、最少连接和 IP 哈希策略。同时提供了健康检查机制和服务缓存，
提高了服务发现的可靠性和性能。

## 主要功能
- 服务注册和注销
- 服务实例注册和注销
- 服务发现
- 负载均衡
- 健康检查
- 服务间通信
- 服务缓存

## 使用示例
```rust
use docker_network::new_network_manager;
use docker_service::{LoadBalancingStrategy, ServiceConfig, new_service_manager};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() {
    // 创建网络管理器
    let network_manager = new_network_manager();

    // 创建服务管理器
    let service_manager = new_service_manager(network_manager);

    // 创建服务配置
    let config = ServiceConfig {
        name: "my-service".to_string(),
        port: 8080,
        network_id: "default".to_string(),
        load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        health_check_path: Some("/health".to_string()),
        health_check_interval: Some(30),
        labels: Default::default(),
    };

    // 创建服务
    let service = service_manager.create_service(&config).await.unwrap();

    // 添加服务实例
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let instance = service_manager
        .add_service_instance(&service.id, "container-1", "my-container", address)
        .await
        .unwrap();

    // 发现服务
    let discovered_service =
        service_manager.service_discovery.discover_service("my-service").await.unwrap();

    // 负载均衡
    let selected_instance = service_manager.load_balance(&service.id, None).await.unwrap();

    // 服务间通信
    let request = b"Hello from service".to_vec();
    let response = service_manager
        .service_to_service_call(&service.id, "my-service", request)
        .await
        .unwrap();

    println!("Service call response: {:?}", response);
}
```