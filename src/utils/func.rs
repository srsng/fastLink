use crate::types::err::{ErrorCode, MyError};
use crate::utils::logs::{FILE_STYLE, PARENT_STYLE};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

const MAIN_SEPARATOR: char = std::path::MAIN_SEPARATOR;

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

/// 用于Re匹配后、创建连接前的检查：按页展示需要建立符号链接的路径对
pub fn display_paginated_paths(
    paths: &[(PathBuf, PathBuf)],
    page_size: usize,
    re_no_check: bool,
) -> Result<bool, MyError> {
    if paths.is_empty() {
        log::warn!("没有匹配的路径。");
        return Ok(true);
    }

    let total_paths = paths.len();
    let end = page_size.min(total_paths);
    let page_paths = &paths[0..end];
    log::info!(
        "\n创建前检查：匹配的路径 (1 到 {}，共 {} 条):\n{}",
        end,
        total_paths,
        format_matched_paths(page_paths)
    );

    // 开启 re_no_check，仅显示一页后直接返回 true
    if re_no_check {
        return Ok(true);
    }

    let mut start = 0;
    start += page_size;
    loop {
        let end = (start + page_size).min(total_paths);
        if start >= end {
            break;
        }
        let page_paths = &paths[start..end];
        log::info!(
            "\n匹配的路径 ({} 到 {}，共 {} 条):\n{}",
            start + 1,
            end,
            total_paths,
            format_matched_paths(page_paths)
        );

        if end == total_paths {
            println!("\n所有路径已显示。");
            break;
        }

        println!("\n按 Enter 显示下一页，'a' 显示全部，'q' 退出并直接创建链接，Ctrl+C 取消创建:");
        io::stdout()
            .flush()
            .map_err(|e| MyError::new(ErrorCode::IoError, format!("无法刷新输出: {}", e)))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| MyError::new(ErrorCode::IoError, format!("无法读取输入: {}", e)))?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "" => start += page_size, // Enter: 下一页
            "a" => {
                println!("\n所有匹配的路径:\n{}", format_matched_paths(paths));
                break;
            }
            "q" => return Ok(true), // 退出分页并继续
            _ => println!("无效输入，请按 Enter、'a' 或 'q'。"),
        }
    }

    println!("\n按 Enter 确认创建链接，输入 'n' 取消创建:");
    io::stdout()
        .flush()
        .map_err(|e| MyError::new(ErrorCode::IoError, format!("无法刷新输出: {}", e)))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| MyError::new(ErrorCode::IoError, format!("无法读取输入: {}", e)))?;
    let input = input.trim().to_lowercase();

    if input == "n" {
        log::info!("用户取消创建链接。");
        Ok(false)
    } else {
        Ok(true)
    }
}

/// 格式化匹配的路径对
pub fn format_matched_paths(paths: &[(PathBuf, PathBuf)]) -> String {
    paths
        .iter()
        .enumerate()
        .map(|(i, (src, dst))| {
            format!(
                "{:4}. <SRC>{PARENT_STYLE}{}{MAIN_SEPARATOR}{PARENT_STYLE:#}{FILE_STYLE}{:?}{FILE_STYLE:#} -> [DST]{PARENT_STYLE}{}{MAIN_SEPARATOR}{PARENT_STYLE:#}{FILE_STYLE}{:?}{FILE_STYLE:#}",
                i + 1,
                src.parent().unwrap_or_else(|| Path::new("")).display(),
                src.file_name().unwrap_or(std::ffi::OsStr::new("[error: Unknown]")),
                dst.parent().unwrap_or_else(|| Path::new("")).display(),
                dst.file_name().unwrap_or(std::ffi::OsStr::new("[error: Unknown]")),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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
