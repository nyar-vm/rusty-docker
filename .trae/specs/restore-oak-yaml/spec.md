# 恢复 oak-yaml 使用 - 产品需求文档

## 概述
- **摘要**：恢复对 oak-yaml 库的使用，为其添加缺失的功能，并使用 serde 进行配置反序列化
- **目的**：解决之前移除 YAML 依赖导致的功能回退问题，确保配置文件能够正确解析和处理
- **目标用户**：使用 docker-tools 和 kubernetes-tools 的开发者和用户

## 目标
- 恢复 oak-yaml 库的使用，替换当前的 mock 实现
- 为 oak-yaml 添加缺失的功能，确保完整的 YAML 解析能力
- 使用 serde 进行配置反序列化，提高代码的可维护性和类型安全性
- 确保所有相关文件能够正确编译和运行

## 非目标（范围外）
- 不修改其他核心功能逻辑
- 不添加新的功能特性，只恢复和完善现有的 YAML 处理能力
- 不修改项目的整体架构

## 背景与上下文
- 之前为了修复 cargo check 错误，临时移除了 oak-yaml 和 yaml_rust 依赖，使用了 mock 实现
- 用户强烈要求恢复 oak-yaml 的使用，并要求添加缺失的功能
- 需要确保所有 YAML 配置文件能够正确解析，包括 kubeconfig、kustomization.yaml 和 docker-compose.yml

## 功能需求
- **FR-1**：恢复 oak-yaml 库的依赖和使用
- **FR-2**：为 oak-yaml 添加缺失的 YAML Value 功能
- **FR-3**：使用 serde 进行配置反序列化
- **FR-4**：确保 kubectl.rs、kustomize.rs、helm.rs 和 docker-compose.rs 能够正确解析 YAML 文件

## 非功能需求
- **NFR-1**：代码编译无错误，通过 cargo check
- **NFR-2**：代码风格一致，符合项目的编码规范
- **NFR-3**：性能良好，YAML 解析速度快
- **NFR-4**：错误处理完善，提供清晰的错误信息

## 约束
- **技术**：使用 Rust 语言，依赖 oak-yaml 和 serde
- **业务**：保持与原有功能的兼容性
- **依赖**：只使用项目中已有的依赖，不添加新的外部依赖

## 假设
- oak-yaml 库已经存在于项目的依赖中
- 所有需要解析的 YAML 文件格式都是标准的
- 项目使用 serde 进行序列化和反序列化

## 验收标准

### AC-1：恢复 oak-yaml 依赖
- **给定**：项目代码库
- **当**：添加 oak-yaml 依赖并恢复其使用
- **然后**：所有相关文件能够正确编译，通过 cargo check
- **验证**：`programmatic`

### AC-2：添加缺失的 YAML Value 功能
- **给定**：oak-yaml 库
- **当**：为其添加缺失的功能
- **然后**：能够正确解析各种 YAML 值类型，包括字符串、数字、布尔值、数组和对象
- **验证**：`programmatic`

### AC-3：使用 serde 进行配置反序列化
- **给定**：配置文件
- **当**：使用 serde 进行反序列化
- **然后**：配置文件能够正确解析为 Rust 结构体，类型安全
- **验证**：`programmatic`

### AC-4：正确解析 YAML 文件
- **给定**：kubectl.rs、kustomize.rs、helm.rs 和 docker-compose.rs
- **当**：恢复 YAML 解析功能
- **然后**：这些文件能够正确解析对应的 YAML 配置文件
- **验证**：`programmatic`

## 未解决的问题
- [ ] oak-yaml 库的具体版本和依赖关系
- [ ] 具体需要添加哪些缺失的 YAML Value 功能
- [ ] 如何处理不同格式的 YAML 配置文件