use crate::{desktop_setter::types::state::DESKTOP_STATE, types::err::MyResult};

pub fn handle_desktop_state() -> MyResult<()> {
    let state = DESKTOP_STATE.state();

    log::info!("{}", state);
    Ok(())
}
