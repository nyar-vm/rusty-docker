# Rusty Docker 项目概述

## 项目简介

Rusty Docker 是一个用 Rust 语言开发的容器管理平台，旨在提供类似 Docker 和 Kubernetes 的功能，同时提供现代化的前端界面。项目的核心目标是为用户提供一个全功能、跨平台的容器管理解决方案，包括容器运行时、编排系统和可视化管理界面。

## 项目架构

### 后端架构

Rusty Docker 的后端由多个模块化组件组成：

- **docker-tools**：核心工具集，对标 Docker CLI、Docker Engine、Docker Compose 和 Kubernetes
  - `docker.rs`：Docker 命令行工具，提供与 Docker CLI 类似的命令
  - `dockerd.rs`：容器运行时，类似 Docker Engine
  - `docker-compose.rs`：多容器应用编排工具，类似 Docker Compose
  - `kubectl.rs`：Kubernetes 命令行工具
  - `kube-apiserver.rs`：Kubernetes API 服务器
  - `kube-controller-manager.rs`：Kubernetes 控制器管理器
  - `kube-scheduler.rs`：Kubernetes 调度器
  - `kube-proxy.rs`：Kubernetes 网络代理
  - `kubeadm.rs`：Kubernetes 集群管理工具

- **docker-container**：容器管理模块，负责容器的创建、启动、停止和删除
- **docker-image**：镜像管理模块，负责镜像的拉取、推送和构建
- **docker-network**：网络管理模块，负责网络的创建、配置和管理
- **docker-storage**：存储管理模块，负责卷的管理和持久化存储
- **docker-config**：配置管理模块，负责系统配置的管理
- **docker-hub**：Docker Hub 客户端，负责与 Docker Hub 的交互
- **docker-types**：类型定义模块，提供系统中使用的各种数据类型
- **rusty-docker**：核心运行时模块，提供容器运行的底层功能

### 前端架构

Rusty Docker 的前端包括两个主要组件：

- **docker-crab-h5**：基于 Vue.js 和 TypeScript 开发的 H5 应用，对标 Portainer，提供可视化的容器管理界面
- **docker-crab-desktop**：基于 Tauri 开发的桌面应用，提供更丰富的本地功能

## 功能特性

### 容器管理
- 创建、启动、停止、删除容器
- 查看容器状态和日志
- 进入容器执行命令
- 容器资源限制管理

### 镜像管理
- 拉取、推送、构建镜像
- 查看镜像详情和历史
- 镜像标签管理
- 镜像导出和导入

### 网络管理
- 创建、删除网络
- 容器网络配置
- 网络类型支持（桥接、主机、覆盖网络等）

### 存储管理
- 卷管理
- 持久化存储
- 存储驱动支持

### 多容器应用编排
- 支持类似 Docker Compose 的配置文件
- 服务编排和管理
- 环境变量和配置管理

### 容器编排
- 集群管理
- 工作负载调度
- 服务发现和负载均衡
- 自动扩缩容

### 可视化管理界面
- 容器资源监控
- 镜像管理界面
- 网络和存储管理界面
- 多集群管理

## 跨平台支持

Rusty Docker 支持在多个操作系统上运行：
- Windows
- Linux
- macOS

## 技术栈

- **后端**：Rust
- **前端**：Vue.js、TypeScript、Tailwind CSS
- **构建工具**：Cargo、Vite
- **包管理**：pnpm

## 项目目标

1. 提供与 Docker CLI 兼容的命令行工具
2. 实现类似 Docker Engine 的容器运行时
3. 支持类似 Docker Compose 的多容器应用编排
4. 提供类似 Kubernetes 的容器编排功能
5. 开发类似 Portainer 的可视化管理界面
6. 支持跨平台运行（Windows、Linux、macOS）

## 非目标

1. 完全兼容 Docker 的所有 API
2. 替代企业级 Kubernetes 解决方案
3. 提供云服务或托管服务
4. 支持所有 Docker 生态系统工具

## 后续计划

- 完善核心功能实现
- 增强跨平台兼容性
- 优化前端用户体验
- 提供更多高级功能
- 完善文档和教程