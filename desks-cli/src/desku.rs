/// desks简化版本，将只能用于设定已有的快速名称
pub mod handler;
pub mod types;
pub mod utils;

use crate::handler::usual::handle_desktop_usual;
use crate::types::args::Args;
use crate::types::state::DESKTOP_STATE;
use clap::Parser;
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
    handle_desktop_usual(args.name)
}
