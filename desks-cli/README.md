## desks-cli
包含两个二进制`desks`与`desku`，均为命令行工具。

`desks` 目前是desks系列的唯一的全功能程序，包含初始化、设置桌面、添加常用路径、重置等。

`desku` 只能通过以设置的常用路径来设置桌面，等效`desks u some_name`（就是为了省2个字）

使用`desku`前，先通过`desks.exe`初始化(`init`)、添加常用路径(`set` some/path/to/dir -u some_name)


### desks help
```
    Windows平台下修改Desktop库目标文件夹, 使用符号链接指向已有的文件夹, 动态修改桌面内容
    声明：
        注意，不保证安全，有很多实际问题没有解决，如果遇到问题，请积极提交issue。
        不支持多用户，任何手动修改桌面库位置、名称等操作都可能让你丢失桌面库。
        Caution: this program is **UNSAFE**!


Usage: desks.exe [OPTIONS] <COMMAND>

Commands:
  init       初始化，可以使用reset恢复
  set        设置一个路径为桌面库，必须是一个目录或指向目录的符号链接
  state      获取当前状态
  original   设置Desktop库为原始的目录 [aliases: o, ori]
  usual      快速切换为通过set --usual <name>设置的一些常用路径，使用state命令查看已设置列表 [aliases: u, switch]
  del-usual  通过名称删除已存在的常用路径 [aliases: del]
  reset      重置所有数据，并将Desktop库恢复为原始状态，可使用-k保留常用路径数据
  help       Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet
          只输出warn与error level的日志

      --debug
          输出debug level的日志

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## feedback
需要注意的是，程序目前能处理的异常情况有限，如果遇到任何无法reset或init的意料之外的错误，
请使用`desks state`， 并使用`fastlink -c $initial_path$`与`fastlink -c $initial_path_temp$`
（手动替换$initial_path$与$initial_path_temp$），将返回信息一并提交到issue内

注：若无法使用`desks state`, 请搜索路径`fastlink\desktop_setter\state.json`获取程序状态
