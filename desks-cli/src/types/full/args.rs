use crate::types::commands::Commands;

#[derive(clap::Parser, Debug)]
#[command(
    version,
    long_about = r#"
    Windows平台下修改Desktop库目标文件夹, 使用符号链接指向已有的文件夹, 动态修改桌面内容
    声明：
        注意，不保证安全，有很多实际问题没有解决，如果遇到问题，请积极提交issue。
        不支持多用户，任何手动修改桌面库位置、名称等操作都可能让你丢失桌面库。
        Caution: this program is **UNSAFE**!
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
