# Rusty Docker 文档

## 文档导航

### 项目概述
- [项目概述](index.md) - 了解 Rusty Docker 的整体架构和功能

### 核心概念
- [后端模块](concepts/index.md) - 详细介绍后端模块的功能和实现
- [前端模块](concepts/frontend.md) - 详细介绍前端模块的功能和实现

### 架构文档
- [架构文档](maintainer/architecture/index.md) - 详细描述项目的整体架构

### 使用教程
- [使用教程](tutorials/index.md) - 安装、配置和使用指南
- [使用场景](tutorials/use-cases/index.md) - 常见使用场景示例

### 高级功能
- [高级功能](advanced/index.md) - 高级功能和使用场景

### 维护指南
- [维护者文档](maintainer/index.md) - 项目维护指南

## 关于 Rusty Docker

Rusty Docker 是一个用 Rust 语言开发的容器管理平台，旨在提供类似 Docker 和 Kubernetes 的功能，同时提供现代化的前端界面。

### 主要功能

- **容器管理**：创建、启动、停止、删除容器
- **镜像管理**：拉取、推送、构建镜像
- **网络管理**：创建、删除网络，配置容器网络
- **存储管理**：卷管理、持久化存储
- **多容器应用编排**：支持类似 Docker Compose 的配置
- **容器编排**：支持类似 Kubernetes 的集群管理
- **命令行工具**：提供与 Docker CLI 类似的命令
- **可视化管理界面**：提供容器、镜像、网络等资源的管理
- **跨平台支持**：支持 Windows、Linux、macOS

### 技术栈

- **后端**：Rust
- **前端**：Vue.js、TypeScript、Tailwind CSS
- **构建工具**：Cargo、Vite
- **包管理**：pnpm

## 开始使用

1. **安装 Rusty Docker**：
   - 参考 [使用教程](tutorials/index.md) 中的安装指南

2. **基本操作**：
   - 使用命令行工具管理容器、镜像等资源
   - 使用前端界面进行可视化管理

3. **高级功能**：
   - 参考 [高级功能](advanced/index.md) 文档

4. **贡献指南**：
   - 参考 [维护者文档](maintainer/index.md) 中的贡献指南

## 文档更新

本文档将定期更新，以反映项目的最新功能和变化。如果您发现文档中的错误或有改进建议，请提交问题或 Pull Request。

## 联系我们

- **GitHub**：[https://github.com/rusty-docker/rusty-docker](https://github.com/rusty-docker/rusty-docker)
- **文档**：[https://rusty-docker.io/docs](https://rusty-docker.io/docs)
- **社区**：[https://rusty-docker.io/community](https://rusty-docker.io/community)