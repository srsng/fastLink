# desktop-layout 模块技术文档

## 1. 模块概述

### 1.1 核心职责

`desktop-layout` 模块是 fastLink 项目中负责桌面图标布局管理的核心组件，主要功能包括：

- **图标布局获取**：通过嵌入式Python程序获取当前桌面图标布局
- **布局数据管理**：支持DSV格式的布局数据序列化和反序列化
- **布局恢复**：将保存的布局应用到当前桌面
- **布局比较和过滤**：支持不同布局间的差异分析
- **跨平台兼容**：专门针对Windows平台的桌面管理

### 1.2 模块结构

```txt
desktop-layout/
├── src/
│   ├── lib.rs          # 模块入口和错误处理
│   ├── layout.rs       # 核心布局数据结构
│   ├── layout2.rs      # 布局应用功能
│   ├── handler.rs      # 布局操作处理器
│   ├── utils.rs        # 工具函数
│   └── bad_exp.rs      # 实验性功能
├── assets/
│   └── get_icon_layout.exe  # 嵌入式布局获取程序
├── py_get_layout/      # Python布局获取源码
└── Cargo.toml          # 依赖配置
```

## 2. 核心数据结构

### 2.1 IconEntry - 图标条目

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconEntry {
    pub name: String, // 图标名称
    pub x: i32,       // X 坐标
    pub y: i32,       // Y 坐标
}

