use std::process::Command;
use std::str;

#[test]
fn test_helm_version() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("helm")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("helm"));
}

#[test]
fn test_helm_repo_list() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("helm")
        .arg("repo")
        .arg("list")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("NAME"));
    assert!(stdout.contains("URL"));
}

#[test]
fn test_helm_install() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("helm")
        .arg("install")
        .arg("test-release")
        .arg("nginx")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Installing Helm chart"));
    assert!(stdout.contains("Release name: test-release"));
    assert!(stdout.contains("Chart: nginx"));
}

#[test]
fn test_helm_upgrade() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("helm")
        .arg("upgrade")
        .arg("test-release")
        .arg("nginx")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Upgrading Helm release"));
    assert!(stdout.contains("Release name: test-release"));
    assert!(stdout.contains("Chart: nginx"));
}

#[test]
fn test_helm_uninstall() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("helm")
        .arg("uninstall")
        .arg("test-release")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("Uninstalling Helm release"));
    assert!(stdout.contains("Release name: test-release"));
    assert!(stdout.contains("Release 'test-release' uninstalled successfully"));
}
