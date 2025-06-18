/// desks简化版本，将只能用于设定已有的快速名称
pub mod types;

use crate::types::args::Args;
use clap::Parser;
use desks_core::handler::usual::handle_desktop_usual_setby;
use desks_core::state::DESKTOP_STATE;
pub use fastlink_core::types::err::{ErrorCode, MyError, MyResult};

fn main() {
    let args: Args = Args::parse();

    // 初始化日志系统
    fastlink_core::utils::logs::LogIniter::new(args.quiet, args.debug, None).init();
    log::debug!("{:?}", args);

    {
        let _state = DESKTOP_STATE.state();
    }

    if let Err(e) = handle_desktop_setter(args) {
        e.log()
    }
}

fn handle_desktop_setter(args: Args) -> MyResult<()> {
    handle_desktop_usual_setby(args.name)
}
