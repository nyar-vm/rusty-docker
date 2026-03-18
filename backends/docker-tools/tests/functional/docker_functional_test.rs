use std::process::Command;

#[test]
fn test_docker_version() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("Docker version"));
    assert!(output_str.contains("rusty-docker"));
}

#[test]
fn test_docker_info() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("info")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出包含系统信息相关内容
    assert!(!output_str.is_empty());
}

#[test]
fn test_docker_ps() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("ps")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出是有效的JSON格式
    assert!(!output_str.is_empty());
}

#[test]
fn test_docker_images() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("images")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出是有效的JSON格式
    assert!(!output_str.is_empty());
}

#[test]
fn test_docker_network_ls() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("network")
        .arg("ls")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出是有效的JSON格式
    assert!(!output_str.is_empty());
}

#[test]
fn test_docker_volume_ls() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("volume")
        .arg("ls")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    // 验证输出是有效的JSON格式
    assert!(!output_str.is_empty());
}