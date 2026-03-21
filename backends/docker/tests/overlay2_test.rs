use docker2::storage::overlay2::*;
use tempfile::tempdir;

#[test]
fn test_overlay2_driver_creation() {
    let temp_dir = tempdir().unwrap();
    let driver = Overlay2Driver::new(temp_dir.path());
    assert_eq!(driver.storage_root, temp_dir.path());
}

#[test]
fn test_get_container_dir() {
    let temp_dir = tempdir().unwrap();
    let driver = Overlay2Driver::new(temp_dir.path());
    let container_dir = driver.get_container_dir("test-container");
    assert_eq!(container_dir, temp_dir.path().join("containers").join("test-container"));
}