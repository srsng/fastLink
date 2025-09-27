# desks-core 模块技术文档

## 1. 模块概述

`desks-core` 是 fastLink 项目中负责桌面环境管理的核心库，提供桌面切换、状态管理、布局保存等功能。该模块基于 `fastlink-core` 构建，专注于 Windows 桌面环境的管理。

### 1.1 核心职责

- **桌面环境管理**：初始化、切换、重置桌面环境
- **状态持久化**：桌面状态的保存和恢复
- **布局管理**：集成 desktop-layout 模块保存桌面图标布局
- **常用路径管理**：管理用户常用的桌面路径
- **符号链接操作**：基于 fastlink-core 的桌面符号链接管理

### 1.2 模块结构

```txt
desks-core/
├── src/
│   ├── lib.rs          # 模块入口和导出
│   ├── state.rs        # 状态管理核心
│   ├── handler/        # 命令处理器
│   │   ├── mod.rs      # 处理器模块导出
│   │   ├── init.rs     # 初始化处理
│   │   ├── set.rs      # 桌面设置处理
│   │   ├── reset.rs    # 重置处理
│   │   ├── original.rs # 原始桌面处理
│   │   ├── state.rs    # 状态查询处理
│   │   ├── usual.rs    # 常用路径处理
│   │   └── fresh.rs    # 刷新处理
│   └── utils/          # 工具函数
│       ├── mod.rs      # 工具模块导出
│       └── ...         # 各种工具函数
└── Cargo.toml          # 依赖配置
```

## 2. 核心数据结构

### 2.1 DesktopState - 桌面状态

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopState {
    /// 原始桌面路径
    pub initial_path: Option<PathBuf>,
    /// 原始桌面临时备份路径
    pub initial_path_temp: Option<PathBuf>,
    /// 当前桌面路径（符号链接）
    pub cur_path: Option<PathBuf>,
    /// 当前桌面目标路径（实际目录）
    pub cur_target: Option<PathBuf>,
    /// 常用桌面路径映射
    pub usual_paths: IndexMap<String, PathBuf>,
}
```

**设计特点**：

- 使用 `Option<PathBuf>` 处理可能不存在的路径
- `IndexMap` 保持常用路径的插入顺序
- 支持 JSON 序列化/反序列化
- 包含完整的桌面状态信息

### 2.2 AutoSaveState - 自动保存状态管理器

```rust
pub struct AutoSaveState {
    state: Mutex<DesktopState>,
    path: String,
}
```

**功能特性**：

- **线程安全**：使用 `Mutex` 保护状态数据
- **自动保存**：状态变更时自动持久化
- **延迟初始化**：使用 `lazy_static` 实现单例
- **错误恢复**：状态文件损坏时自动备份和重建

### 2.3 DESKTOP_STATE - 全局状态实例

```rust
lazy_static! {
    pub static ref DESKTOP_STATE: AutoSaveState = {
        let state_path = get_state_file_path();
        AutoSaveState::new(state_path)
    };
}
```

## 3. 核心功能实现

### 3.1 状态管理机制

#### 3.1.1 状态加载

```rust
impl DesktopState {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        
        if !path.exists() {
            info!("状态文件不存在，创建默认状态: {}", path.display());
            return Self::default();
        }
        
        match fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str::<DesktopState>(&content) {
                    Ok(state) => {
                        info!("成功加载状态文件: {}", path.display());
                        state
                    }
                    Err(e) => {
                        error!("状态文件解析失败: {}, 错误: {}", path.display(), e);
                        Self::handle_corrupted_state(path);
                        Self::default()
                    }
                }
            }
            Err(e) => {
                error!("读取状态文件失败: {}, 错误: {}", path.display(), e);
                Self::default()
            }
        }
    }
    
    fn handle_corrupted_state(path: &Path) {
        let backup_path = path.with_extension("json.backup");
        if let Err(e) = fs::copy(path, &backup_path) {
            error!("备份损坏的状态文件失败: {}", e);
        } else {
            info!("已备份损坏的状态文件到: {}", backup_path.display());
        }
    }
}
```

#### 3.1.2 状态保存

```rust
impl DesktopState {
    pub fn save<P: AsRef<Path>>(&self, path: P) -> MyResult<()> {
        let path = path.as_ref();
        
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 序列化状态
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| MyError::SerializationError(e.to_string()))?;
        
