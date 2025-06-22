use crate::MyResult;
use desks_core::utils::func::get_real_desktop_path;
use desktop_layout::{
    handler::{restore_desktop_layout_from_dsv, store_cur_layout_to_dsv},
    utils::{get_layout_data_dir_path, get_layout_data_file_path},
};
use rfd::FileDialog;
use std::path::{Path, PathBuf};

/// 在当前桌面对应的dsv数据存放目录打开dsv文件选择对话框
pub fn ask_select_dsv_file_at_appdata_dir<P: AsRef<Path>>(
    desk_dir: P,
) -> MyResult<Option<PathBuf>> {
    let data_dir = get_layout_data_dir_path(desk_dir)?;
    let res = FileDialog::new()
        .set_title("选择dsv文件以恢复")
        .add_filter("dsv Files", &["dsv"])
        .set_directory(data_dir)
        .pick_file();

    log_select(res)
}

pub fn ask_select_dsv_file() -> MyResult<Option<PathBuf>> {
    let res = FileDialog::new()
        .set_title("选择dsv文件以恢复")
        .add_filter("dsv Files", &["dsv"])
        .pick_file();

    log_select(res)
}

// /// 选择一个目录
// pub fn ask_select_dir() -> MyResult<Option<PathBuf>> {
//     let dir = rfd::FileDialog::new()
//         .set_title("选择存放数据的文件夹")
//         .pick_folder();

//     log_select(dir)
// }

pub fn ask_select_dsv_to_save() -> MyResult<Option<PathBuf>> {
    let res = rfd::FileDialog::new()
        .set_title("保存桌面布局数据到dsv文件")
        .add_filter("dsv Files", &["dsv"])
        .save_file();
    log_select(res)
}

fn log_select(res: Option<PathBuf>) -> MyResult<Option<PathBuf>> {
    match res {
        Some(path) => {
            log::debug!("Selected : {:?}", path.display());
            Ok(Some(path))
        }
        None => {
            log::debug!("Nothing selected");
            Ok(None)
        }
    }
}

/// 弹窗给用户选择文件并恢复桌面布局
///
/// # Return
/// - Ok(true) 成功
/// - Ok(false) 未选择文件
/// - Err(e) 出错
pub fn handle_ask_restore_layout_from_dsv() -> MyResult<bool> {
    // 获取需要读取的dsv文件路径
    let path = ask_select_dsv_file()?;

    match path {
        None => Ok(false),
        Some(path) => restore_desktop_layout_from_dsv(path, None),
    }
}

/// 在当前桌面对应的dsv数据存放目录打开dsv文件选择对话框，并恢复桌面布局
pub fn handle_ask_restore_layout_at_backup_dir() -> MyResult<bool> {
    // 获取当前桌面目标路径
    let desk_dir = get_real_desktop_path()?;
    // 获取需要读取的dsv文件路径
    let path = ask_select_dsv_file_at_appdata_dir(desk_dir)?;

    match path {
        None => Ok(false),
        Some(path) => restore_desktop_layout_from_dsv(path, None),
    }
}

/// 处理用户手动保存当前桌面数据挖掘到某一路径
pub fn handle_ask_save_layout_to() -> MyResult<bool> {
    let file = ask_select_dsv_to_save()?;
    match file {
        Some(file) => store_cur_layout_to_dsv(file).map(|_| true),
        None => Ok(false),
    }
}

/// 处理快速将当前桌面布局保存到数据文件夹，保存文件名以 %时间%.dsv结尾
pub fn handle_quick_bakcup_cur_layout() -> MyResult<bool> {
    // 获取当前桌面目标路径
    let desk_dir = get_real_desktop_path()?;
    // 获取保存dsv文件路径
    let path = get_layout_data_file_path(desk_dir, Some(true))?;
    store_cur_layout_to_dsv(path)?;
    Ok(true)
}
