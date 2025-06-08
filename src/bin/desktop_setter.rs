use clap::Parser;
use fastlink::desktop_setter::handler::{
    init::handle_desktop_init, reset::handle_desktop_reset, set::handle_desktop_set,
    state::handle_desktop_state, usual::handle_desktop_usual,
};
use fastlink::desktop_setter::types::args::Args;
use fastlink::desktop_setter::types::commands::Commands;
use fastlink::desktop_setter::types::state::DESKTOP_STATE;
use fastlink::types::err::MyResult;

fn main() {
    let args: Args = Args::parse();

    // 初始化日志系统
    fastlink::utils::logs::init_log(args.quiet, args.debug);
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
        Commands::Origin | Commands::O => handle_desktop_init(),
        Commands::Usual { name } | Commands::U { name } => handle_desktop_usual(name),
    }
}
