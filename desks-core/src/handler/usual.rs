use crate::{
    handler::{set::handle_desktop_set, state::handle_desktop_state},
    state::DESKTOP_STATE,
};
use crate::{ErrorCode, MyError, MyResult};

pub fn handle_desktop_usual_setby(name: &String) -> MyResult<bool> {
    let usual_paths = { DESKTOP_STATE.state().usual_paths.clone() };
    if usual_paths.contains_key(name) {
        handle_desktop_set(usual_paths[name].clone(), false, None)
    } else {
        handle_desktop_state()?;
        Err(MyError::new(
            ErrorCode::InvalidInput,
            format!("名称 {} 不在已有列表中", name),
        ))
    }
}

pub fn handle_desktop_usual_del(name: &String) -> MyResult<bool> {
    let res = {
        let res = DESKTOP_STATE.del_usual_path_by_name(name);
        let b = res.is_some();
        if let Some(path) = res {
            DESKTOP_STATE.save()?;
            log::info!("成功删除常用路径 {name} - {}", path.display());
        }
        b
    };

    if res {
        Ok(true)
    } else {
        handle_desktop_state()?;
        Err(MyError::new(
            ErrorCode::InvalidInput,
            format!("名称 {} 不在已有列表中", name),
        ))
    }
}
