use crate::state::DESKTOP_STATE;
use crate::utils::func::{get_dir_temp, get_original_desktop_path};
use crate::utils::rollback::Transaction;
use crate::{ErrorCode, MyError, MyResult};
use fastlink_core::utils::path::get_path_type;
use std::path::PathBuf;

pub fn handle_desktop_init() -> MyResult<bool> {
    // 获取当前Desktop库路径
    let desktop = get_original_desktop_path().map_err(|mut e| {
        e.msg = format!("获取当前Desktop库位置失败：{}", e.msg);
        e
    })?;
    let desktop_temp = get_dir_temp(&desktop)?;

    // 检查Desktop库与临时位置的性质
    // 部分情况需要纠正 todo
    let desktop_status = get_path_type(desktop.as_path()).err().unwrap();
    let desktop_temp_status = get_path_type(desktop_temp.as_path()).err().unwrap();

    // 一般情况：desktop为目录，而desktop_temp不存在
    if desktop_status.code == ErrorCode::TargetExistsAndNotLink
        && desktop.is_dir()
        && desktop_temp_status.code == ErrorCode::FileNotExist
    {
        log::info!("开始初始化");
        desktop_init(desktop.clone(), desktop_temp.clone()).map_err(|mut e| {
            e.msg = format!("初始化失败：{e}");
            e
        })?;
        // 保存状态 todo，纳入可以回滚操作
        {
            let mut state = DESKTOP_STATE.state_mut();
            state.initial_path = Some(desktop.clone());
            state.initial_path_temp = Some(desktop_temp.clone());
            state.cur_path = Some(desktop);
            state.cur_target = Some(desktop_temp);
        }
        {
            DESKTOP_STATE.save()?;
        }
        log::info!("初始化成功");
        Ok(true)
    // desktop不为目录
    } else if desktop_status.code == ErrorCode::TargetExistsAndNotLink && !desktop.is_dir() {
        Err(MyError::new(
            ErrorCode::Unknown,
            format!("你的Desktop库不是目录: {}", desktop.display()),
        ))
    // 已完成情况
    } else if desktop_status.code == ErrorCode::TargetLinkExists && desktop_temp.is_dir() {
        // todo: 检查状态，查看用户是否转移Desktop库，或有其他变动

        // 如果状态为空则修补
        if DESKTOP_STATE.state().initial_path.is_none() {
            // 保存状态
            {
                let mut state = DESKTOP_STATE.state_mut();
                state.initial_path = Some(desktop.clone());
                state.initial_path_temp = Some(desktop_temp.clone());

                let target = std::fs::read_link(&desktop);
                state.cur_path = Some(desktop);
                if let Ok(target) = target {
                    state.cur_target = Some(target);
                } else {
                    log::debug!(
                        "修补状态文件时，无法读取符号链接desktop指向: {}",
                        target.err().unwrap()
                    );
                    state.cur_target = Some(desktop_temp);
                }
            }
            {
                DESKTOP_STATE.save()?;
            }
            log::info!("初始化成功");
        } else {
            log::info!("已初始化，无需重复操作，若需重置，使用reset命令");
        }

        Ok(false)
    } else {
        log::warn!(
            "意外情况: \n{}\n{}\n{}\n{}",
            desktop.display(),
            desktop_status,
            desktop_temp.display(),
            desktop_temp_status
        );
        Err(MyError::new(
            ErrorCode::Unknown,
            "意外情况, 无法处理".to_string(),
        ))
    }
    // handle_path_type_res(path_type_res, desktop.as_path())?;
}

/// 初始化desktop_setter功能
fn desktop_init(desktop: PathBuf, temp: PathBuf) -> MyResult<()> {
    let mut tx = Transaction::new();
    tx.add_op_rename_dir(
        desktop.clone(),
        temp.clone(),
        Some("将Desktop库转到临时名称".into()),
    )?;
    tx.add_op_mklink(
        temp,
        desktop,
        Some("创建临时目录指向原始Desktop库的符号链接".into()),
    )?;
    tx.commit()
}

// fn handle_path_type_res(res: MyResult<()>, path: &Path) -> MyResult<()> {
//     match res {
//         Ok(_) => Ok(()),
//         Err(e) if e.code == ErrorCode::TargetExistsAndNotLink => Ok(()),
//         Err(mut e) if e.code == ErrorCode::TargetLinkExists => {
//             if path.is_dir() {
//                 Ok(())
//             } else {
//                 e.msg = format!("Desktop库是个符号链接，但指向的不是一个目录：{}", e.msg);
//                 Err(e)
//             }
//         }
//         Err(mut e) if e.code == ErrorCode::BrokenSymlink => {
//             e.msg = format!("Desktop库是损坏的符号链接：{}", e.msg);
//             Err(e)
//         }
//         Err(mut e) if e.code == ErrorCode::FileNotExist => {
//             e.msg = format!("Desktop库不存在：{}", e.msg);
//             Err(e)
//         }
//         Err(mut e) if e.code == ErrorCode::FileNotExist => {
//             e.msg = format!("Desktop库不存在：{}", e.msg);
//             Err(e)
//         }
//         Err(mut e) => {
//             e.msg = format!("检查桌面库失败：{}", e.msg);
//             Err(e)
//         }
//     }
// }
