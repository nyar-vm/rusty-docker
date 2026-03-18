# Rusty-Docker Storage: Advanced Storage Management

[![GitHub Stars](https://img.shields.io/github/stars/oovm/opencrab)](https://github.com/oovm/opencrab)

## 🚀 Powerful Storage Management for Containers

Rusty-Docker Storage is the advanced storage management component that powers the Rusty-Docker ecosystem. Designed for reliability, performance, and cross-platform compatibility, it provides comprehensive storage operations for containerized applications.

## ✨ Key Features That Simplify Storage Management

### 📦 Comprehensive Storage Operations
- **Directory Management**: Create and manage container, image, volume, and temporary directories
- **File Operations**: Read, write, create, and delete files with ease
- **Path Resolution**: Smart path resolution for cross-platform compatibility
- **Storage Cleanup**: Remove unused storage to save space
- **Backup/Restore**: Backup and restore container data

### 🔧 Advanced Storage Management
- **Volume Management**: Create and manage persistent volumes
- **Layer Management**: Efficiently manage image layers and their storage
- **Content Addressable Storage**: Use content hashing for efficient storage
- **Storage Optimization**: Automatic storage optimization and deduplication
- **Quota Management**: Set storage quotas for containers and volumes

### 🌐 Cross-Platform Compatibility
- **Multi-Platform Support**: Works seamlessly on Linux, Windows, and macOS
- **Platform-Specific Paths**: Automatic detection and use of platform-appropriate paths
- **Consistent API**: Same storage operations across all platforms
- **Path Normalization**: Automatic path normalization for cross-platform compatibility

### 🔒 Secure Storage
- **Permission Management**: Set and manage file permissions
- **Access Control**: Fine-grained access control for storage resources
- **Encrypted Storage**: Optional encryption for sensitive data
- **Audit Logging**: Track storage operations for compliance and troubleshooting

### 🎯 Developer-Friendly API
- **Clean, Intuitive API**: Easy-to-use interface for storage operations
- **Async Support**: Non-blocking storage operations with async/await
- **Error Handling**: Comprehensive error handling and reporting
- **Documentation**: Well-documented API with examples

## 📋 Core Components

### Storage Manager
- **Path Management**: Manage storage paths across platforms
- **Directory Structure**: Ensure proper directory structure exists
- **File Operations**: High-level file operations
- **Storage Cleanup**: Remove unused storage

### Volume Manager
- **Volume Creation**: Create and manage persistent volumes
- **Volume Mounting**: Mount volumes to containers
- **Volume Backup**: Backup and restore volume data
- **Volume Encryption**: Optional volume encryption

### Layer Store
- **Layer Management**: Manage image layers efficiently
- **Layer Caching**: Cache layers for faster access
- **Layer Deduplication**: Deduplicate layers to save space
- **Layer Garbage Collection**: Clean up unused layers

### Content Addressable Storage
- **Content Hashing**: Hash content for efficient storage
- **Deduplication**: Automatically deduplicate content
- **Data Integrity**: Ensure data integrity with checksums
- **Efficient Storage**: Optimize storage usage

## 🛠️ Quick Start

### Basic Usage

```rust
use docker_storage::{StorageManager, StorageResult};

#[tokio::main]
async fn main() -> StorageResult<()> {
    // Create storage manager
    let manager = StorageManager::new()?;
    
    // Ensure directory structure exists
    manager.ensure_directories().await?;
    
    // Get various paths
    let containers_path = manager.containers_path()?;
    let images_path = manager.images_path()?;
    let volumes_path = manager.volumes_path()?;
    
    // File operation example
    let test_file = manager.tmp_path()?.join("test.txt");
    manager.create_file(&test_file, b"Hello, Docker Storage!").await?;
    
    let content = manager.read_file(&test_file).await?;
    println!("File content: {}", String::from_utf8_lossy(&content));
    
    manager.remove_file(&test_file).await?;
    
    Ok(())
}
```

## 🌟 Why Rusty-Docker Storage?

### 🚀 High Performance
Built with Rust, Rusty-Docker Storage delivers superior performance with faster storage operations and lower latency compared to traditional storage management libraries.

### 🔧 Advanced Features
Comprehensive storage management capabilities that go beyond basic operations, including volume management and content addressable storage.

### 📱 Cross-Platform Freedom
Manage storage consistently across Linux, Windows, and macOS, eliminating platform-specific storage challenges.

### 🎯 Developer-Friendly
A clean, well-documented API makes it easy to integrate storage management into your applications and workflows.

### 🔒 Enhanced Security
Built-in security features to help you maintain secure storage configurations and protect your container data.

## 🤝 Contributing

We welcome contributions from the community! Whether you're interested in adding new features, fixing bugs, or improving documentation, your help is greatly appreciated.

---

**Simplifying container storage management with advanced features and cross-platform compatibility!** 🦀