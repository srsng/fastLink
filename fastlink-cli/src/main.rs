// todo: 编写tests
// todo：编写文档
// todo: 清理无用代码
// 添加子命令wallpaper/w，并且增加桌面绑定壁纸功能（不兼容wallpaper engine等软件，因此需要手动开启）
// 其他行内todo

use clap::Parser;

// 声明包
pub mod types;
pub mod utils;

// 公开错误
pub use fastlink_core::types::err::{ErrorCode, MyError, MyResult};

use crate::types::args::Args;
use fastlink_core::types::link_task::LinkTask;
use utils::func::special_warn;

fn main() {
    let args: Args = Args::parse();

    // 初始化日志系统
    fastlink_core::utils::logs::LogIniter::new(args.quiet, args.debug, args.save_log.clone())
        .init();
    log::debug!("{:?}", args);

    special_warn(&args);

    let task_res = LinkTask::try_from(&args);

    match task_res {
        Ok(task) => match task.work() {
            Ok(()) => (),
            Err(e) => e.log(),
        },
        Err(e) => e.log(),
    }
}
