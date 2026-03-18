use std::process::Command;

#[test]
fn test_docker_run_without_image() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("run")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少镜像参数
    assert!(!output.status.success());
}

#[test]
fn test_docker_stop_without_container() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("stop")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少容器参数
    assert!(!output.status.success());
}

#[test]
fn test_docker_rm_without_container() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("rm")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少容器参数
    assert!(!output.status.success());
}

#[test]
fn test_docker_start_without_container() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("start")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少容器参数
    assert!(!output.status.success());
}

#[test]
fn test_docker_pause_without_container() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("pause")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少容器参数
    assert!(!output.status.success());
}

#[test]
fn test_docker_unpause_without_container() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("unpause")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少容器参数
    assert!(!output.status.success());
}

#[test]
fn test_docker_restart_without_container() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("restart")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少容器参数
    assert!(!output.status.success());
}

#[test]
fn test_docker_inspect_without_container() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("inspect")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少容器参数
    assert!(!output.status.success());
}