# MCP Containerd

这是一个使用RMCP（Rust Model Context Protocol）库实现的MCP服务器，用于操作Containerd的CRI接口。

## 功能特点

- 使用RMCP库实现MCP服务器
- 支持Containerd的所有CRI接口操作
- 提供运行时服务（Runtime Service）接口
- 提供镜像服务（Image Service）接口

## 前置条件

- Rust 开发环境
- Containerd 已安装并运行
- Protobuf 编译工具

## 构建

```bash
cargo build --release
```

## 运行

```bash
cargo run --release
```

默认情况下，服务将连接到 `unix:///run/containerd/containerd.sock` 端点。

## 服务结构

MCP服务器包含以下主要组件：

- `version` 服务：提供CRI版本信息
- `runtime` 服务：提供容器和Pod的运行时操作
- `image` 服务：提供容器镜像操作

## CRI接口

### 运行时服务

- 创建/停止/删除 Pod Sandbox
- 创建/启动/停止/删除 容器
- 查询 Pod/容器 状态
- 执行容器内命令

### 镜像服务

- 列出镜像
- 获取镜像状态
- 拉取镜像
- 删除镜像
- 获取镜像文件系统信息

## 配置

目前使用默认配置，后续版本将支持通过配置文件来自定义连接参数。

## 许可证

Apache-2.0 