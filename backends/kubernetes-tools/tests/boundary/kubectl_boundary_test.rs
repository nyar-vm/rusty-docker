use std::process::Command;

#[test]
fn test_kubectl_get_with_invalid_resource() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("get")
        .arg("invalid-resource-type")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为资源类型无效
    assert!(!output.status.success());
}

#[test]
fn test_kubectl_get_pods_with_all_namespaces() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("get")
        .arg("pods")
        .arg("--all-namespaces")
        .output()
        .expect("Failed to execute command");

    // 应该成功执行，即使使用了--all-namespaces选项
    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(!output_str.is_empty());
}

#[test]
fn test_kubectl_describe_with_nonexistent_pod() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("describe")
        .arg("pod")
        .arg("nonexistent-pod-12345")
        .output()
        .expect("Failed to execute command");

    // 应该成功执行，但显示pod不存在
    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(!output_str.is_empty());
}

#[test]
fn test_kubectl_config_use_context_with_nonexistent_context() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("config")
        .arg("use-context")
        .arg("nonexistent-context")
        .output()
        .expect("Failed to execute command");

    // 应该成功执行，即使上下文不存在
    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("Switched to context"));
}

#[test]
fn test_kubectl_apply_with_nonexistent_file() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("apply")
        .arg("--file")
        .arg("nonexistent-file.yaml")
        .output()
        .expect("Failed to execute command");

    // 应该返回错误，因为文件不存在
    assert!(!output.status.success());
}

#[test]
fn test_kubectl_delete_with_nonexistent_resource() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kubectl")
        .arg("delete")
        .arg("pod")
        .arg("nonexistent-pod-12345")
        .output()
        .expect("Failed to execute command");

    // 应该成功执行，即使资源不存在
    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(!output_str.is_empty());
}