use crate::{
    desktop_setter::{handler::set::handle_desktop_set, types::state::DESKTOP_STATE},
    types::err::MyResult,
};

pub fn handle_desktop_origin() -> MyResult<()> {
    let (original_desktop, initial_path) = {
        let state = DESKTOP_STATE.state();
        (state.initial_path_temp.clone(), state.initial_path.clone())
    };

    if original_desktop.is_none() && initial_path.is_none() {
        log::info!("请先执行init初始化后再使用其他命令");
        Ok(())
    } else {
        handle_desktop_set(original_desktop.unwrap(), false, None)
    }
}
