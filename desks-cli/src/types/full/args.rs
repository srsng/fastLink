use crate::types::commands::Commands;

#[derive(clap::Parser, Debug)]
#[command(
    version,
    long_about = r#"
    Windows平台下修改Desktop库目标文件夹, 使用符号链接指向已有的文件夹, 动态修改桌面内容
    > 声明：
    >     注意，不保证安全无bug，如果遇到问题，请积极提交issue。
    >     不支持多用户，初始化后请勿手动修改`桌面库`的位置、名称等，这些操作都有可能让你丢失你的桌面库。
    >
    > 如果有任何修改桌面库的需求，请使用`desks reset -k`来重置，修改之后再初始化，以防发生棘手的情况。
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
