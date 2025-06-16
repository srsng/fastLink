#[derive(clap::Parser, Debug)]
#[command(
    version,
    long_about = r#"
Desks的子命令Usual的单独二进制，用于省那一个u跟空格
"#
)]
pub struct Args {
    /// 通过name快速切换为已设置的一些常用路径
    #[arg(required = true)]
    pub name: String,

    /// 只输出warn与error level的日志
    #[arg(short, long)]
    pub quiet: bool,

    /// 输出debug level的日志
    #[arg(long)]
    pub debug: bool,
}
