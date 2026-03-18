# Rusty-Docker Kubernetes Tools: Comprehensive Kubernetes Toolset

[![GitHub Stars](https://img.shields.io/github/stars/oovm/opencrab)](https://github.com/oovm/opencrab)

## 🚀 Comprehensive Kubernetes Toolset

Rusty-Docker Kubernetes Tools is a powerful suite of Kubernetes-related tools built with Rust for maximum performance, reliability, and security. Designed to provide a complete Kubernetes toolchain, it includes everything you need to manage Kubernetes clusters and applications.

## ✨ Key Features That Simplify Kubernetes Management

### 📦 Complete Kubernetes Toolchain
- **kubectl**: Kubernetes command-line tool with enhanced performance
- **kubeadm**: Kubernetes cluster bootstrapping and management
- **helm**: Kubernetes package manager for application deployment
- **kustomize**: Kubernetes configuration management
- **kube-apiserver**: Kubernetes API server with improved performance
- **kube-controller-manager**: Kubernetes controller manager
- **kube-scheduler**: Kubernetes scheduler with enhanced algorithms
- **kube-proxy**: Kubernetes network proxy

### 🔧 Advanced Cluster Management
- **Cluster Creation**: Easy creation of Kubernetes clusters
- **Cluster Upgrades**: Seamless cluster version upgrades
- **Node Management**: Add, remove, and manage cluster nodes
- **Cluster Configuration**: Centralized cluster configuration management
- **Cluster Monitoring**: Built-in cluster health monitoring

### 🎯 Application Deployment
- **Helm Charts**: Deploy applications using Helm charts
- **Kustomize**: Customize Kubernetes configurations
- **Deployment Strategies**: Support for various deployment strategies
- **Rolling Updates**: Zero-downtime application updates
- **Application Scaling**: Automatic and manual scaling of applications

### 🔒 Enhanced Security
- **RBAC Management**: Role-based access control configuration
- **Secret Management**: Secure handling of sensitive information
- **TLS Configuration**: Automatic TLS certificate management
- **Network Policies**: Define and enforce network access rules

### 📱 Cross-Platform Compatibility
- **Multi-Platform Support**: Works seamlessly on Linux, Windows, and macOS
- **Consistent Experience**: Same tooling experience across all platforms
- **Cloud Provider Integration**: Support for major cloud providers

## 📋 Core Tools

### Command-Line Tools
- **kubectl**: Kubernetes command-line tool for managing resources
- **kubeadm**: Tool for bootstrapping Kubernetes clusters
- **helm**: Package manager for Kubernetes applications
- **kustomize**: Tool for customizing Kubernetes configurations

### Cluster Components
- **kube-apiserver**: Kubernetes API server
- **kube-controller-manager**: Runs controllers that manage cluster resources
- **kube-scheduler**: Schedules pods to nodes based on resource requirements
- **kube-proxy**: Network proxy that runs on each node

### Additional Tools
- **kube-mcp**: Multi-cluster management platform
- **kube-bench**: Kubernetes security benchmark tool
- **kube-ops-view**: Kubernetes operations view dashboard

## 🛠️ Quick Start

### Basic Usage

```bash
# Create a Kubernetes cluster
cargo run --bin kubeadm init

# Join a node to the cluster
cargo run --bin kubeadm join <join-token>

# Deploy an application using Helm
cargo run --bin helm install my-app bitnami/nginx

# Manage Kubernetes resources
cargo run --bin kubectl get pods
cargo run --bin kubectl create deployment nginx --image=nginx
cargo run --bin kubectl expose deployment nginx --port=80 --type=NodePort
```

### Helm Usage

```bash
# Add a Helm repository
cargo run --bin helm repo add bitnami https://charts.bitnami.com/bitnami

# Update Helm repositories
cargo run --bin helm repo update

# List Helm repositories
cargo run --bin helm repo list

# Install a Helm chart
cargo run --bin helm install my-release bitnami/nginx

# Upgrade a Helm release
cargo run --bin helm upgrade my-release bitnami/nginx

# Uninstall a Helm release
cargo run --bin helm uninstall my-release

# List Helm releases
cargo run --bin helm list

# Check Helm release status
cargo run --bin helm status my-release

# Pull a Helm chart
cargo run --bin helm pull bitnami/nginx

# Search for Helm charts
cargo run --bin helm search repo nginx

# Render Helm chart templates
cargo run --bin helm template my-release bitnami/nginx
```

### Kustomize Usage

```bash
# Create a new kustomization directory
cargo run --bin kustomize create my-app

# Build Kubernetes resources
cargo run --bin kustomize build my-app

# Edit kustomization configuration
cargo run --bin kustomize edit add resource deployment.yaml
cargo run --bin kustomize edit set namespace default
cargo run --bin kustomize edit add patch patch.yaml
cargo run --bin kustomize edit add label app=my-app
cargo run --bin kustomize edit add annotation description="My application"

# Validate kustomization configuration
cargo run --bin kustomize validate my-app

# View kustomization configuration
cargo run --bin kustomize config view my-app

# View diff between base and overlay
cargo run --bin kustomize config diff base overlay
```

## 🌟 Why Rusty-Docker Kubernetes Tools?

### 🚀 High Performance
Built with Rust, these tools deliver superior performance with faster execution times, lower memory usage, and better resource utilization compared to traditional Kubernetes tools.

### 🔧 Comprehensive Toolset
A complete Kubernetes toolchain that covers everything from cluster creation to application deployment, eliminating the need for multiple tools from different sources.

### 📱 Cross-Platform Freedom
Run the same tools across Linux, Windows, and macOS, ensuring a consistent experience regardless of your environment.

### 🎯 Enhanced Functionality
Additional features and improvements over standard Kubernetes tools, including better error handling, improved performance, and enhanced security.

### 🔒 Reliable and Secure
Rust's memory safety guarantees and our security-focused design provide a more secure environment than traditional tools, protecting against common vulnerabilities.

## 🤝 Contributing

We welcome contributions from the community! Whether you're interested in adding new features, fixing bugs, or improving documentation, your help is greatly appreciated.

---

**Powerful, fast, and comprehensive Kubernetes tools for the modern cluster operator!** 🦀