# fastLink
[![Windows release](https://github.com/srsng/fastLink/actions/workflows/windows-release.yml/badge.svg)](https://github.com/srsng/fastLink/actions/workflows/windows-release.yml)


A tool to make symlink fastly and smartly, support regex  
一个智能且方便的符号链接创建工具，支持regex，以及一些基于符号链接实现的程序(`desks`系列)

For Windows (at present)

如果你处于Windows的开发者模式，则不需要`sudo`或者`管理员权限`  
Do not need `sudo` or `administrator` if Developer Mode enabled.  

项目为一个workspace，包含多个crate，每个crate内都有各自的README.md

## fastlink-core
项目核心，包含`LinkTask`以及相关结构体定义，mklink以及相关函数定义，日志系统。  

先构建`LinkTaskArgs`（使用builder或from(Args)），经过`LinkTaskPre`预处理，得到`LinkTask`，
对`LinkTask`实例使用mklinks等方法可以创建、检查、删除符号链接。

## fastlink-cli
包含两个二进制: `fastlink`与`fastlink-slim`

用于在命令行创建符号链接，具体用法可以参考示例或`--help`

`fastlink-slim`不包含re支持与save_log  

两个二进制都可以使用可选参数`-c`检查路径，`--rm`删除已存在的符号链接

[fastlink-help](./fastlink-cli/README.md#fastlink-help)

# Desks 系列
包含两个命令行工具与一个托盘程序。

描述
```
    Windows平台下修改Desktop库目标文件夹, 使用符号链接指向已有的文件夹, 动态修改桌面内容
    声明：
        注意，不保证安全，有很多实际问题没有解决，如果遇到问题，请积极提交issue。
        不支持多用户，任何手动修改桌面库位置、名称等操作都可能让你丢失桌面库。
        Caution: this program is **UNSAFE**!
```

支持保持桌面布局


场景示例：
- 桌面文件/文件夹众多，想要工作、休闲分离
- 有一些文件/文件夹想放桌面，但是平时又不想让其他人看见
- 换一个桌面，即刻切换你跟电脑的状态

## desks-core

**desks**系列(desks, desku, desks-tray)的核心，主要定义了一些处理方法以及一个状态数据文件。


## desks-cli
包含两个二进制`desks`与`desku`，均为命令行工具。

`desks` 目前是desks系列的唯一的全功能程序，包含初始化、设置桌面、添加常用路径、重置等。

`desku` 只能通过以设置的常用路径来设置桌面，等效`desks u some_name`（就是为了省2个字）

[desks-help](./desks-cli/README.md#desks-help)

## desks-tray
相当于`desku`的托盘版本，但多一个original，且使用更方便

已经运行的desks-tray不会反复读取desks系列共用的状态文件，desks修改状态文件desks-tray不会即时相应，需要重启

托盘最多显示10个快捷名称/常用路径, 托盘菜蛋右侧的快捷键暂不可用

[desks-tray usage](./desks-tray/README.md#usage)


## desktop-layout
实现保存、恢复桌面图标布局

读取当前布局信息暂时使用python实现，再打包到exe由rust执行二进制

(python打包已经纳入在GitHub Action Workflow中)

## Feedback
如果发生了一些预期之外的问题，提个issue，记得贴上带`--debug`参数时的日志。

If something not excepet happened, open an issue and paste the log wiht `--debug` argument.  


如果是`desks`系列程序(desks, desku, desks-tray)，还需要使用`desks state`， 并使用`fastlink -c $initial_path$`与`fastlink -c $initial_path_temp$`
（手动替换$initial_path$与$initial_path_temp$），将返回信息一并提交到issue内

注：若无法使用`desks state`, 请搜索路径`fastlink\desktop_setter\state.json`获取程序状态
