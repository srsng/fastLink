use crate::{ErrorCode, MyError, MyResult};
use std::env;
use std::path::Path;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

/// 使用固定逻辑(在路径后加上`_desktop_setter_temp`),
/// 得到输入路径的临时版
pub fn get_dir_temp(path: &Path) -> MyResult<PathBuf> {
    let temp_name = format!(
        "{}{}",
        path.file_name()
            .ok_or_else(|| {
                MyError::new(
                    ErrorCode::IoError,
                    "尝试设定临时路径前，获取路径的名称失败".into(),
                )
            })?
            .display(),
        "_desktop_setter_temp"
    );
    let temp_dir = path
        .parent()
        .ok_or_else(|| {
            MyError::new(
                ErrorCode::IoError,
                "尝试设定临时路径前，获取路径的父目录失败".into(),
            )
        })?
        .join(temp_name);
    Ok(temp_dir)
}

/// 获取存放在注册表中的、经过环境变量解析的Desktop库路径PathBuf
pub fn get_original_desktop_path() -> MyResult<PathBuf> {
    let desktop = get_original_desktop_path_string()?;
    parse_env_vars(desktop)
}

/// 获取存放在注册表中的Desktop库路径String
fn get_original_desktop_path_string() -> MyResult<String> {
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let path = hklm
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders")
        .map_err(|e| MyError::new(ErrorCode::IoError, format!("读取注册表失败: {e}")))?;
    let desktop_path: String = path
        .get_value("Desktop")
        .map_err(|e| MyError::new(ErrorCode::IoError, format!("获取Desktop项失败: {e}")))?;
    Ok(desktop_path)
}

pub fn parse_env_vars(path: String) -> MyResult<PathBuf> {
    let replacements = match_env_placeholders(&path)?;
    let result = apply_replacements(path, replacements);
    Ok(result.into())
}

/// 查找字符串中的环境变量占位符，返回占位符及其值的列表
fn match_env_placeholders(path: &str) -> MyResult<Vec<(String, String)>> {
    let mut replacements = Vec::new();
    let chars: Vec<char> = path.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i + 1 < len && chars[i] == '%' {
            if let Some((placeholder, value, next_i)) = try_parse_windows_var(&chars, path, i)? {
                replacements.push((placeholder, value));
                i = next_i;
                continue;
            }
        } else if chars[i] == '$' {
            if let Some((placeholder, value, next_i)) = try_parse_unix_var(&chars, path, i)? {
                replacements.push((placeholder, value));
                i = next_i;
                continue;
            }
        }
        i += 1;
    }

    Ok(replacements)
}

/// 尝试解析 Windows 风格的 %VAR% 占位符
fn try_parse_windows_var(
    chars: &[char],
    path: &str,
    start: usize,
) -> MyResult<Option<(String, String, usize)>> {
    let len = chars.len();
    let mut i = start + 1; // 跳过起始 %
    let var_start = i;

    while i < len && chars[i] != '%' && (chars[i].is_alphanumeric() || chars[i] == '_') {
        i += 1;
    }

    if i < len && chars[i] == '%' && i > var_start {
        let var_name = &path[var_start..i];
        let var_value = env::var(var_name).map_err(|e| {
            MyError::new(
                ErrorCode::IoError,
                format!("解析Desktop库位置失败，环境变量 {var_name} 不存在: {e}"),
            )
        })?;
        let placeholder = &path[start..=i];
        Ok(Some((placeholder.to_string(), var_value, i + 1)))
    } else {
        Ok(None)
    }
}

/// 尝试解析 Unix 风格的 ${VAR} 或 $VAR 占位符
fn try_parse_unix_var(
    chars: &[char],
    path: &str,
    start: usize,
) -> MyResult<Option<(String, String, usize)>> {
    let len = chars.len();
    let mut i = start + 1; // 跳过 $

    if i < len && chars[i] == '{' {
        i += 1; // 跳过 {
        let var_start = i;

        while i < len && chars[i] != '}' && (chars[i].is_alphanumeric() || chars[i] == '_') {
            i += 1;
        }

        if i < len && chars[i] == '}' && i > var_start {
            let var_name = &path[var_start..i];
            let var_value = env::var(var_name).map_err(|e| {
                MyError::new(
                    ErrorCode::IoError,
                    format!("解析Desktop库位置失败，环境变量 {var_name} 不存在: {e}"),
                )
            })?;
            let placeholder = &path[start..=i];
            return Ok(Some((placeholder.to_string(), var_value, i + 1)));
        }
    } else {
        let var_start = i;

        while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
            i += 1;
        }

        if i > var_start {
            let var_name = &path[var_start..i];
            let var_value = env::var(var_name).map_err(|e| {
                MyError::new(
                    ErrorCode::IoError,
                    format!("解析Desktop库位置失败，环境变量 {var_name} 不存在: {e}"),
                )
            })?;
            let placeholder = &path[start..i];
            return Ok(Some((placeholder.to_string(), var_value, i)));
        }
    }

    Ok(None)
}

/// 将占位符替换为环境变量值
fn apply_replacements(mut path: String, replacements: Vec<(String, String)>) -> String {
    for (placeholder, value) in replacements {
        path = path.replace(&placeholder, &value);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dir_temp() {
        let path = PathBuf::from("C:\\test");
        let dir_temp = get_dir_temp(&path);
        assert!(dir_temp.is_ok());
        assert_eq!(
            PathBuf::from("C:\\test_desktop_setter_temp"),
            dir_temp.unwrap()
        );
    }

    #[test]
    fn test_parse_env_vars() {
        // 设置测试环境变量
        env::set_var("TEST_VAR", "test_value");

        // 测试 Windows 风格
        assert_eq!(
            parse_env_vars("C:\\Users\\%TEST_VAR%\\Desktop".into()).unwrap(),
            PathBuf::from("C:\\Users\\test_value\\Desktop")
        );

        // 测试 Unix 风格
        assert_eq!(
            parse_env_vars("/home/$TEST_VAR/test".into()).unwrap(),
            PathBuf::from("/home/test_value/test")
        );
        assert_eq!(
            parse_env_vars("/home/${TEST_VAR}/test".into()).unwrap(),
            PathBuf::from("/home/test_value/test")
        );

        // 测试不存在的环境变量
        assert!(parse_env_vars("/home/$UNKNOWN_VAR/test".into()).is_err());
    }
}