impl IconEntry {
    pub fn point(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}
```

**设计特点**：

- 简洁的图标位置表示
- 支持坐标点快速获取
- 实现了完整的比较和调试特性

### 2.2 IconLayout - 桌面布局

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IconLayout {
    pub icon_count: usize,       // 图标数量
    pub entries: Vec<IconEntry>, // 图标条目
}
```

**核心方法**：

```rust
impl IconLayout {
    /// 转换为DSV格式字符串
    pub fn fmt2dsv(self) -> String
    
    /// 保存布局到文件
    pub fn dump<P: AsRef<Path>>(&self, path: P) -> MyResult<()>
    
    /// 过滤布局，求两个布局的交集
    pub fn filter(&self, another: &IconLayout) -> (Vec<&IconEntry>, HashMap<&String, usize>)
}
```

**格式转换支持**：

- `From<&IconLayout> for String` - 转换为DSV格式
- `TryFrom<&str> for IconLayout` - 从DSV字符串解析
- `TryFrom<String> for IconLayout` - 从DSV字符串解析

## 3. 核心功能实现

### 3.1 布局获取 (handler.rs)

#### 嵌入式程序执行

```rust
const EXE_BYTES: &[u8] = include_bytes!("../assets/get_icon_layout.exe");

fn run_embedded_exe() -> MyResult<String> {
    // 将嵌入的exe写入临时目录
    let temp_dir = std::env::temp_dir();
    let exe_path = temp_dir.join(r"fastlink\\desktop-setter\\get_icon_layout.exe");
    
    // 执行并获取GBK编码的输出
    let output = Command::new(&exe_path)
        .creation_flags(_CREATE_NO_WINDOW)
        .output()?;
        
    // 处理GBK编码
    let (decoded, _, had_errors) = GBK.decode(&output.stdout);
}
```

**技术特点**：

- 嵌入式二进制文件，无需外部依赖
- 隐藏命令行窗口执行
- GBK编码处理，适配中文环境
- 临时文件管理和清理

#### 布局获取API

```rust
/// 获取当前桌面布局
pub fn get_cur_layout() -> MyResult<IconLayout>

/// 获取新布局（带重试机制）
pub fn get_new_layout(last_layout: Option<&IconLayout>) -> MyResult<IconLayout>

/// 存储当前布局到应用数据目录
pub fn store_cur_layout_by_deskdir_to_appdata<P: AsRef<Path>>(desk_dir: P) -> MyResult<IconLayout>

/// 存储当前布局到DSV文件
pub fn store_cur_layout_to_dsv<P: AsRef<Path>>(path: P) -> MyResult<IconLayout>
```

### 3.2 布局恢复 (layout2.rs)

```rust
/// 恢复桌面布局从应用数据目录
pub fn restore_desktop_layout_by_deskdir_from_appdata<P: AsRef<Path>>(
    desk_dir: P,
    last_layout: Option<&IconLayout>,
) -> MyResult<bool>

/// 从DSV文件恢复桌面布局
pub fn restore_desktop_layout_from_dsv<P: AsRef<Path>>(
    dsv_path: P,
    layout_cur: Option<IconLayout>,
) -> MyResult<bool>
```

**恢复流程**：

1. 读取保存的布局数据
2. 获取当前桌面布局
3. 进行布局过滤和匹配
4. 应用新的图标位置
5. 验证恢复结果

### 3.3 DSV格式处理

#### DSV格式规范

```DSV
Desks (github/fastLink)

# no res info
# 20241220143022
icon_name1,x1,y1
icon_name2,x2,y2
...
```

**格式特点**：

- 头部标识和时间戳
- CSV格式的图标数据
- 支持中文图标名称
- 简洁的文本格式，便于调试

#### 解析实现

```rust
impl TryFrom<&str> for IconLayout {
    type Error = MyError;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut entries = Vec::new();
        
        for line in value.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue; // 跳过注释和空行
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                let entry = IconEntry {
                    name: parts[0].to_string(),
                    x: parts[1].parse()?,
                    y: parts[2].parse()?,
                };
                entries.push(entry);
            }
        }
        
        Ok(IconLayout {
            icon_count: entries.len(),
            entries,
        })
    }
}
```

### 3.4 布局比较和过滤

```rust
pub fn filter(&self, another: &IconLayout) -> (Vec<&IconEntry>, HashMap<&String, usize>) {
    let old_names: HashSet<&String> = another.entries.iter().map(|e| &e.name).collect();
    
    let mut name2idx: HashMap<&String, usize> = HashMap::new();
    for (i, e) in self.entries.iter().enumerate() {
        if old_names.contains(&&e.name) {
            name2idx.insert(&e.name, i);
        }
    }
    
    (
        self.entries
            .iter()
            .filter(|e| name2idx.contains_key(&e.name))
            .collect(),
        name2idx,
    )
}
```

**过滤逻辑**：

- 以图标名称为键进行匹配
- 返回交集图标和索引映射
- 支持增量布局更新
- 处理图标的增删情况

## 4. 错误处理

### 4.1 错误类型

```rust
use fastlink_core::types::err::{ErrorCode, MyError, MyResult};

fn win_err_to_myerr(e: windows::core::Error) -> MyError {
    MyError {
        code: ErrorCode::Unknown,
        msg: e.message(),
    }
}
```

### 4.2 常见错误场景

- **文件I/O错误**：布局文件读写失败
- **解析错误**：DSV格式解析失败
- **执行错误**：嵌入式程序执行失败
- **编码错误**：GBK编码转换失败
- **Windows API错误**：系统调用失败

## 5. 性能优化

### 5.1 重试机制

```rust
const GET_NEW_LAYOUT_MAX_RETRY: u32 = 6;

pub fn get_new_layout(last_layout: Option<&IconLayout>) -> MyResult<IconLayout> {
    for i in 0..GET_NEW_LAYOUT_MAX_RETRY {
        let layout = get_cur_layout()?;
        
        if let Some(last) = last_layout {
            if layout != *last {
                return Ok(layout);
            }
        } else {
            return Ok(layout);
        }
        
        if i < GET_NEW_LAYOUT_MAX_RETRY - 1 {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
}
```

## 6. 平台特性

### 6.1 Windows集成

```rust
use std::os::windows::process::CommandExt;
const _CREATE_NO_WINDOW: u32 = 0x08000000;

// 隐藏窗口执行
Command::new(&exe_path)
    .creation_flags(_CREATE_NO_WINDOW)
    .output()
```

### 6.2 编码处理

```rust
use encoding_rs::GBK;

let (decoded, _, had_errors) = GBK.decode(&output.stdout);
```

**编码特点**：

- 支持GBK编码的中文文件名
- 自动检测编码错误
- 兼容Windows系统默认编码

## 7. 依赖关系

### 7.1 内部依赖

- `fastlink-core`：错误处理和工具函数

### 7.2 外部依赖

- `chrono`：时间戳生成
- `encoding_rs`：GBK编码处理
- `windows`：Windows API调用
- `serde`：序列化支持（可选）
