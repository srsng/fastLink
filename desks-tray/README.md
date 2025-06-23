## desks-tray
不只是`desku`的托盘版本，支持手动保存（备份）、恢复桌面布局。

已经运行的desks-tray不会反复读取desks系列共用的状态文件，desks修改状态文件desks-tray不会即时相应，需要重启

托盘最多显示10个快捷名称/常用路径, 托盘菜蛋右侧的快捷键暂不可用

## 问题
1. 有时获取最新桌面布局时会卡住


### Usage

先通过`desks.exe`初始化(`init`)、添加常用路径(`set` some/path/to/dir -u some_name)，

然后双击`desks-tray.exe`或其他方式启动(单例，不可多开)。

### 其他特性

1. 单击托盘图标打开菜单，点击常用路径名称来切换桌面，也可以按下名称前的数字

2. 双击托盘图标切换原始桌面, 相当于`desks original`

3. 在`desks-tray.exe`所在目录，如果有icon.png或icon.ico，会自动读取作为图标，优先ico格式
    - 注意，部分图标，可能是图片分辨率较大或不是正方形等原因，无法读取作为icon

4. 在AppData `fastlink\desktop_setter`中查看日志文件
