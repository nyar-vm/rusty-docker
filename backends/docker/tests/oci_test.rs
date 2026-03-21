use docker2::oci::*;
use docker_types::ContainerConfig;

#[test]
fn test_container_config_to_oci() {
    let container_config = ContainerConfig {
        name: "test-container".to_string(),
        image: "ubuntu:latest".to_string(),
        command: vec!["/bin/sh".to_string()],
        environment: std::collections::HashMap::new(),
        ports: std::collections::HashMap::new(),
        volumes: vec![],
        resources: docker_types::ResourceLimits { cpu_limit: 1.0, memory_limit: 512, storage_limit: 10, network_limit: 10 },
        network: docker_types::NetworkConfig {
            network_name: "default".to_string(),
            static_ip: None,
            hostname: None,
            aliases: None,
            network_mode: None,
            enable_ipv6: false,
        },
        restart_policy: None,
        healthcheck: None,
        deploy: None,
    };

    let oci_config = container_config_to_oci(&container_config, "/path/to/rootfs");

    assert_eq!(oci_config.version, "1.0.2-dev");
    assert_eq!(oci_config.process.args, vec!["/bin/sh"]);
    assert_eq!(oci_config.root.path, "/path/to/rootfs");
    assert_eq!(oci_config.mounts.len(), 3);
}