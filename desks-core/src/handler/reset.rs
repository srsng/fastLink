use std::path::PathBuf;

use crate::{state::DESKTOP_STATE, utils::rollback::Transaction};
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
        // 或者当initial_path_temp存在且为目录，initial_path不存在时 (其他情况`1`)
        } else if initial_path_temp_status.code == ErrorCode::TargetExistsAndNotLink
            && initial_path_temp.is_dir()
            && initial_path_status.code == ErrorCode::FileNotExist
        {
            // 把initial_path_temp重命名为initial_path
            desktop_reset_1(initial_path, initial_path_temp)?;
            {
                DESKTOP_STATE.reset();
                DESKTOP_STATE.save()?;
            }

            Ok(())
        } else {
            Err(MyError::new(
                ErrorCode::Unknown,
                "重置失败，请使用state命令查询状态并向开发者反馈".into(),
            ))
        }
    }
    // else {
    //     Err(MyError::new(ErrorCode::Unknown, "未知情况".into()))
    // }
}

/// 正常情况下重置程序
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

/// 其他情况`1`重置程序
///
/// 情况：initial_path_temp存在且为目录，initial_path不存在
///
/// 解决方式：把initial_path_temp重命名为initial_path
fn desktop_reset_1(path: PathBuf, temp: PathBuf) -> MyResult<()> {
    let mut tx = Transaction::new();
    tx.add_op_rename_dir(temp, path, Some("重命名临时目录回原Desktop库名".into()))?;
    tx.commit()
}
