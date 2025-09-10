# fastLink 项目文档

AI生成，仅供参考

## 项目简介

fastLink 是一个专为 Windows 平台设计的智能符号链接管理系统，集成了基于符号链接的桌面环境管理功能。项目采用 Rust 语言开发，提供了从底层符号链接操作到高级桌面管理的完整解决方案。

## 核心特性

- **智能符号链接管理**：支持正则表达式模式匹配的批量符号链接创建
- **动态桌面切换**：基于符号链接实现的桌面内容动态切换
- **布局保持**：桌面图标布局的保存与恢复功能
- **多场景适配**：工作、休闲、隐私等不同使用场景的桌面环境切换

## 快速开始

### 环境要求

- **操作系统**：Windows 10/11
- **权限**：开发者模式或管理员权限
- **依赖**：无额外运行时依赖

### 安装使用

1. 下载预编译二进制文件
2. 将可执行文件放置到 PATH 目录
3. 启用 Windows 开发者模式（推荐）
4. 运行初始化命令：

```bash
desks init
```

### 基本使用

```bash
# 创建工作桌面
desks set D:\Desktops\work --mk

# 添加到常用路径
desks set D:\Desktops\personal --mk -u personal

# 快速切换
desku work
desku personal

# 恢复原始桌面(重置请使用reset)
desks original
```

## 📚 文档导航

### 🔍 快速入口

- [📋 文档概览](overview.md) - 完整的文档体系介绍
- [👥 用户指南](user-guide.md) - 新用户必读
- [👨‍💻 开发者指南](developer-guide.md) - 开发者入门

### 📁 分类文档

#### 📋 项目管理

- [需求文档](requirements.md) - 详细的功能和非功能需求
- [待办事项](todo.md) - 项目开发计划和任务跟踪

#### 🏗️ 架构设计

- [系统架构](architecture.md) - 整体架构设计和模块关系
- [架构审查](architecture-review.md) - 架构问题分析和改进建议

#### 🔧 技术文档

- [fastlink-core](fastlink-core.md) - 核心库技术文档
- [desks-core](desks-core.md) - 桌面管理核心库文档
- [desktop-layout](desktop-layout.md) - 桌面布局管理模块文档
- [CLI工具](cli-tools.md) - 命令行工具使用说明

#### 👥 用户文档

- [用户指南](user-guide.md) - 完整的用户使用手册

#### 👨‍💻 开发者文档

- [开发者指南](developer-guide.md) - 开发环境配置和贡献指南

## 🎯 使用建议

### 新用户入门路径

1. 📖 阅读本文档了解项目概况
2. 📋 查看 [需求文档](requirements.md) 了解功能特性
3. 👥 参考 [用户指南](user-guide.md) 进行安装和使用

### 开发者入门路径

1. 🏗️ 阅读 [系统架构](architecture.md) 了解整体设计
2. 👨‍💻 参考 [开发者指南](developer-guide.md) 配置开发环境
3. 🔧 查看 [技术文档](.) 了解具体实现
4. 📋 查看 [架构审查](architecture-review.md) 了解改进方向

### 维护者工作流程

1. 📋 查看 [待办事项](todo.md) 了解待办任务
2. 🏗️ 参考 [架构审查](architecture-review.md) 进行优化
3. 🔧 更新相关技术文档

## 📊 项目状态

- **开发状态**: 🚧 积极开发中
- **文档状态**: ✅ 完整覆盖
- **测试状态**: ⚠️ 需要改进
- **发布状态**: 🔄 持续集成

## 🤝 贡献指南

欢迎贡献代码、文档或反馈问题！请参考 [开发者指南](developer-guide.md) 了解详细的贡献流程。
