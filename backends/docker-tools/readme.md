# Rusty-Docker Tools: High-Performance Container Tools Suite

[![GitHub Stars](https://img.shields.io/github/stars/oovm/opencrab)](https://github.com/oovm/opencrab)

## 🚀 Comprehensive Container Tools Suite

Rusty-Docker Tools is a powerful suite of command-line tools that provides a drop-in replacement for Docker CLI, Docker Compose, and Kubernetes kubectl, all built with Rust for maximum performance, reliability, and security.

## ✨ Key Features That Set Us Apart

### 📦 Docker CLI Compatibility
- **Drop-in Replacement**: Full compatibility with existing Docker CLI commands
- **Blazing Fast Performance**: Rust-powered for faster execution and lower resource usage
- **Cross-Platform Support**: Works seamlessly on Linux, macOS, and Windows
- **Modern Feature Set**: Includes the latest Docker features and improvements
- **Enhanced Error Handling**: More informative error messages and better error recovery

### 🐳 Docker Compose
- **Multi-Container Orchestration**: Define and run complex multi-container applications
- **YAML Configuration**: Use familiar docker-compose.yml files with extended capabilities
- **Service Management**: Start, stop, and manage services with ease
- **Automatic Networking**: Seamless network creation and management
- **Volume Management**: Persistent storage for stateful applications

### ☸️ Kubernetes Integration
- **Kubectl Compatible**: Full support for Kubernetes resource management
- **Seamless Workflow**: Switch between Docker and Kubernetes contexts effortlessly
- **Resource Management**: Create, update, and delete Kubernetes resources with ease
- **Cluster Operations**: Manage clusters, nodes, and services from a single toolset
- **Configuration Management**: Simplified Kubernetes configuration

### 🔧 Advanced Tools
- **Docker Buildx**: Advanced image building with multi-architecture support
- **Docker Swarm**: Orchestrate containers as a swarm for high availability
- **Containerd**: Lightweight container runtime with enhanced performance
- **Runc**: Low-level container execution with improved security
- **Credential Helpers**: Secure credential management for registries

## 📋 Available Tools

### Core Tools
- **docker**: Main Docker CLI with all standard commands and enhanced performance
- **docker-compose**: Multi-container application orchestration with extended features
- **kubectl**: Kubernetes command-line tool with full compatibility
- **docker-buildx**: Advanced image building with multi-architecture support
- **docker-swarm**: Swarm orchestration for high-availability deployments

### Supporting Tools
- **containerd**: High-performance container runtime
- **runc**: Low-level container executor with enhanced security
- **docker-credential-helpers**: Secure credential management for registries
- **docker-mcp**: Multi-container platform management
- **podman**: Alternative container runtime with rootless support

## 🛠️ Quick Start

### Basic Usage

```bash
# List running containers
cargo run --bin docker ps

# Run a container
cargo run --bin docker run -d --name nginx -p 8080:80 nginx:latest

# Build an image
cargo run --bin docker build -t my-app .

# Use Docker Compose
cargo run --bin docker-compose up -d

# Manage Kubernetes
cargo run --bin kubectl get pods
```

## 🌟 Why Rusty-Docker Tools?

### 🚀 Unmatched Performance
Built with Rust, Rusty-Docker Tools delivers superior performance with faster execution times, lower memory usage, and better resource utilization compared to traditional container tools.

### 🔒 Enhanced Security
Rust's memory safety guarantees and our security-focused design provide a more secure environment than traditional tools, protecting against common vulnerabilities.

### 📱 Cross-Platform Freedom
Run the same commands across Linux, macOS, and Windows without modification, ensuring a consistent experience everywhere.

### 🎯 Complete Compatibility
Drop-in replacement for existing Docker and Kubernetes tools, allowing you to switch without changing your workflow or scripts.

### 🔧 Extensible Architecture
Modular design allows for easy extension and customization to meet your specific needs, whether you're a developer, DevOps engineer, or system administrator.

## 🤝 Contributing

We welcome contributions from the community! Whether you're interested in adding new features, fixing bugs, or improving documentation, your help is greatly appreciated.

---

**Powerful, fast, and compatible container tools for the modern developer!** 🦀