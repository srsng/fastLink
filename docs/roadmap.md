# fastLink 项目路线图

AI生成，仅供参考

## 🎯 项目愿景

将 fastLink 发展为现代化的异步 Rust 应用程序，提供高性能的符号链接管理和桌面环境管理功能，支持 Windows 和 Linux 双平台。

## 📋 当前状态分析

### 已实现功能

- ✅ 符号链接创建和管理 (Windows)
- ✅ 桌面图标布局保存/恢复 (Windows)
- ✅ 系统托盘集成 (Windows)
- ✅ CLI 工具套件
- ✅ 基础错误处理系统

### 技术债务

- ❌ 依赖管理不统一
- ❌ fastlink-core 中存在无用的 clap 依赖
- ❌ 缺乏异步支持
- ❌ 日志系统较为简单
- ❌ 错误处理可以进一步优化
- ❌ 仅支持 Windows 平台

## 🚀 发展阶段

### 阶段一：技术债务清理（1-2个月）

#### 1.1 依赖管理优化

**目标**：统一依赖版本，清理无用依赖

- 在根 Cargo.toml 中定义 workspace dependencies
- 清理 fastlink-core 中的无用 clap 依赖
- 统一所有 crate 的依赖版本

#### 1.2 代码质量提升

- 添加更多单元测试
- 改进文档覆盖率
- 统一代码风格

### 阶段二：日志系统升级（2-3个月）

#### 2.1 迁移到 tracing 生态

**当前**：log + env_logger  
**目标**：tracing + tracing-subscriber

**迁移策略**：

1. 逐步替换 log 宏为 tracing 宏
2. 实现结构化日志
3. 添加分布式追踪支持
4. 保持向后兼容性

#### 2.2 预期收益

- 结构化日志输出
- 更好的性能监控
- 异步友好的日志系统
- 更丰富的上下文信息

### 阶段三：异步支持（3-4个月）

#### 3.1 引入 tokio 运行时

**目标**：为核心 API 添加异步支持

**重构计划**：

1. 文件系统操作异步化
2. 桌面状态管理异步化
3. 网络操作支持（未来扩展）
4. 并发任务处理

#### 3.2 API 设计示例

```rust
// 当前同步 API
pub fn create_link(src: &Path, dst: &Path) -> Result<(), MyError>

// 未来异步 API
pub async fn create_link(src: &Path, dst: &Path) -> Result<(), MyError>
pub async fn create_links_batch(links: Vec<(PathBuf, PathBuf)>) -> Result<Vec<Result<(), MyError>>, MyError>
```

### 阶段四：错误处理优化（4-5个月）

#### 4.1 引入现代错误处理

**当前**：自定义 MyError 系统  
**目标**：thiserror + anyhow

**迁移策略**：

1. 使用 thiserror 定义结构化错误类型
2. 在应用层使用 anyhow 进行错误传播
3. 保持错误信息的丰富性
4. 添加错误恢复机制

#### 4.2 事务性操作支持

- 批量操作的原子性保证
- 操作失败时的自动回滚
- 操作日志和审计

### 阶段五：跨平台支持 - Linux（5-7个月）

#### 5.1 平台抽象层设计

**目标**：创建统一的平台抽象接口

**架构设计**：

```rust
// 平台抽象 trait
pub trait PlatformProvider {
    async fn create_symlink(&self, src: &Path, dst: &Path) -> Result<(), PlatformError>;
    async fn get_desktop_layout(&self) -> Result<DesktopLayout, PlatformError>;
    async fn set_desktop_layout(&self, layout: &DesktopLayout) -> Result<(), PlatformError>;
    async fn create_system_tray(&self) -> Result<Box<dyn SystemTray>, PlatformError>;
}

// Windows 实现
pub struct WindowsPlatform;
impl PlatformProvider for WindowsPlatform { /* ... */ }

// Linux 实现
pub struct LinuxPlatform;
impl PlatformProvider for LinuxPlatform { /* ... */ }
```

#### 5.2 Linux 特定功能实现

**5.2.1 符号链接管理**

- 利用 Linux 原生符号链接支持
- 处理权限和所有权问题
- 支持硬链接和软链接选择

**5.2.2 桌面环境支持**

- **GNOME 支持**：通过 gsettings 管理桌面图标
- **KDE 支持**：通过 kconfig 和 plasma API
- **XFCE 支持**：通过配置文件管理
- **其他 DE**：通用 freedesktop.org 标准支持

