# Rusty-Docker Registry: Advanced Registry Integration

[![GitHub Stars](https://img.shields.io/github/stars/oovm/opencrab)](https://github.com/oovm/opencrab)

## 🚀 Powerful Registry Integration for Containers

Rusty-Docker Registry is the advanced registry integration component that powers the Rusty-Docker ecosystem. Designed for reliability, performance, and flexibility, it provides comprehensive interactions with Docker registries, including Docker Hub, private registries, and cloud-based registries.

## ✨ Key Features That Simplify Registry Operations

### 📦 Comprehensive Registry Operations
- **Image Pulling**: Efficiently pull images from registries with parallel downloading
- **Image Pushing**: Push images to registries with progress tracking
- **Image Tagging**: Manage image tags across registries
- **Registry Search**: Search for images across multiple registries
- **Manifest Management**: Create and manage image manifests

### 🔧 Advanced Registry Management
- **Multi-Registry Support**: Work with Docker Hub, private registries, and cloud registries
- **Authentication**: Securely authenticate with registries using various methods
- **Rate Limit Handling**: Smart retry mechanisms for registry rate limits
- **Proxy Support**: Configure proxies for registry access
- **Registry Caching**: Cache registry responses for improved performance

### 🌐 Registry API Integration
- **Docker Registry API**: Full support for the Docker Registry HTTP API
- **OCI Registry Support**: Compatibility with OCI-compliant registries
- **GraphQL API**: Advanced query capabilities for registry data
- **Webhook Support**: Receive notifications for registry events

### 🔒 Secure Registry Interactions
- **TLS Support**: Encrypted communication with registries
- **Credential Management**: Securely store and manage registry credentials
- **Signature Verification**: Verify image signatures for authenticity
- **Content Trust**: Ensure image integrity with content trust

### 🎯 Developer-Friendly API
- **Clean, Intuitive API**: Easy-to-use interface for registry operations
- **Async Support**: Non-blocking registry operations with async/await
- **Error Handling**: Comprehensive error handling and reporting
- **Documentation**: Well-documented API with examples

## 📋 Core Components

### Registry Client
- **API Client**: Full-featured client for registry APIs
- **Authentication Handler**: Manage registry authentication
- **Rate Limit Manager**: Handle registry rate limits intelligently
- **Retry Mechanism**: Automatic retries for transient errors

### Image Downloader
- **Parallel Downloading**: Download image layers in parallel
- **Resume Support**: Resume interrupted downloads
- **Checksum Verification**: Verify layer integrity
- **Progress Tracking**: Real-time download progress

### Manifest Manager
- **Manifest Creation**: Create image manifests
- **Manifest Inspection**: Examine manifest details
- **Multi-Architecture Support**: Handle multi-architecture manifests
- **Manifest Push/Pull**: Push and pull manifests to/from registries

### Credential Store
- **Secure Storage**: Encrypted storage for registry credentials
- **Credential Rotation**: Automatic credential rotation
- **Multi-Platform Support**: Works across all supported platforms
- **Integration with System Keychains**: Use system keychains for credential storage

## 🛠️ Quick Start

### Basic Usage

```rust
use docker_registry::RegistryClient;

#[tokio::main]
async fn main() {
    // Create a registry client
    let client = RegistryClient::new("registry-1.docker.io").unwrap();
    
    // Authenticate (optional)
    client.authenticate("username", "password").await.unwrap();
    
    // Pull an image
    client.pull("nginx:latest").await.unwrap();
    
    // Push an image
    client.push("my-app:latest").await.unwrap();
    
    // Search for images
    let results = client.search("ubuntu").await.unwrap();
    for result in results {
        println!("Image: {}", result.name);
    }
}
```

## 🌟 Why Rusty-Docker Registry?

### 🚀 High Performance
Built with Rust, Rusty-Docker Registry delivers superior performance with faster image operations and lower latency compared to traditional registry clients.

### 🔧 Advanced Features
Comprehensive registry integration capabilities that go beyond basic operations, including multi-registry support and advanced authentication methods.

### 📱 Cross-Platform Compatibility
Work with registries consistently across Linux, Windows, and macOS, eliminating platform-specific challenges.

### 🎯 Developer-Friendly
A clean, well-documented API makes it easy to integrate registry operations into your applications and workflows.

### 🔒 Enhanced Security
Built-in security features to help you maintain secure registry interactions and protect your container images.

## 🤝 Contributing

We welcome contributions from the community! Whether you're interested in adding new features, fixing bugs, or improving documentation, your help is greatly appreciated.

---

**Simplifying container registry interactions with advanced features and cross-platform compatibility!** 🦀