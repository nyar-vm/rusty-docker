use docker2::namespace::*;
use tempfile::tempdir;

#[test]
fn test_namespace_type_as_str() {
    assert_eq!(NamespaceType::Pid.as_str(), "pid");
    assert_eq!(NamespaceType::Network.as_str(), "net");
    assert_eq!(NamespaceType::Mount.as_str(), "mnt");
    assert_eq!(NamespaceType::Uts.as_str(), "uts");
    assert_eq!(NamespaceType::Ipc.as_str(), "ipc");
    assert_eq!(NamespaceType::User.as_str(), "user");
}

#[test]
fn test_namespace_config_all() {
    let config = NamespaceConfig::all();
    assert!(config.pid);
    assert!(config.network);
    assert!(config.mount);
    assert!(config.uts);
    assert!(config.ipc);
    assert!(!config.user);
}

#[test]
fn test_id_mapping() {
    let mapping = IdMapping::new(0, 1000, 65536);
    assert_eq!(mapping.container_id, 0);
    assert_eq!(mapping.host_id, 1000);
    assert_eq!(mapping.size, 65536);
    assert_eq!(mapping.to_string(), "0 1000 65536");
}

#[test]
fn test_namespace_manager_creation() {
    let temp_dir = tempdir().unwrap();
    let manager = NamespaceManager::new(temp_dir.path()).unwrap();
    assert_eq!(manager.base_path(), temp_dir.path());
}

#[test]
fn test_get_namespace_path() {
    let temp_dir = tempdir().unwrap();
    let manager = NamespaceManager::new(temp_dir.path()).unwrap();
    let path = manager.get_namespace_path("test-container", NamespaceType::Pid);
    assert_eq!(path, temp_dir.path().join("test-container-pid"));
}