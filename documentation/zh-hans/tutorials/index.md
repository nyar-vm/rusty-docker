# Rusty Docker 使用教程

## 1. 安装指南

### 1.1 系统要求

- **操作系统**：Windows 10/11、Linux（Ubuntu 20.04+、CentOS 7+）、macOS 10.15+
- **内存**：至少 4GB RAM
- **存储空间**：至少 10GB 可用空间
- **处理器**：支持虚拟化的 64 位处理器

### 1.2 安装步骤

#### 1.2.1 Windows 安装

1. **下载安装包**：从官方网站下载 Rusty Docker Windows 安装包
2. **运行安装程序**：双击安装包，按照提示完成安装
3. **验证安装**：打开命令提示符，运行 `docker --version` 验证安装是否成功

#### 1.2.2 Linux 安装

1. **下载安装包**：从官方网站下载 Rusty Docker Linux 安装包
2. **安装依赖**：
   - Ubuntu/Debian：`sudo apt-get update && sudo apt-get install -y curl`
   - CentOS/RHEL：`sudo yum install -y curl`
3. **运行安装脚本**：
   ```bash
   curl -fsSL https://rusty-docker.io/install.sh | sudo sh
   ```
4. **验证安装**：运行 `docker --version` 验证安装是否成功

#### 1.2.3 macOS 安装

1. **下载安装包**：从官方网站下载 Rusty Docker macOS 安装包
2. **运行安装程序**：双击安装包，按照提示完成安装
3. **验证安装**：打开终端，运行 `docker --version` 验证安装是否成功

## 2. 基本配置

### 2.1 配置文件

Rusty Docker 的配置文件位于：
- **Windows**：`C:\Users\<用户名>\.rusty-docker\config.toml`
- **Linux**：`~/.rusty-docker/config.toml`
- **macOS**：`~/.rusty-docker/config.toml`

### 2.2 配置示例

```toml
# 基本配置
[general]
# 日志级别：debug, info, warn, error
log_level = "info"

# 容器配置
[container]
# 默认网络
default_network = "bridge"
# 默认存储驱动
storage_driver = "overlay2"

# 镜像配置
[image]
# 默认镜像仓库
default_registry = "docker.io"

# 网络配置
[network]
# 默认子网
default_subnet = "172.17.0.0/16"

# 存储配置
[storage]
# 数据根目录
data_root = "/var/lib/rusty-docker"
```

### 2.3 环境变量

| 环境变量 | 描述 | 默认值 |
|---------|------|-------|
| `docker_CONFIG` | 配置文件路径 | 见上文 |
| `docker_DATA_ROOT` | 数据根目录 | 见上文 |
| `docker_LOG_LEVEL` | 日志级别 | info |
| `docker_REGISTRY` | 默认镜像仓库 | docker.io |

## 3. 基本操作

### 3.1 容器管理

#### 3.1.1 创建并运行容器

```bash
# 运行一个简单的 Nginx 容器
docker run -d --name my-nginx -p 8080:80 nginx

# 运行一个带有环境变量的容器
docker run -d --name my-app -e "MY_ENV=value" my-app-image

# 运行一个挂载卷的容器
docker run -d --name my-db -v /path/to/data:/data my-db-image
```

#### 3.1.2 查看容器

```bash
# 查看所有容器
docker ps -a

# 查看运行中的容器
docker ps

# 查看容器详情
docker inspect my-container
```

#### 3.1.3 管理容器

```bash
# 启动容器
docker start my-container

# 停止容器
docker stop my-container

# 重启容器
docker restart my-container

# 删除容器
docker rm my-container

# 进入容器
docker exec -it my-container bash
```

#### 3.1.4 查看容器日志

```bash
# 查看容器日志
docker logs my-container

# 实时查看容器日志
docker logs -f my-container
```

### 3.2 镜像管理

#### 3.2.1 拉取镜像

```bash
# 拉取最新版本的镜像
docker pull nginx

# 拉取指定版本的镜像
docker pull nginx:1.21
```

#### 3.2.2 查看镜像

```bash
# 查看所有本地镜像
docker images

# 查看镜像详情
docker inspect nginx
```

#### 3.2.3 构建镜像

```bash
# 在当前目录构建镜像
docker build -t my-image .

# 使用指定的 Dockerfile 构建镜像
docker build -t my-image -f Dockerfile.dev .
```

#### 3.2.4 推送镜像

```bash
# 推送镜像到仓库
docker push my-registry/my-image:tag
```

### 3.3 网络管理

#### 3.3.1 创建网络

