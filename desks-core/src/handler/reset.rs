use std::path::PathBuf;

use crate::handler::fresh::handle_fresh_desktop;
use crate::{state::DESKTOP_STATE, utils::func::get_temp_path, utils::rollback::Transaction};
use crate::{ErrorCode, MyError, MyResult};
use fastlink_core::utils::path::get_path_type;

pub fn handle_desktop_reset(keep_usual_paths: Option<bool>) -> MyResult<()> {
    log::debug!("handle_desktop_reset");
    let (initial_path, initial_path_temp, cur_target) = {
        let state = DESKTOP_STATE.state_mut();
        (
            state.initial_path.clone(),
            state.initial_path_temp.clone(),
            state.cur_target.clone(),
        )
    };

    // todo: state文件被"备份后"，怎么reset
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
    } else if cur_target.is_none() {
        Err(MyError::new(
            ErrorCode::Unknown,
            "未知情况, cur_target为空".into(),
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
            desktop_reset(initial_path, initial_path_temp, cur_target.unwrap())?;
            {
                DESKTOP_STATE.reset(keep_usual_paths);
                DESKTOP_STATE.save()?;
            };
            log::info!("重置成功");
            handle_fresh_desktop();
            log::info!("桌面已刷新");
            Ok(())
        // 或者当initial_path_temp存在且为目录，initial_path不存在时 (其他情况`1`)
        } else if initial_path_temp_status.code == ErrorCode::TargetExistsAndNotLink
            && initial_path_temp.is_dir()
            && initial_path_status.code == ErrorCode::FileNotExist
        {
            // 把initial_path_temp重命名为initial_path
            desktop_reset_1(initial_path, initial_path_temp)?;
            {
                DESKTOP_STATE.reset(keep_usual_paths);
                DESKTOP_STATE.save()?;
            }
            log::info!("重置成功");
            handle_fresh_desktop();
            log::info!("桌面已刷新");
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
///
/// - `path`: 初始化时，原始的Desktop库位置，也是当前符号链接所在位置
/// - `temp`: 初始化时，原始的Desktop库被转移到的临时位置
/// - `cur_target`: 当前符号链接指向的位置，可能与temp相同
fn desktop_reset(path: PathBuf, temp: PathBuf, cur_target: PathBuf) -> MyResult<()> {
    let mut tx = Transaction::new();
    let path_temp = get_temp_path(&path);

    // 转移到当前符号链接path到临时路径
    tx.add_op_rename_dir(
        path.clone(),
        path_temp.clone(),
        Some("将桌面库位置符号链接重命名以backup".into()),
    )?;
    // 重命名desktop库临时目录回原Desktop库名
    tx.add_op_rename_dir(
        temp.clone(),
        path.clone(),
        Some("重命名临时目录回原Desktop库名".into()),
    )?;
    // 删除链接path_temp，由于链接指向的路径已经被重命名，temp又被移走，该步操作的undo将会创建一个指向路径不存在的符号链接
    tx.add_op_del_link_unsafe_dir(
        cur_target,
        path_temp,
        Some("删除转移到临时路径的、指向Desktop库临时路径的链接".into()),
    )?;

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

#[cfg(test)]
mod tests {
    #[test]
    fn test_1() {
        let path1 = r"D:\not_exists1";
        let path2 = r"D:\not_exists2";
        let res = std::os::windows::fs::symlink_dir(path1, path2);
        println!("{:?}", res);
    }
}
