# Rusty-Docker Types: Shared Type Definitions

[![GitHub Stars](https://img.shields.io/github/stars/oovm/opencrab)](https://github.com/oovm/opencrab)

## 🚀 Core Type Definitions for Rusty-Docker

Rusty-Docker Types is the foundational type definition module that powers the Rusty-Docker ecosystem. Designed for consistency, reliability, and type safety, it provides shared data structures and interfaces that ensure seamless integration across all components.

## ✨ Key Features That Ensure Consistency

### 📦 Comprehensive Type Definitions
- **Core Docker Types**: Complete set of Docker-related data structures
- **Kubernetes Types**: Kubernetes resource definitions and interfaces
- **API Models**: Request and response models for API interactions
- **Error Types**: Consistent error handling across components
- **Configuration Types**: Configuration structures for all components

### 🔧 Type Safety
- **Strongly Typed**: Leverage Rust's type system for compile-time safety
- **Deserialization Support**: Built-in serialization/deserialization for JSON and other formats
- **Validation**: Built-in validation for data structures
- **Documentation**: Well-documented types with usage examples
- **Version Compatibility**: Versioned types for backward compatibility

### 🌐 Cross-Component Integration
- **Shared Interfaces**: Consistent interfaces across all components
- **Data Exchange**: Seamless data exchange between components
- **API Consistency**: Consistent API models for external interactions
- **Error Propagation**: Unified error handling across the ecosystem

### 🎯 Developer-Friendly
- **Clean, Intuitive Types**: Easy-to-understand type definitions
- **Comprehensive Documentation**: Detailed documentation for all types
- **Example Usage**: Practical examples for common use cases
- **IDE Support**: Excellent IDE support with type hints and documentation

## 📋 Core Type Categories

### Container Types
- **Container**: Container configuration and state
- **Image**: Image metadata and configuration
- **Volume**: Volume definitions and management
- **Network**: Network configuration and state
- **Exec**: Execution parameters and results

### Kubernetes Types
- **Pod**: Pod definitions and status
- **Deployment**: Deployment configurations
- **Service**: Service definitions and endpoints
- **ConfigMap**: Configuration data
- **Secret**: Sensitive data management

### API Types
- **Request**: API request models
- **Response**: API response models
- **Event**: Event data structures
- **Filter**: Filtering parameters
- **Pagination**: Pagination support

### Error Types
- **DockerError**: Docker-specific error types
- **KubernetesError**: Kubernetes-specific error types
- **NetworkError**: Network-related errors
- **StorageError**: Storage-related errors
- **ValidationError**: Data validation errors

## 🛠️ Quick Start

### Basic Usage

```rust
use docker_types::{Container, Image, Network, Volume};

fn main() {
    // Create a container configuration
    let container = Container {
        id: "container-id".to_string(),
        name: Some("my-container".to_string()),
        image: "nginx:latest".to_string(),
        status: "running".to_string(),
        // ... other fields
        ..Default::default()
    };
    
    // Create an image configuration
    let image = Image {
        id: "image-id".to_string(),
        name: "nginx".to_string(),
        tag: Some("latest".to_string()),
        // ... other fields
        ..Default::default()
    };
    
    println!("Container: {} ({})\n", container.name.unwrap(), container.status);
    println!("Image: {}:{}", image.name, image.tag.unwrap());
}
```

## 🌟 Why Rusty-Docker Types?

### 🚀 Consistent Integration
Ensure consistent data structures and interfaces across all components of the Rusty-Docker ecosystem, reducing integration issues and bugs.

### 🔧 Type Safety
Leverage Rust's powerful type system to catch errors at compile time, reducing runtime errors and improving code quality.

### 📱 Cross-Component Compatibility
Seamless data exchange between components with consistent type definitions, ensuring smooth integration across the entire ecosystem.

### 🎯 Developer-Friendly
Well-documented types with clear usage examples make it easy for developers to understand and use the type system effectively.

### 🔒 Reliable Error Handling
Consistent error types and handling mechanisms across all components, making it easier to manage errors and edge cases.

## 🤝 Contributing

We welcome contributions from the community! Whether you're interested in adding new types, improving existing ones, or enhancing documentation, your help is greatly appreciated.

---

**Providing the foundation for type-safe, consistent integration across the Rusty-Docker ecosystem!** 🦀