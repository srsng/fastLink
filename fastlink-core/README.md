## fastlink-core

项目核心，包含`LinkTask`以及相关结构体定义，mklink以及相关函数定义，日志系统。  

先构建`LinkTaskArgs`（使用builder或from(Args)），经过`LinkTaskPre`预处理，得到`LinkTask`，
对`LinkTask`实例使用mklinks等方法可以创建、检查、删除符号链接