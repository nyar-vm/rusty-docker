use std::process::Command;

#[test]
fn test_docker_ps_with_invalid_option() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("ps")
        .arg("--invalid-option")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误
    assert!(!output.status.success());
}

#[test]
fn test_docker_images_with_empty_args() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("images")
        .output()
        .expect("Failed to execute command");

    // 应该成功执行，即使没有参数
    assert!(output.status.success());
}

#[test]
fn test_docker_network_ls_with_empty_args() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("network")
        .arg("ls")
        .output()
        .expect("Failed to execute command");

    // 应该成功执行，即使没有参数
    assert!(output.status.success());
}

#[test]
fn test_docker_volume_ls_with_empty_args() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("volume")
        .arg("ls")
        .output()
        .expect("Failed to execute command");

    // 应该成功执行，即使没有参数
    assert!(output.status.success());
}

#[test]
fn test_docker_inspect_with_nonexistent_container() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("inspect")
        .arg("nonexistent-container-12345")
        .output()
        .expect("Failed to execute command");

    // 应该成功执行，但显示容器不存在
    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("Container not found"));
}

#[test]
fn test_docker_image_inspect_with_nonexistent_image() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("image")
        .arg("inspect")
        .arg("nonexistent-image:latest")
        .output()
        .expect("Failed to execute command");

    // 应该成功执行，但显示镜像不存在
    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("Image not found"));
}