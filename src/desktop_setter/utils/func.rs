use crate::types::err::{ErrorCode, MyError, MyResult};
use regex::Regex;
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

/// 解析路径中的环境变量占位符（如 %USERPROFILE%, $HOME, ${HOME}）
pub fn parse_env_vars(path: String) -> MyResult<PathBuf> {
    // 匹配 Windows 风格 (%VAR%) 和 Unix 风格 ($VAR 或 ${VAR})
    let re = Regex::new(r"%([\w\d_]+)%|\$\{([\w\d_]+)\}|\$([\w\d_]+)").map_err(|e| {
        MyError::new(
            ErrorCode::IoError,
            format!("解析Desktop库位置失败，无法创建正则表达式: {e}"),
        )
    })?;

    let mut result = path.to_string();
    let mut replacements = Vec::new();

    // 查找所有匹配的环境变量
    for cap in re.captures_iter(path.as_str()) {
        // 提取变量名（Windows: cap[1], Unix: cap[2] 或 cap[3]）
        let var_name = if let Some(name) = cap.get(1).map(|m| m.as_str()) {
            name
        } else if let Some(name) = cap.get(2).map(|m| m.as_str()) {
            name
        } else if let Some(name) = cap.get(3).map(|m| m.as_str()) {
            name
        } else {
            continue;
        };

        // 获取环境变量值
        let var_value = env::var(var_name).map_err(|e| {
            MyError::new(
                ErrorCode::IoError,
                format!("解析Desktop库位置失败，环境变量 {var_name} 不存在: {e}"),
            )
        })?;

        // 记录替换信息
        let match_str = cap.get(0).unwrap().as_str();
        replacements.push((match_str.to_string(), var_value));
    }

    // 执行替换
    for (placeholder, value) in replacements {
        result = result.replace(&placeholder, &value);
    }

    Ok(result.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
    #[test]
    fn test_rename() {
        let path = Path::new(r"C:\Users\srsnn\Desktop_desktop_setter_temp");
        let path_new = Path::new(r"C:\Users\srsnn\Desktop");
        let res = std::fs::rename(path, path_new);
        println!("{:?}", res)
    }
}
