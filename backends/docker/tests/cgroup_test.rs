use docker2::cgroup::*;
use tempfile::tempdir;

#[test]
#[cfg(target_os = "linux")]
fn test_cgroup_manager_creation() {
    let manager = CgroupManager::new();
    assert!(manager.is_ok());
}

#[test]
fn test_cgroup_version_enum() {
    let v1 = CgroupVersion::V1;
    let v2 = CgroupVersion::V2;
    assert_ne!(v1, v2);
}