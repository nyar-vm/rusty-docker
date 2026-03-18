use std::process::Command;

#[test]
fn test_kubectl_get_without_resource() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("get")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少资源类型参数
    assert!(!output.status.success());
}

#[test]
fn test_kubectl_describe_without_resource() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("describe")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少资源类型参数
    assert!(!output.status.success());
}

#[test]
fn test_kubectl_delete_without_resource() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("delete")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少资源类型参数
    assert!(!output.status.success());
}

#[test]
fn test_kubectl_apply_without_file() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("apply")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少文件参数
    assert!(!output.status.success());
}

#[test]
fn test_kubectl_create_without_file() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("create")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少文件参数
    assert!(!output.status.success());
}

#[test]
fn test_kubectl_edit_without_resource() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("edit")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少资源类型参数
    assert!(!output.status.success());
}

#[test]
fn test_kubectl_patch_without_resource() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("patch")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少资源类型参数
    assert!(!output.status.success());
}

#[test]
fn test_kubectl_logs_without_pod() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("logs")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为缺少pod参数
    assert!(!output.status.success());
}