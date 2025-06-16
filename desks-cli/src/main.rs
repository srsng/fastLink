pub mod handler;
pub mod types;
pub mod utils;

use crate::handler::{
    init::handle_desktop_init, original::handle_desktop_origin, reset::handle_desktop_reset,
    set::handle_desktop_set, state::handle_desktop_state, usual::handle_desktop_usual,
};
use crate::types::args::Args;
use crate::types::commands::Commands;
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
    match args.command {
        Commands::Init => handle_desktop_init(),
        Commands::Reset => handle_desktop_reset(),
        Commands::Set {
            new_desktop_dir_path,
            make_dir,
            usual,
        } => handle_desktop_set(new_desktop_dir_path, make_dir, usual),
        Commands::State => handle_desktop_state(),
        Commands::Original => handle_desktop_origin(),
        Commands::Usual { name } => handle_desktop_usual(name),
    }
}
