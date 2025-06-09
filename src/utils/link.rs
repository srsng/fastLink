use crate::{
    types::err::{ErrorCode, MyError, MyResult},
    utils::func::mklink_pre_check,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// 创建符号链接并处理错误
/// 在dst创建，指向src
/// Ok(false)表示跳过创建
pub fn mklink(
    src: &PathBuf,
    dst: &PathBuf,
    overwrite_links: Option<bool>,
    overwrite_broken_links: Option<bool>,
    skip_exist_links: Option<bool>,
    skip_broken_src_links: Option<bool>,
    allow_broken_src: Option<bool>,
) -> Result<bool, MyError> {
    let overwrite_links = overwrite_links.unwrap_or(false);
    let overwrite_broken_links = overwrite_broken_links.unwrap_or(true);
    let skip_exist_links = skip_exist_links.unwrap_or(false);
    let skip_broken_src_links = skip_broken_src_links.unwrap_or(true);
    let allow_broken_src = allow_broken_src.unwrap_or(false);

    // 检查src
    let res = mklink_pre_check(src);
    match crate::types::link_task_pre::handle_mklink_pre_check_error_for_src(res) {
        Ok(_) => Ok(()),
        Err(e) if e.code == ErrorCode::BrokenSymlink => {
            if allow_broken_src {
                log::warn!("将使用损坏的符号链接作为src: {}", src.display());
                Ok(())
            } else if skip_broken_src_links {
                log::warn!("src为损坏的符号链接，已跳过: {}", src.display());
                return Ok(false);
            } else {
                Err(e)
            }
        }
        e => e,
    }?;

    // 检查dst
    let res = mklink_pre_check(dst);
    match handle_mklink_pre_check_error_for_dst(
        res,
        dst,
        overwrite_links,
        overwrite_broken_links,
        skip_exist_links,
        allow_broken_src,
    ) {
        Ok(_) => Ok(()),
        Err(e) if e.code == ErrorCode::SkipExistingLink => return Ok(false),
        e => e,
    }?;
    // 接下来能够保证dst不存在（且不是已有的其他文件、不是损坏的符号链接），
    // src是有效路径(且不是损坏符号链接)

    let mklink_res = create_symlink(src, dst);
    match mklink_res {
        Ok(_) => {
            log::info!(
                "创建符号链接: 在 '{}', 指向 '{}'",
                dst.display(),
                src.display()
            );
            Ok(true)
        }
        res => handle_create_symlink_error(res, src, dst).map(|_| true),
    }
}

/// 检查 overwrite 和 skip_exist_links 参数选项
/// 覆写前删除符号链接也在此处完成
///
/// 调用需要保证dst是一个已存在的符号链接（损坏与否不重要）
fn handle_exists_link(
    dst: &Path,
    overwrite_links: bool,
    skip_exist_links: bool,
    log: bool,
) -> Result<(), MyError> {
    // let metadata = fs::symlink_metadata(dst).map_err(|e| {
    //     MyError::new(
    //         ErrorCode::FailToGetFileMetadata,
    //         format!("无法获取路径元数据 {}: {}", dst.display(), e),
    //     )
    // })?;

    // 跳过已存在链接
    if skip_exist_links {
        log::info!("已跳过目标链接 {}", dst.display());
        Err(MyError::new(
            ErrorCode::SkipExistingLink,
            format!("目标路径符号链接 {} 已存在，跳过创建", dst.display()),
        ))
    // 删除已存在链接
    } else if overwrite_links {
        del_exists_link(dst, overwrite_links, None).map(|b| {
            if log && b {
                log::info!("删除符号链接成功: {}", dst.display());
            } else if b {
                log::debug!("删除符号链接成功: {}", dst.display());
            }
        })

    // 不处理，由调用函数错误处理时终止程序
    } else {
        Err(MyError::new(
            ErrorCode::TargetLinkExists,
            format!("{}", dst.display()),
        ))
    }
}

fn handle_mklink_pre_check_error_for_dst(
    res: Result<(), MyError>,
    dst: &Path,
    overwrite_links: bool,
    overwrite_broken_links: bool,
    skip_exist_links: bool,
    allow_broken_src: bool,
) -> Result<(), MyError> {
    if let Some(mut e) = res.err() {
        match e.code {
            ErrorCode::FileNotExist => Ok(()),
            ErrorCode::TargetExistsAndNotLink => {
                // e.log();
                if allow_broken_src {
                    Ok(())
                } else {
                    e.msg = format!("无法创建链接：src部分为损坏的符号链接 {}", e.msg);
                    Err(e)
                }
            }
            // 确定目标路径已存在符号链接，需要考虑覆写/跳过
            ErrorCode::TargetLinkExists => {
                e.warn();
                handle_exists_link(dst, overwrite_links, skip_exist_links, true)
            }
            // 确定目标路径已存在且损坏的符号链接，需要考虑覆写/跳过
            ErrorCode::BrokenSymlink => {
                e.warn();
                // overwrite参数满足其一即可
                let cond = overwrite_links || overwrite_broken_links;
                handle_exists_link(dst, cond, skip_exist_links, true)
            }
            _ => Err(e),
        }
    } else {
        Ok(())
    }
}

/// 删除符号链接，需要传入overwrite_links参数，避免误用
pub fn del_exists_link(
    dst: &Path,
    overwrite_links: bool,
    not_exist_ok: Option<bool>,
) -> Result<bool, MyError> {
    if overwrite_links {
        // 检查是否是symlink
        match mklink_pre_check(dst) {
            Ok(_) => Ok(()),
            Err(e) if e.code == ErrorCode::TargetLinkExists => Ok(()),
            Err(e) if e.code == ErrorCode::BrokenSymlink => Ok(()),
            Err(mut e) if e.code == ErrorCode::FileNotExist => {
                if not_exist_ok.unwrap_or(true) {
                    return Ok(false);
                } else {
                    e.msg = format!("目标不存在 {}", e.msg);
                    Err(e)
                }
            }
            Err(mut e) if e.code == ErrorCode::TargetNotALink => {
                e.msg = format!("尝试删除非符号链接路径 {}", e.msg);
                Err(e)
            }
            e => e,
        }?;
        // 区分符号链接类型
        if dst.is_dir() {
            fs::remove_dir(dst).map_err(|e| {
                MyError::new(
                    ErrorCode::FailToDelLink,
                    format!("(DIR) {}: {}", dst.display(), e),
                )
            })?;
            Ok(true)
        } else if dst.is_file() {
            fs::remove_file(dst).map_err(|e| {
                MyError::new(
                    ErrorCode::FailToDelLink,
                    format!("(FILE){}: {}", dst.display(), e),
                )
            })?;
            Ok(true)
        } else {
            log::debug!("损坏的符号链接 {}, 尝试作为文件删除", dst.display());
            let res_file = fs::remove_file(dst);
            if res_file.is_err() {
                log::debug!("删除失败: {}，尝试作为目录删除", dst.display());
                let res_dir = fs::remove_dir(dst).map_err(|_| {
                    MyError::new(
                        ErrorCode::FailToDelLink,
                        format!(
                            "(Unkown){}: 未知类型的符号链接，无法删除，请尝试手动删除",
                            dst.display()
                        ),
                    )
                });
                if let Err(e) = res_dir {
                    log::debug!("作为目录删除失败: {}", dst.display());
                    Err(e)
                } else {
                    Ok(false)
                }
            } else {
                Ok(true)
            }
        }
    } else {
        Ok(false)
    }
}

/// 处理/转换create_symlink返回的错误
fn handle_create_symlink_error(
    res: Result<(), MyError>,
    _src: &Path,
    dst: &Path,
) -> Result<(), MyError> {
    match res {
        // 成功创建
        Ok(_) => Ok(()),
        // 目标链接已存在
        Err(mut e) if e.code == ErrorCode::TargetLinkExists => {
            e.msg = format!(
                "目标链接 {} 已存在，若需覆盖请添加参数--overwrite，需要跳过请添加参数--skip-exist",
                dst.display()
            );
            Err(e)
        }
        // 目标路径存在，但不是符号链接
        Err(e) if e.code == ErrorCode::TargetExistsAndNotLink => {
            log::error!("目标路径存在，但不是符号链接，无法处理 {}", e.msg);
            Ok(())
        }
        // 跳过已存在的符号链接
        Err(e) if e.code == ErrorCode::SkipExistingLink => {
            log::info!("已跳过目标链接{}", dst.display());
            Ok(())
        }
        // 无法删除已存在的符号链接
        Err(e) if e.code == ErrorCode::FailToDelLink => {
            log::error!("删除链接失败：{}", dst.display());
            Ok(())
        }
        // 其他错误, 直接返回由main输出: PermissionDenied, FailToGetPathParent, FailToGetFileMetadata, FailAtMakeLink, Unknown
        Err(e) => Err(e),
    }
}

/// 智能创建符号链接（自动判断文件/目录）
#[cfg(windows)]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), MyError> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    // check_overwrite(dst, overwrite_links, skip_exist_links)?;

    // 获取源文件元数据
    let metadata = fs::metadata(src).map_err(|e| {
        MyError::new(
            ErrorCode::FailToGetFileMetadata,
            format!("无法获取源文件元数据 {}: {}", src.display(), e),
        )
    })?;

    // 根据类型选择创建方式
    if metadata.is_file() {
        let res = std::os::windows::fs::symlink_file(src, dst);
        convert_create_symlink_res(res, src, dst)
    } else if metadata.is_dir() {
        let res = std::os::windows::fs::symlink_dir(src, dst);
        convert_create_symlink_res(res, src, dst)
    } else {
        Err(MyError::new(
            ErrorCode::Unknown,
            "奇怪的错误: <SRC>既不是文件也不是目录，可能是损坏的符号链接或别的什么".into(),
        ))
    }
}

/// 转换create_symlink中创建符号链接的res
fn convert_create_symlink_res(
    res: Result<(), std::io::Error>,
    src: &Path,
    dst: &Path,
) -> MyResult<()> {
    if let Err(e) = res {
        match e.kind() {
            #[cfg(windows)]
            std::io::ErrorKind::PermissionDenied => Err(MyError::new(
                ErrorCode::PermissionDenied,
                "权限不足，请尝试使用管理员权限，或开启开发者模式".into(),
            )),
            #[cfg(not(windows))]
            std::io::ErrorKind::PermissionDenied => Err(MyError::new(
                ErrorCode::PermissionDenied,
                "权限不足，请尝试sudo".into(),
            )),
            _ => Err(MyError::new(
                ErrorCode::FailAtMakeLink,
                format!(
                    "无法创建目录符号链接 '{}' -> '{}': {}",
                    dst.display(),
                    src.display(),
                    e
                ),
            )),
        }
    } else {
        Ok(())
    }
}

#[cfg(unix)]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), MyError> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    // check_overwrite(dst, overwrite_links, skip_exist_links)?;
    let res = std::os::unix::fs::symlink(src, dst);
    convert_create_symlink_res(res, src, dst)
}
