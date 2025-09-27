# fastLink 用户使用指南

## 1. 项目简介

### 1.1 什么是 fastLink

fastLink 是一个智能的符号链接管理系统和桌面环境管理工具，专为 Windows 用户设计。它提供了两个核心功能：

- **智能符号链接管理**：快速、智能地创建和管理符号链接
- **桌面环境切换**：轻松切换不同的桌面布局和图标排列

### 1.2 主要特性

- ✅ **智能符号链接创建**：支持正则表达式批量创建
- ✅ **桌面环境管理**：快速切换不同桌面布局
- ✅ **图标布局保存**：自动保存和恢复桌面图标位置
- ✅ **命令行界面**：强大的CLI工具
- ✅ **系统托盘集成**：便捷的图形化操作
- ✅ **中文支持**：完整的中文界面和文档

### 1.3 适用场景

- **开发者**：管理项目文件的符号链接
- **设计师**：切换不同工作环境的桌面布局
- **系统管理员**：批量管理文件和目录链接
- **普通用户**：整理和管理桌面环境

## 2. 安装和配置

### 2.1 系统要求

- **操作系统**：Windows 10/11 (x64)
- **权限要求**：管理员权限（用于创建符号链接）
- **磁盘空间**：约 50MB

### 2.2 下载安装

#### 方式一：从 GitHub Releases 下载

