use crate::types::err::{ErrorCode, MyError};
use std::path::Path;

/// 创建目录
pub fn mkdirs(path: &Path) -> Result<(), MyError> {
    let res = std::fs::create_dir_all(path);
    match res {
        Err(e) => Err(MyError::new(ErrorCode::FailToMakeDir, format!("{}", e))),
        _ => Ok(()),
    }
}
/// 创建传入路径的父目录
pub fn mk_parents(path: &Path) -> Result<(), MyError> {
    let parent = path.parent();
    if parent.is_none() {
        return Err(MyError::new(
            ErrorCode::FailToGetPathParent,
            format!("{}", &path.display()),
        ));
    }
    let parent = parent.unwrap();
    if !parent.exists() {
        let res = std::fs::create_dir_all(parent);
        match res {
            Err(e) => Err(MyError::new(ErrorCode::FailToMakeDir, format!("{}", e))),
            _ => Ok(()),
        }
    } else {
        Ok(())
    }
}
