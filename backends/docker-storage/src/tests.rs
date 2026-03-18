use super::*;

#[tokio::test]
async fn test_storage_manager() {
    let manager = StorageManager::new().unwrap();
    
    // 测试基础路径
    let base_path = StorageManager::get_base_path().unwrap();
    println!("Base path: {:?}", base_path);
    
    // 测试目录创建
    manager.ensure_directories().await.unwrap();
    
    // 测试路径生成
    let containers_path = manager.containers_path().unwrap();
    let images_path = manager.images_path().unwrap();
    let volumes_path = manager.volumes_path().unwrap();
    let tmp_path = manager.tmp_path().unwrap();
    
    println!("Containers path: {:?}", containers_path);
    println!("Images path: {:?}", images_path);
    println!("Volumes path: {:?}", volumes_path);
    println!("Tmp path: {:?}", tmp_path);
    
    // 测试文件操作
    let test_file = tmp_path.join("test.txt");
    let test_content = b"Hello, DockerCrab!";
    
    manager.create_file(&test_file, test_content).await.unwrap();
    let read_content = manager.read_file(&test_file).await.unwrap();
    assert_eq!(read_content, test_content);
    
    manager.remove_file(&test_file).await.unwrap();
    assert!(!test_file.exists());
    
    println!("All tests passed!");
}