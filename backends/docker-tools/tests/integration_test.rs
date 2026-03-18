use std::process::Command;

#[test]
fn test_docker_version() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("Docker version"));
}

#[test]
fn test_docker_compose_version() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("docker-compose")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("docker-compose version"));
}

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
