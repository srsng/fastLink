# CLI工具技术文档

## 1. 模块概述

### 1.1 CLI工具组件

fastLink项目包含三个主要的命令行工具：

- **fastlink-cli**：智能符号链接创建工具
- **desks-cli**：桌面环境管理工具
- **desks-tray**：系统托盘桌面切换工具

### 1.2 设计理念

- **用户友好**：直观的命令行接口和丰富的参数选项
- **功能强大**：支持正则表达式、批量操作、智能检测
- **错误处理**：完善的错误处理和用户反馈机制
- **日志系统**：统一的日志管理和调试支持

## 2. fastlink-cli - 符号链接工具

### 2.1 核心功能

```rust
// 主要入口点
fn main() {
    let args: Args = Args::parse();
    
    // 初始化日志系统
    fastlink_core::utils::logs::LogIniter::new(args.quiet, args.debug, args.save_log.clone())
        .init();
    
    // 构建和执行任务
    let task_res = LinkTask::try_from(&args);
    match task_res {
        Ok(task) => match task.work() {
            Ok(()) => (),
            Err(e) => e.log(),
        },
        Err(e) => e.log(),
    }
}
```

### 2.2 命令行参数设计

#### 基础参数

```rust
#[derive(Parser, Debug)]
#[command(
    version,
    about = "A tool to make symlink fastly and smartly\\n一个智能且方便的符号链接创建工具",
    long_about = EXAMPLE
)]
pub struct Args {
    /// 源文件/源目录路径
    #[arg(required = true, value_parser = validate_src)]
    pub src: String,
    
    /// 目标路径，可选
    pub dst: Option<String>,
    
    /// 检查模式
    #[arg(short, long)]
    pub check: bool,
    
    /// 删除模式
    #[arg(long)]
    pub rm: bool,
}
```

#### 高级功能参数

```rust
/// 保留文件扩展名
#[arg(short, long)]
pub keep_extention: bool,

/// 自动创建目录
#[arg(long, visible_alias("md"))]
pub make_dir: bool,

/// 正则表达式匹配
#[cfg(feature = "fastlink-regex")]
#[arg(long, visible_alias("re"), value_parser = validate_regex)]
pub regex: Option<regex::Regex>,

/// 正则匹配最大深度
#[cfg(feature = "fastlink-regex")]
#[arg(long, visible_alias("re-depth"), value_parser = validate_re_max_depth)]
pub re_max_depth: Option<usize>,
```

#### 过滤和行为控制

```rust
/// 只处理文件
#[arg(long, conflicts_with = "only_dir", visible_alias("F"))]
pub only_file: bool,

/// 只处理目录
#[arg(long, conflicts_with = "only_file", visible_alias("D"))]
pub only_dir: bool,

/// 覆盖已存在的链接
#[arg(long, visible_alias("overwrite"), conflicts_with = "skip_exist_links")]
pub overwrite_links: bool,

/// 跳过已存在的链接
#[arg(long, visible_alias("skip-exist"), conflicts_with = "overwrite_links")]
pub skip_exist_links: bool,
```

### 2.3 参数验证

```rust
/// 源路径验证
fn validate_src(s: &str) -> Result<String, String> {
    let path = PathBuf::from(s);
    let cleaned = path.clean();
    
    if cleaned.to_string_lossy().is_empty() {
        return Err("路径不能为空".to_string());
    }
    
    Ok(cleaned.to_string_lossy().to_string())
}

/// 正则表达式验证
#[cfg(feature = "fastlink-regex")]
pub fn validate_regex(pattern: &str) -> Result<regex::Regex, String> {
    regex::Regex::new(pattern)
        .map_err(|e| format!("无效的正则表达式: {}", e))
}

/// 深度参数验证
#[cfg(feature = "fastlink-regex")]
fn validate_re_max_depth(s: &str) -> Result<usize, String> {
    s.parse::<usize>()
        .map_err(|_| "深度必须是非负整数".to_string())
}
```

## 3. desks-cli - 桌面管理工具

### 3.1 核心架构

```rust
fn main() {
    let args: Args = Args::parse();
    
    // 初始化日志系统
    fastlink_core::utils::logs::LogIniter::new(args.quiet, args.debug, None).init();
    
    // 初始化桌面状态
    {
        let _state = DESKTOP_STATE.state();
    }
    
    // 处理命令
    if let Err(e) = handle_desktop_setter(args) {
        e.log()
    }
}
```

### 3.2 命令系统

#### 命令定义

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 初始化桌面管理
    Init,
    
    /// 重置桌面到原始状态
    Reset {
        #[arg(long)]
        keep_usual_paths: bool,
    },
    
    /// 设置新的桌面目录
    Set {
        new_desktop_dir_path: String,
        #[arg(long)]
        make_dir: bool,
        #[arg(long)]
        usual: Option<String>,
    },
    
    /// 显示当前桌面状态
    State,
    
    /// 恢复到原始桌面
    Original,
    
    /// 切换到常用桌面
    Usual { name: String },
    
    /// 删除常用桌面
    DelUsual { name: String },
}
```

#### 命令处理器

```rust
fn handle_desktop_setter(args: Args) -> MyResult<bool> {
    match args.command {
        Commands::Init => handle_desktop_init(),
        Commands::Reset { keep_usual_paths } => handle_desktop_reset(Some(keep_usual_paths)),
        Commands::Set { new_desktop_dir_path, make_dir, usual } => 
            handle_desktop_set(new_desktop_dir_path, make_dir, usual),
        Commands::State => handle_desktop_state(),
        Commands::Original => handle_desktop_origin(),
        Commands::Usual { name } => handle_desktop_usual_setby(&name),
        Commands::DelUsual { name } => handle_desktop_usual_del(&name),
    }
}
```

## 4. desks-tray - 系统托盘工具

### 4.1 功能特性

- **系统托盘集成**：常驻系统托盘，提供快速访问
- **图形化界面**：直观的右键菜单操作
- **实时状态显示**：显示当前桌面状态
- **快速切换**：一键切换常用桌面
- **消息通知**：操作结果的系统通知

### 4.2 核心组件

```rust
// 托盘管理器
pub struct TrayManager {
    menu: Menu,
    icon: Icon,
    event_handler: EventHandler,
}

// 消息框显示
pub fn show_message_box(title: &str, message: &str, icon_type: MessageBoxIcon) {
    // Windows消息框实现
}
```

## 5. 日志与错误处理

### 5.1 统一日志管理

```rust
// 日志初始化
fastlink_core::utils::logs::LogIniter::new(quiet, debug, save_log)
    .init();
```

- **级别**: ERROR, WARN, INFO, DEBUG
- **输出**: 控制台, 可选文件
- **安静模式**: `--quiet`

### 5.2 统一错误处理

- 使用 `fastlink-core` 的 `MyError` 和 `MyResult`
- 提供中文、详细的错误信息

## 6. 特性标志

- **fastlink-regex**：启用正则表达式支持
- **save-log**：启用日志文件保存
- **tray**：启用系统托盘功能
