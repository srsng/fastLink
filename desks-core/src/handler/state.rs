use crate::state::DESKTOP_STATE;
use crate::MyResult;

pub fn handle_desktop_state() -> MyResult<()> {
    let state = DESKTOP_STATE.state();

    log::info!("{}", state);
    Ok(())
}
