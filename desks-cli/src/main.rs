pub mod types;

use crate::types::args::Args;
use crate::types::commands::Commands;
use clap::Parser;
use desks_core::handler::usual::handle_desktop_usual_del;
use desks_core::handler::{
    init::handle_desktop_init, original::handle_desktop_origin, reset::handle_desktop_reset,
    set::handle_desktop_set, state::handle_desktop_state, usual::handle_desktop_usual_setby,
};
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
    match args.command {
        Commands::Init => handle_desktop_init(),
        Commands::Reset { keep_usual_paths } => handle_desktop_reset(Some(keep_usual_paths)),
        Commands::Set {
            new_desktop_dir_path,
            make_dir,
            usual,
        } => handle_desktop_set(new_desktop_dir_path, make_dir, usual),
        Commands::State => handle_desktop_state(),
        Commands::Original => handle_desktop_origin(),
        Commands::Usual { name } => handle_desktop_usual_setby(name),
        Commands::DelUsual { name } => handle_desktop_usual_del(name),
    }
}
