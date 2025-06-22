use crate::{ErrorCode, MyError, MyResult};
use fastlink_core::utils::fs::mkdirs;
use std::path::{Path, PathBuf};

const DESKTOP_LAYOUT_FILE_EXTENSION: &str = "dsv";

/// 将路径的根或盘符标识移除
pub fn strip_root<P: AsRef<Path>>(path: P) -> String {
    let path_str = path.as_ref().to_string_lossy().into_owned();

    // 处理Windows扩展路径 \\?\，如\\?\D:\
    if let Some(after_prefix) = path_str.strip_prefix(r#"\\?\"#) {
        // 查找第一个反斜杠后的内容
        if let Some(pos) = after_prefix.find('\\') {
            return after_prefix[pos + 1..].to_string();
        }
        return after_prefix.to_string();
    }

    // 处理Windows驱动器路径，如 D:\
    if path_str.len() >= 2 && path_str.chars().nth(1) == Some(':') {
        let after_drive = &path_str[3..]; // 跳过 D:\
        return after_drive.to_string();
    }

    // 处理Linux根路径 /
    if let Some(path_str) = path_str.strip_prefix('/') {
        return path_str.to_string();
    }

    // 无需处理的情况，直接返回原路径
    path_str
}

/// 根据传入路径获取桌面布局所在目录路径: 在`fastlink\desktop_setter\data\layout\{resolution}\`，并追加传入路径
///
/// ## params
/// - `desk_dir`: 桌面路径
///
/// ## example
/// 1. desk_dir = "C:\System\test" -> fastlink\desktop_setter\data\layout\{resolution}\System\test\
/// 2. desk_dir = "C:\" -> fastlink\desktop_setter\data\layout\{resolution}\
///
pub fn get_layout_data_dir_path<P: AsRef<Path>>(desk_dir: P) -> MyResult<PathBuf> {
    let desk_dir = desk_dir.as_ref();

    // todo 分辨率
    let dir = dirs::config_dir()
        .ok_or_else(|| MyError::new(ErrorCode::Unknown, "fail to load config dir".into()))?;

    let relative_str = strip_root(desk_dir);
    let relative_path = PathBuf::from(relative_str.replace('/', "\\"));
    let p = dir
        .join(r"fastlink\desktop_setter\data\layout\no_resolution")
        .join(relative_path);

    // 创建目录
    mkdirs(&p).map_err(|mut e| {
        e.msg = format!("无法创建布局文件目标目录: {e}");
        e
    })?;

    Ok(p)
}

/// 获取桌面布局文件路径，由桌面布局所在目录路径追加同名文件并设置拓展名`dsv`
///
/// 保证文件所在目录存在（不存在就报错）
pub fn get_layout_data_file_path<P: AsRef<Path>>(
    path: P,
    backup: Option<bool>,
) -> MyResult<PathBuf> {
    let backup = backup.unwrap_or(false);
    let mut path = get_layout_data_dir_path(path)?;

    // 创建
    if !path.exists() {
        mkdirs(&path)?;
    }

    // 获取文件名string
    let mut file_name = path
        .file_name()
        .ok_or_else(|| {
            MyError::new(
                ErrorCode::InvalidInput,
                format!("无法获取路径名称 {}", path.display()),
            )
        })?
        .to_string_lossy()
        .to_string();

    // backup名称
    if backup {
        let timestamp = chrono::Local::now().format("%y-%m-%d-%H-%M-%S");
        file_name = format!("{file_name}-{}", timestamp)
    };

    path = path.join(file_name);
    path.set_extension(DESKTOP_LAYOUT_FILE_EXTENSION);
    log::debug!("desktop layout file: {}", path.display());
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_strip_root() {
        assert_eq!(strip_root("/home/user"), "home/user");
        assert_eq!(strip_root("D:/Program Files"), "Program Files");
        assert_eq!(strip_root(r#"\\?\E:\Data"#), "Data");
        assert_eq!(strip_root("relative/path"), "relative/path");
        assert_eq!(strip_root("/"), "");
        assert_eq!(strip_root("C:/"), "");
        assert_eq!(strip_root(r#"\\?\F:\"#), "");
        assert_eq!(strip_root("\\123/123"), "\\123/123");
    }
}
