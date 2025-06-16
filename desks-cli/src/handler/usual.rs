use crate::{
    handler::{set::handle_desktop_set, state::handle_desktop_state},
    types::state::DESKTOP_STATE,
};
use crate::{ErrorCode, MyError, MyResult};

pub fn handle_desktop_usual(name: String) -> MyResult<()> {
    let usual_paths = { DESKTOP_STATE.state().usual_paths.clone() };
    if usual_paths.contains_key(&name) {
        handle_desktop_set(usual_paths[&name].clone(), false, None)
    } else {
        handle_desktop_state()?;
        Err(MyError::new(
            ErrorCode::InvalidInput,
            format!("名称 {} 不在已有列表中", name),
        ))
    }
}