```bash
# 创建桥接网络
docker network create my-network

# 创建覆盖网络
docker network create -d overlay my-overlay-network
```

#### 3.3.2 查看网络

```bash
# 查看所有网络
docker network ls

# 查看网络详情
docker network inspect my-network
```

#### 3.3.3 连接容器到网络

```bash
# 运行容器时连接到网络
docker run -d --name my-container --network my-network my-image

# 将现有容器连接到网络
docker network connect my-network my-container
```

### 3.4 存储管理

#### 3.4.1 创建卷

```bash
# 创建卷
docker volume create my-volume

# 查看所有卷
docker volume ls

# 查看卷详情
docker volume inspect my-volume
```

#### 3.4.2 使用卷

```bash
# 运行容器时挂载卷
docker run -d --name my-container -v my-volume:/data my-image
```

### 3.5 Docker Compose

#### 3.5.1 编写 Compose 文件

创建 `docker-compose.yml` 文件：

```yaml
version: '3'
services:
  web:
    image: nginx
    ports:
      - "8080:80"
    volumes:
      - ./html:/usr/share/nginx/html
  db:
    image: mysql
    environment:
      MYSQL_ROOT_PASSWORD: example
      MYSQL_DATABASE: mydb
    volumes:
      - db-data:/var/lib/mysql

volumes:
  db-data:
```

#### 3.5.2 运行 Compose 应用

```bash
# 启动应用
docker-compose up -d

# 查看应用状态
docker-compose ps

# 停止应用
docker-compose down
```

### 3.6 Kubernetes 基本操作

#### 3.6.1 集群管理

```bash
# 初始化集群
kubeadm init

# 加入节点
kubeadm join <master-ip>:<port> --token <token> --discovery-token-ca-cert-hash <hash>

# 查看集群状态
kubectl cluster-info
```

#### 3.6.2 资源管理

```bash
# 查看 Pod
kubectl get pods

# 查看服务
kubectl get services

# 查看部署
kubectl get deployments

# 应用配置
kubectl apply -f deployment.yaml

# 删除资源
kubectl delete -f deployment.yaml
```

## 4. 前端界面使用

### 4.1 访问 docker-crab-h5

1. **启动前端服务**：
   ```bash
   cd frontends/docker-crab-h5
   pnpm dev
   ```

2. **访问界面**：打开浏览器，访问 `http://localhost:5173`

### 4.2 容器管理界面

- **容器列表**：查看所有容器的状态、名称、镜像、端口等信息
- **容器操作**：启动、停止、重启、删除容器
- **容器详情**：查看容器的详细信息，包括环境变量、网络配置、挂载卷等
- **容器日志**：实时查看容器的日志输出
- **容器终端**：通过 Web 终端进入容器执行命令

### 4.3 镜像管理界面

- **镜像列表**：查看所有本地镜像的信息
- **镜像操作**：拉取、删除镜像
- **镜像详情**：查看镜像的详细信息，包括标签、大小、创建时间等
- **镜像构建**：通过 Dockerfile 构建镜像

### 4.4 网络和存储管理界面

- **网络管理**：创建、删除网络，查看网络详情
- **存储管理**：创建、删除卷，查看卷详情

### 4.5 多容器应用编排界面

- **Compose 文件管理**：上传、编辑、删除 Compose 配置文件
- **服务管理**：启动、停止、重启服务
- **服务详情**：查看服务的详细信息

### 4.6 集群管理界面

- **集群状态**：查看集群的整体状态
- **节点管理**：查看和管理集群节点
- **工作负载管理**：查看和管理集群中的工作负载
- **服务管理**：查看和管理集群中的服务

## 5. 高级功能

### 5.1 容器资源限制

```bash
# 限制容器 CPU 和内存
docker run -d --name my-container --cpus=1 --memory=1g my-image
```

### 5.2 容器健康检查

```bash
# 添加健康检查
docker run -d --name my-container --health-cmd="curl -f http://localhost/ || exit 1" --health-interval=30s --health-timeout=10s --health-retries=3 my-image
```

### 5.3 镜像分层管理

```bash
# 查看镜像分层
docker history my-image
```

### 5.4 网络模式

```bash
# 使用主机网络
docker run -d --name my-container --network=host my-image

# 使用容器网络
docker run -d --name my-container --network=container:other-container my-image
```

### 5.5 存储驱动

```bash
# 指定存储驱动
dockerd --storage-driver=overlay2
```

## 6. 故障排除

### 6.1 常见问题

#### 6.1.1 容器启动失败

