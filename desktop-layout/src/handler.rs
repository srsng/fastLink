use crate::layout::{read_layout_from_path, IconLayout};
use crate::layout2::set_icon_layout;
use crate::utils::get_layout_data_file_path;
use crate::{ErrorCode, MyError, MyResult};
use fastlink_core::utils::fs::mk_parents;

use encoding_rs::GBK;

use std::fs::File;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

// windows隐藏命令行窗口参数
const _CREATE_NO_WINDOW: u32 = 0x08000000;
// 嵌入py打包的获取桌面图标布局信息的二进制
const EXE_BYTES: &[u8] = include_bytes!("../assets/get_icon_layout.exe");
// 获取最新桌面布局重试次数
const GET_NEW_LAYOUT_MAX_RETRY: u32 = 6;

/// 下策：用python实现获取桌面图标布局信息，打包为exe，输出到stdout
fn run_embedded_exe() -> MyResult<String> {
    // 获取临时目录并创建临时 .exe 文件
    let temp_dir = std::env::temp_dir();
    let exe_path = temp_dir.join(r"fastlink\desktop-setter\get_icon_layout.exe");

    // 将嵌入的 .exe 写入临时文件
    if !exe_path.exists() {
        mk_parents(&exe_path)?;
        File::create(&exe_path)
            .map_err(|e| MyError::new(ErrorCode::IoError, format!("{e}")))?
            .write_all(EXE_BYTES)
            .map_err(|e| MyError::new(ErrorCode::IoError, format!("{e}")))?;
    }
    // 执行临时 .exe 文件
    let output = Command::new(&exe_path)
        .creation_flags(_CREATE_NO_WINDOW)
        .output()
        .map_err(|e| MyError::new(ErrorCode::IoError, format!("{e}")))?;

    // 检查执行结果并处理 GBK 编码的 stdout
    if output.status.success() {
        let (decoded, _, had_errors) = GBK.decode(&output.stdout);
        if !had_errors {
            Ok(decoded.as_ref().to_string())
        } else {
            Err(MyError::new(
                ErrorCode::IoError,
                "解码失败，包含无效字符".to_string(),
            ))
        }
    } else {
        let (decoded, _, _) = GBK.decode(&output.stderr);
        Err(MyError::new(
            ErrorCode::Unknown,
            decoded.as_ref().to_string(),
        ))
    }
}

/// 获取当前桌面布局
pub fn get_cur_layout() -> MyResult<IconLayout> {
    let text = run_embedded_exe()?;
    IconLayout::try_from(text)
}

/// 保存当前桌面的布局到文件
pub fn store_cur_layout_by_deskdir_to_appdata<P: AsRef<Path>>(desk_dir: P) -> MyResult<IconLayout> {
    // 1. 使用python打包二进制，得到stdout: &str
    // 2. 解析dsv格式
    // 3. 保存到文件
    let path = get_layout_data_file_path(desk_dir, None)?;
    store_cur_layout_to_dsv(path)
}

/// 直接调用`get_cur_layout`获取layout数据并保存到path
pub fn store_cur_layout_to_dsv<P: AsRef<Path>>(path: P) -> MyResult<IconLayout> {
    let layout = get_cur_layout()?;
    layout.dump(path)?;
    Ok(layout)
}

/// 抗拒加载新桌面产生的延迟，加载真正新桌面的布局
///
/// 最多`GET_NEW_LAYOUT_MAX_RETRY`次，每次延迟80ms
///
/// 注：使用` std::thread::spawn`多线程可能有未知原因阻塞/阻止桌面刷新
pub fn get_new_layout(last_layout: Option<&IconLayout>) -> MyResult<IconLayout> {
    let mut cnt = 0;
    let null_layout = IconLayout::default();
    let last_layout = last_layout.unwrap_or(&null_layout);

    loop {
        let dur = std::time::Duration::from_millis(80);
        std::thread::sleep(dur);
        cnt += 1;

        let new_layout = get_cur_layout();
        log::debug!("get cur layout to up to date, cnt: {cnt}");

        // 满次数后退出
        if cnt >= GET_NEW_LAYOUT_MAX_RETRY {
            break new_layout;
        }

        if let Err(e) = new_layout {
            e.debug();
            continue;
        }
        let new_layout = new_layout.unwrap();

        // 最大一半次数重试后layout为空则提前退出
        if cnt >= GET_NEW_LAYOUT_MAX_RETRY / 2 && new_layout.entries.is_empty() {
            break Ok(new_layout);
        }

        // 相同
        if new_layout.eq(last_layout) {
            continue;
        // 与上次不同则直接退出
        } else {
            break Ok(new_layout);
        }
    }
}

/// 供desks应用为当前桌面恢复桌面
///
/// - `desk_dir`: 当前桌面路径，自动转为 dsv文件的路径
/// - `layout_cur`: 当前布局数据，若为None则自动使用`get_cur_layout`结果
///
/// # Return
/// - Ok(true) 成功
/// - Ok(false) 布局文件不存在
/// - Err(e) 出错
pub fn restore_desktop_layout_by_deskdir_from_appdata<P: AsRef<Path>>(
    desk_dir: P,
    last_layout: Option<&IconLayout>,
) -> MyResult<bool> {
    // 对应到文件
    let dsv_path = get_layout_data_file_path(desk_dir, None)?;
    // 布局文件存在
    if dsv_path.exists() {
        log::debug!("存在已有的dsv布局文件");
        // 获取新桌面 layout_new 数据
        let layout_new: IconLayout = get_new_layout(last_layout)?;
        // 恢复布局
        restore_desktop_layout_from_dsv(dsv_path, Some(layout_new))
    } else {
        Ok(false)
    }
}

/// 从文件获取布局，并替换当前布局
///
/// - `dsv_path`: 直接指向 dsv文件的路径
/// - `layout_cur`: 当前布局数据，若为None则自动使用`get_cur_layout`结果
///
/// # Return
/// - Ok(true) 成功
/// - Err(e) 错误
pub fn restore_desktop_layout_from_dsv<P: AsRef<Path>>(
    dsv_path: P,
    layout_cur: Option<IconLayout>,
) -> MyResult<bool> {
    // 获取当前布局
    let layout_cur = if let Some(layout_cur) = layout_cur {
        layout_cur
    } else {
        get_cur_layout()?
    };
    // 从dsv读取新layout
    let layout_load = read_layout_from_path(dsv_path)?;
    log::debug!("get layout icon entries: {:?}", layout_load.entries);

    // 过滤加载布局中不存在的icon, 获取name到原始idx的map
    let (icons_ok, name2idx) = layout_load.filter(&layout_cur);
    // 设置layout
    set_icon_layout(&icons_ok, Some(name2idx))?;
    Ok(true)
}

// #[cfg(test)]
// mod tests {

// }