1. 访问 [fastLink Releases](https://github.com/your-repo/fastLink/releases)
2. 下载最新版本的 `fastlink-windows-x64.zip`
3. 解压到任意目录（建议：`C:\Program Files\fastLink`）
4. 将解压目录添加到系统 PATH 环境变量

#### 方式二：从源码编译

```bash
# 克隆仓库
git clone https://github.com/your-repo/fastLink.git
cd fastLink

# 编译发布版本
cargo build --release

# 编译结果在 target/release/ 目录
```

### 2.3 环境配置

#### 添加到 PATH 环境变量

1. 右键「此电脑」→「属性」→「高级系统设置」
2. 点击「环境变量」
3. 在「系统变量」中找到「Path」，点击「编辑」
4. 点击「新建」，添加 fastLink 安装目录
5. 点击「确定」保存

#### 验证安装

```bash
# 检查 fastlink-cli
fastlink-cli --version

# 检查 desks-cli
desks-cli --help
```

## 3. fastlink-cli 使用指南

### 3.1 基础用法

#### 创建简单符号链接

```bash
# 基本语法
fastlink-cli <源路径> [目标路径]

# 示例：创建文件符号链接
fastlink-cli "C:\Users\用户名\Documents\重要文件.txt" "D:\快捷访问\重要文件.txt"

# 示例：创建目录符号链接
fastlink-cli "C:\Projects\MyProject" "D:\工作区\MyProject"
```

#### 自动目标路径

```bash
# 如果不指定目标路径，会在当前目录创建同名链接
fastlink-cli "C:\Source\file.txt"
# 等同于
fastlink-cli "C:\Source\file.txt" "./file.txt"
```

### 3.2 高级功能

#### 保留文件扩展名

```bash
# 使用 -k 或 --keep-extention 参数
fastlink-cli -k "C:\app.exe" "D:\shortcuts\app"
# 结果：D:\shortcuts\app.exe
```

#### 自动创建目录

```bash
# 使用 --make-dir 参数自动创建不存在的目录
fastlink-cli --make-dir "C:\source.txt" "D:\new\path\target.txt"
```

#### 正则表达式批量操作

```bash
# 匹配所有 .txt 文件
fastlink-cli --regex ".*\.txt$" "C:\Documents" "D:\TextFiles"

# 匹配特定模式的文件
fastlink-cli --regex "^project_.*\.log$" "C:\Logs" "D:\ProjectLogs"

# 设置匹配深度
fastlink-cli --regex ".*\.dll$" --re-max-depth 3 "C:\Program Files" "D:\DLLs"
```

#### 过滤选项

```bash
# 只处理文件
fastlink-cli --only-file --regex ".*" "C:\Mixed" "D:\FilesOnly"

# 只处理目录
fastlink-cli --only-dir --regex ".*" "C:\Mixed" "D:\DirsOnly"

# 扁平化输出（不保持目录结构）
fastlink-cli --regex ".*\.txt$" --re-output-flatten "C:\Nested" "D:\Flat"
```

### 3.3 链接管理

#### 检查文件状态

```bash
# 检查单个文件
fastlink-cli --check "C:\some\file.txt"

# 检查目录中的所有文件
fastlink-cli --check --regex ".*" "C:\directory"
```

#### 删除符号链接

```bash
# 删除单个链接
fastlink-cli --rm "D:\link.txt"

# 批量删除匹配的链接
fastlink-cli --rm --regex ".*\.lnk$" "D:\shortcuts"
```

#### 覆盖和跳过选项

```bash
# 覆盖已存在的链接
fastlink-cli --overwrite-links "C:\source" "D:\target"

# 跳过已存在的链接
fastlink-cli --skip-exist-links "C:\source" "D:\target"

# 覆盖损坏的链接（默认开启）
fastlink-cli --overwrite-broken-link "C:\source" "D:\target"
```

### 3.4 实用示例

#### 场景一：开发环境配置

```bash
# 将配置文件链接到项目目录
fastlink-cli "C:\Users\用户名\.vscode\settings.json" "D:\Project\.vscode\settings.json" --make-dir

# 批量链接开发工具
fastlink-cli --regex ".*\.(exe|dll)$" "C:\DevTools" "D:\Project\tools" --make-dir
```

#### 场景二：文档管理

```bash
# 创建文档快捷访问
fastlink-cli --regex ".*\.(pdf|docx|xlsx)$" "C:\Documents" "D:\QuickAccess\Docs" --make-dir

# 保持原始扩展名
fastlink-cli -k --regex ".*" "C:\ImportantFiles" "D:\Desktop\Important" --make-dir
```

#### 场景三：游戏存档管理

```bash
# 链接游戏存档到云盘
fastlink-cli "C:\Users\用户名\AppData\Local\GameName\Saves" "D:\OneDrive\GameSaves\GameName" --make-dir
```

## 4. desks-cli 使用指南

### 4.1 初始化设置

#### 首次使用

```bash
# 初始化桌面管理系统
desks-cli init
```

这个命令会：

- 备份当前桌面路径
- 创建状态管理文件
- 初始化配置目录

#### 查看当前状态

```bash
# 显示当前桌面状态
desks-cli state
```

输出示例：

```text
当前桌面状态：

- 原始桌面：C:\Users\用户名\Desktop
- 当前桌面：D:\WorkDesktop
- 常用桌面：
  - work: D:\WorkDesktop
  - personal: D:\PersonalDesktop
  - gaming: D:\GamingDesktop
```

### 4.2 桌面切换

#### 设置新桌面

```bash
# 切换到指定目录作为桌面
desks-cli set "D:\WorkDesktop"

# 如果目录不存在，自动创建
desks-cli set "D:\NewDesktop" --make-dir

# 同时保存为常用桌面
desks-cli set "D:\WorkDesktop" --usual work
```

#### 恢复原始桌面

```bash
# 恢复到系统原始桌面
desks-cli original
```

#### 使用常用桌面

```bash
# 切换到已保存的常用桌面
desks-cli usual work
desks-cli usual personal
desks-cli usual gaming
```

### 4.3 常用桌面管理

#### 添加常用桌面

```bash
# 方式一：在设置桌面时同时保存
desks-cli set "D:\ProjectDesktop" --usual project

# 方式二：将当前桌面保存为常用桌面
# （需要先切换到目标桌面，然后重新设置并保存）
desks-cli set "D:\CurrentDesktop" --usual current
```

#### 删除常用桌面

```bash
# 删除指定的常用桌面配置
desks-cli del-usual work
```

注意：这只会删除常用桌面的配置记录，不会删除实际的目录。

### 4.4 重置和恢复

#### 完全重置

```bash
# 重置所有设置，恢复到初始状态
desks-cli reset
```

这会：

- 恢复到原始桌面
- 清除所有常用桌面配置
- 重置状态文件

#### 保留常用桌面的重置

```bash
# 重置但保留常用桌面配置
desks-cli reset --keep-usual-paths
```

### 4.5 实用场景

#### 场景一：工作环境切换

```bash
# 设置工作桌面
desks-cli set "D:\Workspaces\Office" --make-dir --usual office

# 设置开发桌面
desks-cli set "D:\Workspaces\Development" --make-dir --usual dev

# 快速切换
desks-cli usual office  # 切换到办公环境
desks-cli usual dev     # 切换到开发环境
desks-cli original      # 回到个人桌面
```

#### 场景二：项目管理

```bash
# 为不同项目创建专用桌面
desks-cli set "D:\Projects\ProjectA\Desktop" --make-dir --usual projecta
desks-cli set "D:\Projects\ProjectB\Desktop" --make-dir --usual projectb

# 项目间快速切换
desks-cli usual projecta
desks-cli usual projectb
```

#### 场景三：临时桌面

```bash
# 创建临时工作桌面
desks-cli set "C:\Temp\TempDesktop" --make-dir

# 完成后恢复
desks-cli original
```

## 5. desks-tray 系统托盘工具

### 5.1 启动托盘工具

```bash
# 启动系统托盘程序
desks-tray
```

程序启动后会在系统托盘显示图标，提供图形化的桌面切换功能。

### 5.2 托盘菜单功能

右键点击托盘图标，可以看到以下菜单：

- **当前状态**：显示当前桌面信息
- **原始桌面**：快速恢复到系统原始桌面
- **常用桌面**：列出所有已保存的常用桌面
- **设置新桌面**：打开目录选择对话框
- **退出**：关闭托盘程序

### 5.3 使用技巧

- **开机自启**：将 `desks-tray.exe` 添加到开机启动项
- **快捷键**：可以通过第三方工具为托盘操作设置全局快捷键
- **状态指示**：托盘图标会根据当前桌面状态显示不同样式

## 6. 桌面图标布局管理

### 6.1 自动布局保存

desks-cli 在切换桌面时会自动：

- 保存当前桌面的图标布局
- 恢复目标桌面的图标布局

### 6.2 布局文件位置

布局文件保存在：

```text
D:\Links\           # 统一的链接存放目录
├── Documents\      # 文档链接
├── Projects\       # 项目链接
├── Tools\          # 工具链接
└── Games\          # 游戏链接
```

#### 命名规范

- 使用有意义的名称
- 保持一致的命名风格
- 避免特殊字符和空格

#### 备份策略

```bash
# 定期备份重要的符号链接配置
# 可以创建批处理脚本自动化这个过程
```

### 8.2 桌面环境管理

#### 桌面分类

- **工作桌面**：只放工作相关的文件和快捷方式
- **个人桌面**：个人文件和娱乐应用
- **项目桌面**：特定项目的相关文件
- **临时桌面**：临时工作和测试

#### 目录结构建议

```txt
D:\Desktops
├── Work\           # 工作桌面
├── Personal\       # 个人桌面
├── Projects\       # 项目桌面
│   ├── ProjectA 
│   └── ProjectB 
└── Temp\           # 临时桌面
```

### 8.3 性能优化

#### 避免深层嵌套

```bash
# 限制正则匹配深度
fastlink-cli --regex ".*" --re-max-depth 3 <源> <目标>
```

#### 批量操作

```bash
# 一次性处理多个文件，而不是逐个处理
fastlink-cli --regex ".*\.(txt|doc|pdf)$" <源目录> <目标目录>
```

## 9. 故障排除

### 9.1 常见错误

#### 权限不足

**错误信息**："拒绝访问" 或 "权限不足"

**解决方案**：

1. 以管理员身份运行命令提示符
2. 确保对源文件和目标目录有足够权限

#### 路径不存在

**错误信息**："系统找不到指定的路径"

**解决方案**：

1. 检查路径拼写是否正确
2. 使用 `--make-dir` 参数自动创建目录
3. 使用绝对路径而不是相对路径

#### 符号链接创建失败

**错误信息**："无法创建符号链接"

**解决方案**：

1. 确保目标位置没有同名文件
2. 检查磁盘空间是否充足
3. 使用 `--overwrite-links` 覆盖已存在的链接

### 9.2 桌面切换问题

#### 桌面切换失败

**可能原因**：

- 目标目录不存在
- 权限不足
- 桌面正在被其他程序使用

**解决方案**：

1. 使用 `desks-cli state` 检查当前状态
2. 确保目标目录存在且可访问
3. 关闭可能占用桌面的程序

#### 图标布局丢失

**可能原因**：

- 布局文件损坏
- 图标文件被移动或删除

**解决方案**：

1. 检查布局文件是否存在
2. 重新排列图标并切换桌面以保存新布局

### 9.3 获取帮助

#### 命令行帮助

```bash
# 查看完整帮助
fastlink-cli --help
desks-cli --help

# 查看子命令帮助
desks-cli set --help
```

#### 调试信息

```bash
# 启用调试模式获取详细信息
fastlink-cli --debug <命令>
desks-cli --debug <命令>
```

#### 社区支持

- GitHub Issues：报告 bug 和功能请求
- 文档：查看最新的技术文档
- 示例：参考项目中的示例代码

## 10. 高级用法

### 10.1 批处理脚本

#### 创建批处理文件

```batch
@echo off
REM 工作环境设置脚本
echo 设置工作环境...
desks-cli set "D:\Work\Desktop" --usual work
fastlink-cli --regex ".*\.(exe|lnk)$" "C:\WorkTools" "D:\Work\Desktop\Tools" --make-dir
echo 工作环境设置完成！
pause
```

#### 自动化任务

```batch
REM 每日备份脚本
fastlink-cli --regex ".*\.(doc|docx|xlsx|pdf)$" "C:\Users\%USERNAME%\Documents" "D:\Backup\%DATE%" --make-dir
```

### 10.2 配置文件管理

虽然 fastLink 主要通过命令行参数配置，但你可以创建配置脚本：

```batch
REM config.bat - 常用配置
set FASTLINK_SOURCE=C:\Projects
set FASTLINK_TARGET=D:\Links\Projects
set FASTLINK_OPTIONS=--make-dir --keep-extention

fastlink-cli %FASTLINK_OPTIONS% "%FASTLINK_SOURCE%" "%FASTLINK_TARGET%"
```

### 10.3 与其他工具集成

#### PowerShell 集成

```powershell
# PowerShell 函数封装
function Set-WorkDesktop {
    param([string]$Name)
    desks-cli usual $Name
    Write-Host "已切换到工作桌面: $Name" -ForegroundColor Green
}

# 使用
Set-WorkDesktop "development"
```

#### 任务计划程序

1. 打开「任务计划程序」
2. 创建基本任务
3. 设置触发器（如登录时）
4. 设置操作为运行 `desks-tray.exe`

## 11. 更新和维护

### 11.1 版本更新

1. 下载新版本
2. 备份当前配置
3. 替换可执行文件
4. 验证功能正常

### 11.2 配置备份

```bash
# 备份桌面状态文件
copy "%APPDATA%\fastlink\desktop-state.json" "D:\Backup\desktop-state.json"

# 备份布局文件
xcopy "%APPDATA%\fastlink\desktop-layout" "D:\Backup\desktop-layout" /E /I
```

### 11.3 清理和重置

```bash
# 完全重置桌面管理
desks-cli reset

# 手动清理配置文件
rmdir /S "%APPDATA%\fastlink"
```

---

## 附录

### A. 命令参考

#### fastlink-cli 参数列表

| 参数 | 简写 | 说明 |
|------|------|------|
| `--check` | `-c` | 检查模式 |
| `--rm` | | 删除模式 |
| `--keep-extention` | `-k` | 保留扩展名 |
| `--make-dir` | | 自动创建目录 |
| `--regex` | `--re` | 正则表达式匹配 |
| `--re-max-depth` | | 匹配最大深度 |
| `--only-file` | `-F` | 只处理文件 |
| `--only-dir` | `-D` | 只处理目录 |
| `--overwrite-links` | | 覆盖已存在链接 |
| `--skip-exist-links` | | 跳过已存在链接 |
| `--quiet` | `-q` | 安静模式 |
| `--debug` | | 调试模式 |

#### desks-cli 命令列表

| 命令 | 说明 |
|------|------|
| `init` | 初始化桌面管理 |
| `state` | 显示当前状态 |
| `set <path>` | 设置新桌面 |
| `original` | 恢复原始桌面 |
| `usual <name>` | 切换常用桌面 |
| `del-usual <name>` | 删除常用桌面 |
| `reset` | 重置所有设置 |

### B. 文件位置

| 类型 | 位置 |
|------|------|
| 桌面状态 | `%APPDATA%\fastlink\desktop-state.json` |
| 图标布局 | `%APPDATA%\fastlink\desktop-layout\*.dsv` |
| 日志文件 | 用户指定位置 |
| 临时文件 | `%TEMP%\fastlink\` |

### C. 错误代码

| 代码 | 说明 |
|------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 权限错误 |
| 3 | 路径错误 |
| 4 | 文件操作错误 |
| 5 | 网络错误 |

---

*本指南基于 fastLink v1.0，如有疑问请参考最新文档或提交 Issue。*