- **检查日志**：`docker logs my-container`
- **检查端口冲突**：确保端口未被其他服务占用
- **检查资源限制**：确保容器有足够的资源

#### 6.1.2 镜像拉取失败

- **检查网络连接**：确保网络连接正常
- **检查镜像名称**：确保镜像名称正确
- **检查认证信息**：如果使用私有仓库，确保认证信息正确

#### 6.1.3 网络问题

- **检查网络配置**：确保网络配置正确
- **检查防火墙**：确保防火墙未阻止容器网络
- **检查 DNS 配置**：确保 DNS 配置正确

### 6.2 日志管理

```bash
# 查看 Docker 守护进程日志
# Windows：事件查看器 -> 应用程序和服务日志 -> Rusty Docker
# Linux：journalctl -u rusty-docker
# macOS：控制台 -> 日志 -> Rusty Docker
```

### 6.3 系统检查

```bash
# 检查系统信息
docker info

# 检查磁盘使用情况
docker system df

# 清理未使用的资源
docker system prune
```

## 7. 最佳实践

### 7.1 容器最佳实践

- **使用官方镜像**：优先使用官方镜像
- **最小化镜像大小**：使用 Alpine 基础镜像，减少镜像大小
- **使用多阶段构建**：减少最终镜像大小
- **设置健康检查**：确保容器健康状态可监控
- **使用非 root 用户**：提高安全性

### 7.2 网络最佳实践

- **使用自定义网络**：为应用创建专用网络
- **使用网络别名**：便于容器间通信
- **限制网络访问**：只开放必要的端口

### 7.3 存储最佳实践

- **使用卷**：持久化数据
- **使用命名卷**：便于管理
- **定期备份**：定期备份卷数据

### 7.4 安全最佳实践

- **定期更新镜像**：及时更新镜像中的安全补丁
- **使用 Secrets**：管理敏感信息
- **限制容器权限**：使用最小权限原则
- **启用内容信任**：验证镜像完整性

## 8. 示例应用

### 8.1 基本 Web 应用

**目标**：部署一个基本的 Nginx Web 服务器

**步骤**：
1. **拉取镜像**：`docker pull nginx`
2. **运行容器**：`docker run -d --name web-server -p 80:80 nginx`
3. **访问应用**：打开浏览器，访问 `http://localhost`

### 8.2 多容器应用

**目标**：部署一个包含 Web 服务器和数据库的应用

**步骤**：
1. **创建 Compose 文件**：
   ```yaml
   version: '3'
   services:
     web:
       image: nginx
       ports:
         - "80:80"
       volumes:
         - ./html:/usr/share/nginx/html
     db:
       image: mysql
       environment:
         MYSQL_ROOT_PASSWORD: example
         MYSQL_DATABASE: mydb
       volumes:
         - db-data:/var/lib/mysql
   
   volumes:
     db-data:
   ```
2. **启动应用**：`docker-compose up -d`
3. **访问应用**：打开浏览器，访问 `http://localhost`

### 8.3 Kubernetes 部署

**目标**：在 Kubernetes 集群中部署一个应用

**步骤**：
1. **创建 Deployment 文件**：
   ```yaml
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: my-app
   spec:
     replicas: 3
     selector:
       matchLabels:
         app: my-app
     template:
       metadata:
         labels:
           app: my-app
       spec:
         containers:
         - name: my-app
           image: nginx
           ports:
           - containerPort: 80
   ```
2. **创建 Service 文件**：
   ```yaml
   apiVersion: v1
   kind: Service
   metadata:
     name: my-app
   spec:
     selector:
       app: my-app
     ports:
     - port: 80
       targetPort: 80
     type: LoadBalancer
   ```
3. **应用配置**：`kubectl apply -f deployment.yaml -f service.yaml`
4. **访问应用**：使用 `kubectl get services` 获取服务 IP，然后访问 `http://<service-ip>`

## 9. 总结

本教程介绍了 Rusty Docker 的基本使用方法，包括安装、配置、基本操作和高级功能。通过本教程，您应该能够：

- 安装和配置 Rusty Docker
- 管理容器、镜像、网络和存储
- 使用 Docker Compose 编排多容器应用
- 使用 Kubernetes 管理容器集群
- 使用前端界面管理容器资源
- 排查常见问题
- 遵循最佳实践

Rusty Docker 提供了与 Docker 和 Kubernetes 类似的功能，同时具有 Rust 语言的安全性和性能优势。通过本教程的学习，您可以开始使用 Rusty Docker 来管理和部署容器化应用。