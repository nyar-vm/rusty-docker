use std::fs;
use std::path::Path;
use std::process::Command;
use std::str;

#[test]
fn test_kustomize_version() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kustomize")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("kustomize"));
}

#[test]
fn test_kustomize_create() {
    // 创建测试目录
    let test_dir = "test-kustomize";
    if Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir).expect("Failed to remove test directory");
    }

    // 测试 create 命令
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kustomize")
        .arg("create")
        .arg(test_dir)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Creating kustomization directory"));
    assert!(stdout.contains(format!("Path: {}", test_dir)));

    // 检查 kustomization.yaml 文件是否创建
    let kustomization_path = format!("{}/kustomization.yaml", test_dir);
    assert!(Path::new(&kustomization_path).exists());

    // 清理测试目录
    fs::remove_dir_all(test_dir).expect("Failed to remove test directory");
}

#[test]
fn test_kustomize_build() {
    // 创建测试目录和 kustomization.yaml 文件
    let test_dir = "test-kustomize-build";
    if Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir).expect("Failed to remove test directory");
    }
    fs::create_dir_all(test_dir).expect("Failed to create test directory");

    // 创建 kustomization.yaml 文件
    let kustomization_content = "apiVersion: kustomize.config.k8s.io/v1beta1\nkind: Kustomization\nresources:\n- service.yaml\n- deployment.yaml\n";
    fs::write(
        format!("{}/kustomization.yaml", test_dir),
        kustomization_content,
    )
    .expect("Failed to write kustomization.yaml");

    // 测试 build 命令
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kustomize")
        .arg("build")
        .arg(test_dir)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Building kustomization"));
    assert!(stdout.contains("apiVersion: v1"));
    assert!(stdout.contains("kind: ConfigMap"));
    assert!(stdout.contains("apiVersion: apps/v1"));
    assert!(stdout.contains("kind: Deployment"));

    // 清理测试目录
    fs::remove_dir_all(test_dir).expect("Failed to remove test directory");
}

#[test]
fn test_kustomize_validate() {
    // 创建测试目录和 kustomization.yaml 文件
    let test_dir = "test-kustomize-validate";
    if Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir).expect("Failed to remove test directory");
    }
    fs::create_dir_all(test_dir).expect("Failed to create test directory");

    // 创建 kustomization.yaml 文件
    let kustomization_content = "apiVersion: kustomize.config.k8s.io/v1beta1\nkind: Kustomization\nresources:\n- service.yaml\n- deployment.yaml\n";
    fs::write(
        format!("{}/kustomization.yaml", test_dir),
        kustomization_content,
    )
    .expect("Failed to write kustomization.yaml");

    // 测试 validate 命令
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kustomize")
        .arg("validate")
        .arg(test_dir)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Validating kustomization"));
    assert!(stdout.contains("Validation successful"));

    // 清理测试目录
    fs::remove_dir_all(test_dir).expect("Failed to remove test directory");
}

#[test]
fn test_kustomize_config_view() {
    // 创建测试目录和 kustomization.yaml 文件
    let test_dir = "test-kustomize-config";
    if Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir).expect("Failed to remove test directory");
    }
    fs::create_dir_all(test_dir).expect("Failed to create test directory");

    // 创建 kustomization.yaml 文件
    let kustomization_content = "apiVersion: kustomize.config.k8s.io/v1beta1\nkind: Kustomization\nresources:\n- service.yaml\n- deployment.yaml\n";
    fs::write(
        format!("{}/kustomization.yaml", test_dir),
        kustomization_content,
    )
    .expect("Failed to write kustomization.yaml");

    // 测试 config view 命令
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("kustomize")
        .arg("config")
        .arg("view")
        .arg(test_dir)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Viewing config"));
    assert!(stdout.contains("apiVersion: kustomize.config.k8s.io/v1beta1"));
    assert!(stdout.contains("kind: Kustomization"));
    assert!(stdout.contains("resources:"));
    assert!(stdout.contains("- service.yaml"));
    assert!(stdout.contains("- deployment.yaml"));

    // 清理测试目录
    fs::remove_dir_all(test_dir).expect("Failed to remove test directory");
}