**5.2.3 系统托盘集成**

- 使用 `libappindicator` 或 `StatusNotifierItem`
- 支持不同桌面环境的托盘实现
- 处理 Wayland 和 X11 的差异

#### 5.3 依赖管理

**新增 Linux 依赖**：

```toml
# Linux 特定依赖
[target.'cfg(unix)'.dependencies]
libc = "0.2"
nix = "0.27"
gtk = { version = "0.18", optional = true }
libappindicator = { version = "0.9", optional = true }

# 桌面环境特定
[features]
gnome = ["gtk", "gio"]
kde = ["dbus", "kdialog"]
xfce = ["gtk"]
system-tray = ["libappindicator"]
```

#### 5.4 配置和数据存储

**Linux 标准目录**：

- 配置文件：`~/.config/fastlink/`
- 数据文件：`~/.local/share/fastlink/`
- 缓存文件：`~/.cache/fastlink/`
- 日志文件：`~/.local/share/fastlink/logs/`

#### 5.5 测试策略

**多发行版测试**：

- Ubuntu/Debian 系列
- Fedora/RHEL 系列
- Arch Linux
- openSUSE

**桌面环境测试**：

- GNOME (Wayland/X11)
- KDE Plasma (Wayland/X11)
- XFCE
- i3/sway (窗口管理器)

### 阶段六：性能优化（7-8个月）

#### 5.1 缓存机制

- 文件系统状态缓存
- 桌面布局缓存
- 智能缓存失效

#### 5.2 批处理优化

- 批量符号链接创建
- 并行文件操作
- 内存使用优化

## 📊 预期收益

### 跨平台支持

- 扩大用户基础到 Linux 用户
- 统一的跨平台 API
- 更好的代码复用性

### Linux 生态集成

- 符合 Linux 桌面标准
- 支持主流桌面环境
- 原生 Linux 用户体验

### 开发体验

- 现代化的错误处理
- 结构化日志
- 更好的可测试性

### 可维护性

- 统一的依赖管理
- 清晰的模块边界
- 完善的文档

### 稳定性

- 事务性操作
- 更好的错误恢复
- 全面的测试覆盖

### 可扩展性

- 异步架构为未来功能奠定基础
- 模块化设计便于功能扩展
- 现代化技术栈

## 🎯 里程碑

- **M1 (2个月)**：技术债务清理完成
- **M2 (3个月)**：日志系统升级完成
- **M3 (4个月)**：核心异步支持完成
- **M4 (5个月)**：错误处理优化完成
- **M5 (7个月)**：Linux 平台支持完成
- **M6 (8个月)**：性能优化完成，发布 v2.0 (跨平台版本)

## 🐧 Linux 支持详细规划

### 技术挑战

1. **桌面环境多样性**
   - 不同 DE 的配置方式差异巨大
   - 需要为每个主流 DE 提供专门支持
   - Wayland vs X11 的兼容性问题

2. **权限管理**
   - Linux 的严格权限模型
   - 符号链接的权限继承
   - 用户目录访问限制

3. **包管理和分发**
   - 支持多种包管理器 (apt, yum, pacman, zypper)
   - AppImage/Flatpak/Snap 打包
   - 依赖管理复杂性

### 实现优先级

**高优先级**：

1. 核心符号链接功能 (所有 Linux 发行版通用)
2. GNOME 桌面支持 (最大用户群体)
3. 基础 CLI 工具

**中优先级**：

1. KDE Plasma 支持
2. 系统托盘集成
3. XFCE 支持

**低优先级**：

1. 其他窗口管理器支持
2. 特殊发行版优化
3. 高级桌面定制功能

### 兼容性策略

1. **渐进式支持**：先支持核心功能，再扩展桌面特性
2. **功能降级**：不支持的桌面环境提供基础功能
3. **用户选择**：允许用户手动指定桌面环境类型
4. **自动检测**：智能检测当前桌面环境并选择最佳实现

## 📝 注意事项

1. **向后兼容性**：确保现有用户的使用不受影响
2. **渐进式迁移**：避免大规模重写，采用渐进式改进
3. **测试覆盖**：每个阶段都要保证充分的测试
4. **文档更新**：及时更新文档以反映最新状态
5. **社区反馈**：积极收集和响应用户反馈
6. **跨平台兼容性**：确保 Windows 和 Linux 功能对等
7. **桌面环境适配**：充分测试各种 Linux 桌面环境
8. **权限处理**：妥善处理 Linux 权限模型的复杂性
