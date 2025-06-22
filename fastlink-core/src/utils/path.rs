use crate::types::err::MyError;
use path_clean::PathClean;
use std::path::{Path, PathBuf};

/// 路径规范化
pub fn canonicalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf().clean()
    } else {
        crate::WORK_DIR.join(path.clean())
    }
}

#[inline]
/// 获取路径的一些状态，通过Err(ErrorCode)返回，无Ok返回
///
/// 1. 是否存在于文件系统(FileNotExist)
/// 2. 是否是损坏的符号链接(BrokenSymlink)
/// 3. 是否是存在但不是符号链接(TargetExistsAndNotLink)
/// 4. 是否已存在的符号链接(TargetLinkExists)
pub fn get_path_type<P: AsRef<Path>>(path: P) -> Result<(), MyError> {
    crate::utils::func::mklink_pre_check(path)
}
