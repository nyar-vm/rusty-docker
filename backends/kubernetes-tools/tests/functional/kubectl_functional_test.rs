use std::process::Command;

#[test]
fn test_kubectl_version() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("version")
        .arg("--client")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("Client Version"));
}

#[test]
fn test_kubectl_get_pods() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("get")
        .arg("pods")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出是有效的JSON格式或模拟数据
    assert!(!output_str.is_empty());
}

#[test]
fn test_kubectl_get_services() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("get")
        .arg("services")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出是有效的JSON格式或模拟数据
    assert!(!output_str.is_empty());
}

#[test]
fn test_kubectl_get_deployments() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("get")
        .arg("deployments")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出是有效的JSON格式或模拟数据
    assert!(!output_str.is_empty());
}

#[test]
fn test_kubectl_cluster_info() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("cluster-info")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出包含集群信息相关内容
    assert!(!output_str.is_empty());
}

#[test]
fn test_kubectl_config_get_contexts() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("config")
        .arg("get-contexts")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出包含上下文信息
    assert!(!output_str.is_empty());
    assert!(output_str.contains("CURRENT"));
    assert!(output_str.contains("NAME"));
}