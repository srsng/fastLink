use std::path::{Path, PathBuf};

use crate::{
    handler::fresh::handle_fresh_desktop, state::DESKTOP_STATE, utils::func::get_temp_path,
    utils::rollback::Transaction,
};
use crate::{ErrorCode, MyError, MyResult};
use fastlink_core::utils::fs::mk_parents;

pub fn handle_desktop_set(
    new_desktop_dir_path: PathBuf,
    make_dir: bool,
    // overwrite_links: bool,
    usual: Option<String>,
) -> MyResult<()> {
    log::debug!("handle_desktop_set");
    // 经过validate_new_desktop_dir_path处理, 保证是目录或指向目录的符号链接
    let new_desktop_path: PathBuf = new_desktop_dir_path;

    let (initial, cur_path, cur_target) = {
        let state = DESKTOP_STATE.state();
        (
            state.initial_path.clone(),
            state.cur_path.clone(),
            state.cur_target.clone(),
        )
    };

    // 当新desktop与目前desktop为同一个时，跳过
    if cur_target.is_some() && Some(new_desktop_path.clone()) == cur_target {
        return Ok(());
    }

    if cur_path.is_none() {
        if initial.is_none() {
            Err(MyError::new(
                ErrorCode::InvalidInput,
                "请先使用init命令初始化".into(),
            ))
        } else {
            Err(MyError::new(
                ErrorCode::Unknown,
                "未知错误，当前Desktop库未知已丢失，使用state查询状态".into(),
            ))
        }
    } else {
        if make_dir {
            let res = mk_parents(&new_desktop_path)?;
            if res {
                log::info!("已创建 {} 的父目录", new_desktop_path.display())
            }
        } else {
            let parent = new_desktop_path.parent();
            if parent.is_none() || !parent.unwrap().exists() {
                return Err(MyError::new(
                    ErrorCode::ParentNotExist,
                    "输入路径的父目录不存在，若需自动创建请添加--mk参数重试".into(),
                ));
            }
        }

        desktop_set(
            new_desktop_path.clone(),
            cur_path.unwrap(),
            cur_target.unwrap(),
        )?;
        log::info!(
            "已设置 {} 作为Desktop，在桌面F5刷新或等待片刻以应用",
            new_desktop_path.display()
        );
        // 先添加usual不用path，但是无法set的话，添加了倒有问题
        desktop_add_usual(usual, &new_desktop_path)?;
        {
            let mut state = DESKTOP_STATE.state_mut();
            state.cur_target = Some(new_desktop_path.clone());
        }
        {
            DESKTOP_STATE.save()?;
        }
        handle_fresh_desktop();
        Ok(())
    }
    // Ok(())
}

/// 在cur_path位置覆写符号链接：由指向cur_target转为指向new_desktop_path
///
/// - `new_desktop_path`: 新的桌面库位置
/// - `cur_path`: 当前桌面库符号链接的位置
/// - `cur_target`: 当前桌面库符号链接指向的位置
///
/// 使用temp_cur_path转移cur_path，从而将op_del_link操作放置最后，
/// 防止op_del_link之前的op_mklink失败，难以恢复被删除的link
fn desktop_set(new_desktop_path: PathBuf, cur_path: PathBuf, cur_target: PathBuf) -> MyResult<()> {
    let mut tx = Transaction::new();

    let temp_cur_path = get_temp_path(&cur_path);
    tx.add_op_rename_dir(
        cur_path.clone(),
        temp_cur_path.clone(),
        Some("rename to backup cur_path".into()),
    )?;
    tx.add_op_mklink(
        new_desktop_path,
        cur_path,
        Some("创建新链接，指向Desktop位置".into()),
    )?;
    tx.add_op_del_link(cur_target, temp_cur_path, Some("".into()))?;
    tx.commit()
}

/// 成功添加时返回Ok(true)
fn desktop_add_usual(name: Option<String>, path: &Path) -> MyResult<bool> {
    if let Some(name) = name {
        let mut state = DESKTOP_STATE.state();

        if state.usual_paths.contains_key(&name) {
            Err(MyError::new(
                ErrorCode::InvalidInput,
                format!(
                    "名称 {} 已在已有列表中\n{} -> {}",
                    name,
                    name,
                    path.display()
                ),
            ))
        } else {
            state.usual_paths.insert(name.clone(), path.to_path_buf());
            log::info!("已添加常用快捷名称 {} -> {}", name, path.display());
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

// fn handle_mklink_pre_check_error(res: MyResult<()>, path: &Path) -> MyResult<()> {
//     if let Some(e) = res.err() {
//         match e.code {
//             ErrorCode::FileNotExist | ErrorCode::BrokenSymlink => Err(e),
//             ErrorCode::TargetExistsAndNotLink | ErrorCode::TargetLinkExists => {
//                 if path.is_dir() {
//                     Ok(())
//                 } else {
//                     Err(e)
//                 }
//             }
//             _ => Err(e),
//         }
//     } else {
//         Ok(())
//     }
// }
