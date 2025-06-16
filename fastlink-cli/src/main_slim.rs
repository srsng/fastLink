/// 与 main.rs 基本一致
pub mod utils;
use utils::func::special_warn;
pub mod types;
use crate::types::args::Args;
use clap::Parser;
pub use fastlink_core::types::err::{ErrorCode, MyError, MyResult};
use fastlink_core::types::link_task::LinkTask;

fn main() {
    let args: Args = Args::parse();

    // 初始化日志系统
    fastlink_core::utils::logs::LogIniter::new(args.quiet, args.debug, None).init();
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
