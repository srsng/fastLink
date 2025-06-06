use crate::types::err::{ErrorCode, MyError};
use std::path::Path;

/// 创建目录
pub fn mkdirs<P: AsRef<Path>>(path: P) -> Result<(), MyError> {
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

/// 创建symlink的前置检查，包含path：
/// 1. 是否存在于文件系统(FileNotExist)
/// 2. 是否是损坏的符号链接(BrokenSymlink)
/// 3. 是否是存在但不是符号链接(TargetExistsAndNotLink)
/// 4. 是否已存在的符号链接(TargetLinkExists)
///
/// 不满足则会返回对应的ErrorCode
/// （没有返回Ok的条件，但在后续处理FileNotExists就是Ok）
pub fn mklink_pre_check(path: &Path) -> Result<(), MyError> {
    // 读取元数据
    match std::fs::symlink_metadata(path) {
        // 路径存在且读取成功
        Ok(metadata) => {
            // 是符号链接
            if metadata.file_type().is_symlink() {
                // 路径元数据有效但 path.exists() == false，符号链接损坏
                if !path.exists() {
                    Err(MyError::new(
                        ErrorCode::BrokenSymlink,
                        format!("{}", path.display()),
                    ))
                } else {
                    Err(MyError::new(
                        ErrorCode::TargetLinkExists,
                        format!("{}", path.display()),
                    ))
                }

            // 路径存在，但不是符号链接
            } else {
                Err(MyError::new(
                    ErrorCode::TargetExistsAndNotLink,
                    format!("{}", path.display()),
                ))
            }
        }
        // 文件不存在
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(MyError::new(
            ErrorCode::FileNotExist,
            format!("{}", path.display()),
        )),
        // 其他错误，如权限问题
        Err(e) => Err(MyError::new(
            ErrorCode::FailToGetFileMetadata,
            format!("无法获取路径元数据({}): {} {}", e.kind(), e, path.display(),),
        )),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_mklink_pre_check() {
//         let broken = Path::new(r"E:\cache\gs_test\test2\123-rename.md");
//         let ok_symlink = Path::new(r"E:\cache\gs_test\test2\1.md");
//         let not_exist_path = Path::new(r"E:\cache\gs_test\test2\not-exist-file.md");
//         let special = Path::new(r"E:\cache\gs_test\test4\1.md");

//         println!("{:?}", mklink_pre_check(special));
//         println!("{:?}", special.exists());
//         println!("{:?}", special.try_exists());
//     }
// }