        // 原子性写入
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content)?;
        fs::rename(temp_path, path)?;
        
        debug!("状态已保存到: {}", path.display());
        Ok(())
    }
}
```

### 3.2 命令处理器

#### 3.2.1 初始化处理 (init)

```rust
pub fn handle_desktop_init() -> MyResult<bool> {
    let mut state = DESKTOP_STATE.state_mut();
    
    // 检查是否已初始化
    if state.initial_path.is_some() {
        warn!("桌面管理已初始化");
        return Ok(false);
    }
    
    // 获取当前桌面路径
    let desktop_path = get_desktop_path()?;
    
    // 创建备份目录
    let backup_path = desktop_path.with_file_name("Desktop_temp");
    
    // 备份原始桌面
    if desktop_path.exists() {
        if backup_path.exists() {
            fs::remove_dir_all(&backup_path)?;
        }
        fs::rename(&desktop_path, &backup_path)?;
        info!("原始桌面已备份到: {}", backup_path.display());
    }
    
    // 更新状态
    state.initial_path = Some(desktop_path.clone());
    state.initial_path_temp = Some(backup_path);
    state.cur_path = Some(desktop_path);
    
    // 保存状态
    drop(state);
    DESKTOP_STATE.save()?;
    
    info!("桌面管理初始化完成");
    Ok(true)
}
```

#### 3.2.2 桌面设置处理 (set)

```rust
pub fn handle_desktop_set(target_path: &str) -> MyResult<bool> {
    let target_path = PathBuf::from(target_path);
    
    // 验证目标路径
    if !target_path.exists() {
        return Err(MyError::PathNotFound(target_path));
    }
    
    if !target_path.is_dir() {
        return Err(MyError::InvalidArgument(
            "目标路径必须是目录".to_string()
        ));
    }
    
    let state = DESKTOP_STATE.state();
    
    // 检查初始化状态
    let desktop_path = state.initial_path.as_ref()
        .ok_or(MyError::NotInitialized)?;
    
    drop(state);
    
    // 保存当前布局（如果启用）
    #[cfg(feature = "keep-layout")]
    if let Some(current_target) = DESKTOP_STATE.state().cur_target.as_ref() {
        if let Err(e) = desktop_layout::handler::store_layout(current_target) {
            warn!("保存当前桌面布局失败: {}", e);
        }
    }
    
    // 执行桌面切换
    desktop_set(desktop_path, &target_path)?;
    
    // 恢复目标布局（如果启用）
    #[cfg(feature = "keep-layout")]
    if let Err(e) = desktop_layout::handler::restore_layout(&target_path) {
        warn!("恢复目标桌面布局失败: {}", e);
    }
    
    // 添加到常用路径
    desktop_add_usual(&target_path)?;
    
    info!("桌面已切换到: {}", target_path.display());
    Ok(true)
}

fn desktop_set(desktop_path: &Path, target_path: &Path) -> MyResult<()> {
    // 删除现有符号链接
    if desktop_path.exists() {
        if desktop_path.is_symlink() {
            fs::remove_file(desktop_path)?;
        } else {
            return Err(MyError::InvalidState(
                "桌面路径不是符号链接".to_string()
            ));
        }
    }
    
    // 创建新的符号链接
    #[cfg(windows)]
    {
        use std::os::windows::fs;
        fs::symlink_dir(target_path, desktop_path)?;
    }
    
    // 更新状态
    let mut state = DESKTOP_STATE.state_mut();
    state.cur_target = Some(target_path.to_path_buf());
    drop(state);
    
    DESKTOP_STATE.save()?;
    Ok(())
}
```

#### 3.2.3 重置处理 (reset)

```rust
pub fn handle_desktop_reset(keep_usual: bool) -> MyResult<bool> {
    let state = DESKTOP_STATE.state();
    
    let desktop_path = state.initial_path.as_ref()
        .ok_or(MyError::NotInitialized)?;
    let backup_path = state.initial_path_temp.as_ref()
        .ok_or(MyError::BackupNotFound)?;
    
    drop(state);
    
    // 删除符号链接
    if desktop_path.exists() && desktop_path.is_symlink() {
        fs::remove_file(desktop_path)?;
    }
    
    // 恢复原始桌面
    if backup_path.exists() {
        fs::rename(backup_path, desktop_path)?;
        info!("原始桌面已恢复: {}", desktop_path.display());
    }
    
    // 重置状态
    let mut state = DESKTOP_STATE.state_mut();
    state.initial_path = None;
    state.initial_path_temp = None;
    state.cur_path = None;
    state.cur_target = None;
    
    if !keep_usual {
        state.usual_paths.clear();
    }
    
    drop(state);
    DESKTOP_STATE.save()?;
    
    info!("桌面管理已重置");
    Ok(true)
}
```

### 3.3 常用路径管理

```rust
pub fn desktop_add_usual(path: &Path) -> MyResult<()> {
    let path_str = path.to_string_lossy().to_string();
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let mut state = DESKTOP_STATE.state_mut();
    
    // 避免重复添加
    if !state.usual_paths.values().any(|p| p == path) {
        state.usual_paths.insert(name, path.to_path_buf());
        drop(state);
        DESKTOP_STATE.save()?;
        info!("已添加常用路径: {}", path.display());
    }
    
    Ok(())
}

