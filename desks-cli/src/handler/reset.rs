use std::path::PathBuf;

use crate::{types::state::DESKTOP_STATE, utils::rollback::Transaction};
use crate::{ErrorCode, MyError, MyResult};
use fastlink_core::utils::path::get_path_type;

pub fn handle_desktop_reset() -> MyResult<()> {
    let (initial_path, initial_path_temp) = {
        let state = DESKTOP_STATE.state_mut();
        (state.initial_path.clone(), state.initial_path_temp.clone())
    };

    // 两路径都为空
    if initial_path.is_none() && initial_path_temp.is_none() {
        log::warn!("未经过初始化");
        Ok(())
    // 有一个非空，一个空
    } else if initial_path.is_none() || initial_path_temp.is_none() {
        Err(MyError::new(
            ErrorCode::Unknown,
            format!(
                "未知情况，两路径应该均空或均有值: \ninitial_path={:?}\ninitial_path_temp={:?}",
                initial_path, initial_path_temp
            ),
        ))
    } else {
        let initial_path = initial_path.unwrap();
        let initial_path_temp = initial_path_temp.unwrap();
        let initial_path_status = get_path_type(&initial_path).err().unwrap();
        let initial_path_temp_status = get_path_type(&initial_path_temp).err().unwrap();

        // 仅当initial_path存在且为符号链接，initial_path_temp存在时重置
        if initial_path_status.code == ErrorCode::TargetLinkExists
            && (initial_path_temp_status.code == ErrorCode::TargetExistsAndNotLink
                || initial_path_temp_status.code == ErrorCode::TargetLinkExists)
        {
            desktop_reset(initial_path, initial_path_temp)?;
            {
                DESKTOP_STATE.reset();
                DESKTOP_STATE.save()?;
            };
            Ok(())
        } else {
            Err(MyError::new(
                ErrorCode::Unknown,
                "注册表中当前Desktop库路径不为符号链接，".into(),
            ))
        }
    }
    // else {
    //     Err(MyError::new(ErrorCode::Unknown, "未知情况".into()))
    // }
}

fn desktop_reset(path: PathBuf, temp: PathBuf) -> MyResult<()> {
    let mut tx = Transaction::new();

    tx.add_op_del_link(
        temp.clone(),
        path.clone(),
        Some("删除指向Desktop库临时路径的链接".into()),
    )?;
    tx.add_op_rename_dir(temp, path, Some("重命名临时目录回原Desktop库名".into()))?;

    tx.commit()
}
