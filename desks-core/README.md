## desks-core

**desks**系列(desks, desku, desks-tray)的核心，主要定义了一些处理方法以及一个状态数据文件。

需要注意的是，程序目前能处理的异常情况有限，如果遇到任何无法reset或init的意料之外的错误，
请使用`desks state`， 并使用`fastlink -c $initial_path$`与`fastlink -c $initial_path_temp$`
（手动替换initial_path与initial_path_temp），将返回信息一并提交到issue内

注：若无法使用`desks state`, 请搜索路径`fastlink\desktop_setter\state.json`获取程序状态
