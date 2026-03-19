# Windows 安装和配置指南

本文档详细介绍如何在 Windows 操作系统上安装和配置 Rusty-Docker。

## 系统要求

- Windows 10 1903 或更高版本，或 Windows 11
- 至少 4GB 内存
- 至少 20GB 可用磁盘空间
- 64 位处理器

## 安装选项

Rusty-Docker 在 Windows 上支持两种运行模式：

1. **Hyper-V 模式**：使用 Windows 内置的 Hyper-V 虚拟化技术
2. **WSL 2 模式**：使用 Windows Subsystem for Linux 第二版

## 安装步骤

### 方法一：使用安装程序

1. 下载 Rusty-Docker Windows 安装程序
2. 双击运行安装程序
3. 按照安装向导的指示完成安装
4. 安装完成后，启动 Rusty-Docker

### 方法二：从源代码构建

1. 安装 Rust 开发环境：
   - 访问 https://www.rust-lang.org/zh-CN/tools/install
   - 下载并运行 Rust 安装程序
   - 按照安装向导的指示完成安装

2. 克隆 Rusty-Docker 源代码：
   ```powershell
   git clone https://github.com/nyar-vm/rusty-docker.git
   cd rusty-docker
   ```

3. 构建项目：
   ```powershell
   cargo build
   ```

4. 运行 Rusty-Docker：
   ```powershell
   .\target\debug\docker
   ```

## 配置 Hyper-V 模式

### 启用 Hyper-V

1. 以管理员身份打开 PowerShell
2. 运行以下命令启用 Hyper-V：
   ```powershell
   Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All
   ```
3. 重启计算机

### 验证 Hyper-V 启用状态

1. 以管理员身份打开 PowerShell
2. 运行以下命令：
   ```powershell
   Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V
   ```
3. 确认输出中显示 `State : Enabled`

## 配置 WSL 2 模式

### 启用 WSL

1. 以管理员身份打开 PowerShell
2. 运行以下命令启用 WSL：
   ```powershell
   wsl --install
   ```
3. 重启计算机

### 升级到 WSL 2

1. 以管理员身份打开 PowerShell
2. 运行以下命令设置 WSL 2 为默认版本：
   ```powershell
   wsl --set-default-version 2
   ```

### 验证 WSL 2 安装状态

1. 打开 PowerShell
2. 运行以下命令：
   ```powershell
   wsl --version
   ```
3. 确认输出中显示 WSL 版本信息

## 配置文件共享

### 挂载 Windows 目录到容器

在运行容器时，可以使用 `-v` 参数挂载 Windows 目录到容器：

```powershell
docker run -v C:\Users\username\Documents:/app ubuntu:latest
```

### 支持的路径格式

- Windows 绝对路径：`C:\Users\username\Documents`
- 相对路径：`./data`

## 配置网络

### 端口映射

在运行容器时，可以使用 `-p` 参数映射容器端口到主机端口：

```powershell
docker run -p 8080:80 nginx:latest
```

### 网络模式

Rusty-Docker 支持以下网络模式：

- `bridge`：默认网络模式，容器通过虚拟网络与主机通信
- `host`：容器直接使用主机网络
- `none`：容器没有网络连接

## 常见问题解决

### Hyper-V 启用失败

**问题**：运行 `Enable-WindowsOptionalFeature` 命令失败

**解决方案**：
1. 确认你的 Windows 版本支持 Hyper-V（Windows 10 Pro、Enterprise 或 Education）
2. 检查 BIOS 中是否启用了虚拟化技术
3. 尝试在控制面板中手动启用 Hyper-V：
   - 打开 "控制面板" > "程序" > "程序和功能"
   - 点击 "启用或关闭 Windows 功能"
   - 勾选 "Hyper-V" 选项
   - 点击 "确定" 并重启计算机

### WSL 2 安装失败

**问题**：运行 `wsl --install` 命令失败

**解决方案**：
1. 确认你的 Windows 版本至少是 Windows 10 1903
2. 检查 Windows 更新是否已安装最新更新
3. 尝试手动启用 WSL 功能：
   ```powershell
   Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
   Enable-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform
   ```
4. 下载并安装 WSL 2 内核更新包：https://aka.ms/wsl2kernel

### 容器无法访问外部网络

**问题**：容器无法访问互联网

**解决方案**：
1. 检查 Windows 防火墙设置
2. 尝试重启 Docker 服务
3. 检查网络连接设置

### 文件共享权限问题

**问题**：容器无法读写挂载的 Windows 目录

**解决方案**：
1. 检查 Windows 目录的权限设置
2. 确保容器以正确的用户身份运行
3. 尝试使用 `--privileged` 参数运行容器

## 卸载 Rusty-Docker

### 使用安装程序卸载

1. 打开 "控制面板" > "程序" > "程序和功能"
2. 找到 Rusty-Docker
3. 点击 "卸载" 并按照指示完成卸载

### 从源代码构建的卸载

1. 删除 Rusty-Docker 源代码目录
2. 运行以下命令清理构建文件：
   ```powershell
   cargo clean
   ```

## 联系支持

如果您在安装或使用 Rusty-Docker 时遇到问题，请访问我们的 GitHub 仓库提交 issue：

https://github.com/nyar-vm/rusty-docker/issues
