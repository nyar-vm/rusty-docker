#![warn(missing_docs)]

use super::{NetworkConfig, NetworkManager, new_network_manager};

#[test]
fntest_windows_network_manager() {
    // 创建网络管理器
    let mut manager = new_network_manager();
    
    // 测试列出网络
    let networks = manager.list_networks().unwrap();
    assert!(!networks.is_empty());
    
    // 测试创建网络
    let config = NetworkConfig {
        name: "test-network".to_string(),
        driver: "bridge".to_string(),
        ipam: None,
        options: None,
    };
    
    let network = manager.create_network(&config).unwrap();
    assert_eq!(network.name, "test-network");
    assert_eq!(network.driver, "bridge");
    
    // 测试检查网络
    let inspected = manager.inspect_network(&network.id).unwrap();
    assert_eq!(inspected.id, network.id);
    
    // 测试删除网络
    manager.remove_network(&network.id).unwrap();
    
    // 验证网络已删除
    let result = manager.inspect_network(&network.id);
    assert!(result.is_err());
}