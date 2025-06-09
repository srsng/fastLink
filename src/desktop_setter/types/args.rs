use crate::desktop_setter::types::commands::Commands;
// use crate::types::err::{ErrorCode, MyError};
// use path_clean::PathClean;

#[derive(clap::Parser, Debug)]
#[command(
    version,
    long_about = r#"
Windows平台下修改Desktop库目标文件夹, 使用符号链接指向已有的文件夹, 动态修改桌面内容
注意，不保证安全，有很多实际问题没有解决。
不支持多用户，任何手动修改桌面库位置、名称等操作都可能搞你的系统崩溃。
Caution: this program is UNSAFE!
"#
)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// 只输出warn与error level的日志
    #[arg(short, long)]
    pub quiet: bool,

    /// 输出debug level的日志
    #[arg(long)]
    pub debug: bool,
}
