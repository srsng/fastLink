use crate::layout::{read_layout_from_path, IconLayout};
use crate::layout2::set_icon_layout;
use crate::utils::get_layout_data_file_path;
use crate::{ErrorCode, MyError, MyResult};
use encoding_rs::GBK;
use fastlink_core::utils::fs::mk_parents;

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
pub fn store_cur_layout_to_path<P: AsRef<Path>>(path: P) -> MyResult<IconLayout> {
    // 1. 使用python打包二进制，得到stdout: &str
    // 2. 解析dsv格式
    // 3. 保存到文件
    log::debug!("获取当前desktop layout中");
    let layout = get_cur_layout()?;
    let path = get_layout_data_file_path(path)?;
    layout.dump(path)?;
    Ok(layout)
}

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

/// 指定路径，加载并应用布局
///
/// - Ok(true) 成功
/// - Ok(false) 布局不存在
/// - Err(e) 出错
pub fn restore_desktop_layout_by_path<P: AsRef<Path>>(
    path: P,
    last_layout: Option<&IconLayout>,
) -> MyResult<bool> {
    // 对应到文件
    let path = get_layout_data_file_path(path)?;
    // 布局文件存在
    if path.exists() {
        log::debug!("存在已有的dsv布局文件");
        // 读取layout数据
        let mut layout_old = read_layout_from_path(path)?;
        log::debug!("get layout icon entries: {:?}", layout_old.entries);
        // 获取新桌面 layout_new 数据
        let layout_new: IconLayout = get_new_layout(last_layout)?;

        // 过滤掉在layout_new中不在layout_old的 icon name, 并记录 idx
        let (icons_ok, name2idx) = layout_old.filter(&layout_new);
        set_icon_layout(&icons_ok, Some(name2idx))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

// #[cfg(test)]
// mod tests {

// }