pub fn handle_desktop_del_usual(name: &str) -> MyResult<bool> {
    let mut state = DESKTOP_STATE.state_mut();
    
    if let Some(removed_path) = state.usual_paths.remove(name) {
        drop(state);
        DESKTOP_STATE.save()?;
        info!("已删除常用路径: {} -> {}", name, removed_path.display());
        Ok(true)
    } else {
        warn!("常用路径不存在: {}", name);
        Ok(false)
    }
}
```

## 4. 布局管理集成

### 4.1 条件编译支持

```rust
#[cfg(feature = "keep-layout")]
mod layout_integration {
    use desktop_layout::handler::{store_layout, restore_layout};
    
    pub fn save_current_layout(desktop_path: &Path) -> MyResult<()> {
        store_layout(desktop_path)
            .map_err(|e| MyError::LayoutError(e.to_string()))
    }
    
    pub fn restore_target_layout(desktop_path: &Path) -> MyResult<()> {
        restore_layout(desktop_path)
            .map_err(|e| MyError::LayoutError(e.to_string()))
    }
}

#[cfg(not(feature = "keep-layout"))]
mod layout_integration {
    use super::*;
    
    pub fn save_current_layout(_desktop_path: &Path) -> MyResult<()> {
        // 布局功能未启用
        Ok(())
    }
    
    pub fn restore_target_layout(_desktop_path: &Path) -> MyResult<()> {
        // 布局功能未启用
        Ok(())
    }
}
```

## 5. 错误处理

### 5.1 扩展错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum DesksError {
    #[error("桌面管理未初始化")]
    NotInitialized,
    
    #[error("备份文件不存在")]
    BackupNotFound,
    
    #[error("状态无效: {0}")]
    InvalidState(String),
    
    #[error("序列化错误: {0}")]
    SerializationError(String),
    
    #[cfg(feature = "keep-layout")]
    #[error("布局管理错误: {0}")]
    LayoutError(String),
    
    #[error("核心库错误: {0}")]
    Core(#[from] fastlink_core::types::MyError),
}
```

## 6. 配置管理

### 6.1 配置文件路径

```rust
pub fn get_state_file_path() -> String {
    let app_data = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fastlink")
        .join("desktop_setter");
    
    app_data.join("state.json")
        .to_string_lossy()
        .to_string()
}

pub fn get_desktop_path() -> MyResult<PathBuf> {
    dirs::desktop_dir()
        .ok_or_else(|| MyError::InvalidArgument(
            "无法获取桌面路径".to_string()
        ))
}
```

### 6.2 日志配置

```rust
pub fn get_log_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fastlink")
        .join("desktop_setter")
        .join("log")
}
```

## 7. 平台特性

### 7.1 Windows 集成

```rust
#[cfg(windows)]
mod windows_integration {
    use winreg::enums::*;
    use winreg::RegKey;
    
    pub fn get_desktop_path_from_registry() -> MyResult<PathBuf> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let desktop_key = hkcu.open_subkey(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders"
        )?;
        
        let desktop_path: String = desktop_key.get_value("Desktop")?;
        Ok(PathBuf::from(desktop_path))
    }
    
    pub fn refresh_desktop() -> MyResult<()> {
        // 刷新桌面显示
        unsafe {
            use windows::Win32::UI::Shell::*;
            SHChangeNotify(
                SHCNE_ASSOCCHANGED,
                SHCNF_IDLIST,
                None,
                None,
            );
        }
        Ok(())
    }
}
```

## 8. 测试策略

### 8.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_state_serialization() {
        let state = DesktopState {
            initial_path: Some(PathBuf::from("C:\\\\Users\\\\test\\\\Desktop")),
            cur_target: Some(PathBuf::from("D:\\\\MyDesktop")),
            usual_paths: IndexMap::new(),
            ..Default::default()
        };
        
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: DesktopState = serde_json::from_str(&json).unwrap();
        
        assert_eq!(state.initial_path, deserialized.initial_path);
        assert_eq!(state.cur_target, deserialized.cur_target);
    }
    
    #[test]
    fn test_state_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test_state.json");
        
        let state = DesktopState::default();
        state.save(&state_file).unwrap();
        
        let loaded_state = DesktopState::load(&state_file);
        assert_eq!(state.initial_path, loaded_state.initial_path);
    }
}
```
