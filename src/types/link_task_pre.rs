use crate::types::args::Args;
use crate::types::err::{ErrorCode, MyError, MyResult};
use crate::types::link_task_args::LinkTaskArgs;
use crate::utils::func::mkdirs;
use path_clean::PathClean;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// 创建链接任务的准备
/// 负责解析、规范、验证路径src、dst
/// 可通过from或try_new构建
#[derive(Debug, Default)]
pub struct LinkTaskPre {
    pub args: LinkTaskArgs,        // 创建链接需要的一些参数
    pub src_path: Option<PathBuf>, // 规范化后的源路径
    pub dst_path: Option<PathBuf>, // 规范化后的目标目录路径
}

impl From<&Args> for LinkTaskPre {
    fn from(args: &Args) -> Self {
        LinkTaskPre {
            args: LinkTaskArgs::from(args),
            src_path: None,
            dst_path: None,
        }
    }
}

impl LinkTaskPre {
    /// 创建一个已经完成解析src、dst的 LinkTaskPre
    pub fn try_new(args: LinkTaskArgs) -> MyResult<Self> {
        let mut task_pre = LinkTaskPre {
            args,
            src_path: None,
            dst_path: None,
        };
        task_pre.parse()?;
        Ok(task_pre)
    }
}

impl LinkTaskPre {
    pub fn parse(&mut self) -> MyResult<()> {
        // 获取src_path
        self.src_path = Some(check_src(&self.args)?);
        // 获取dst_path
        self.dst_path = Some(check_dst(&self.args)?);
        Ok(())
    }
}

/// 解析、规范、验证src,
/// 并将结果存于src_path
pub fn check_src(task_args: &LinkTaskArgs) -> MyResult<PathBuf> {
    let src_abs_res = dunce::canonicalize(&task_args.src);
    if let Err(e) = src_abs_res {
        Err(MyError::new(
            ErrorCode::InvalidInput,
            format!(
                "请检查<SRC>'{}'是否存在 (或是否为损坏的符号链接). Fail to canonicalize <SRC>: {}",
                &task_args.src, e
            ),
        ))
    } else {
        Ok(src_abs_res.unwrap())
    }
}

/// 解析dst, 验证dst路径的父目录是否存在,
/// 并将结果存放在dst_path
pub fn check_dst(task_args: &LinkTaskArgs) -> MyResult<PathBuf> {
    // 解析dst
    let dst_path = parse_args_dst(task_args)?;
    // 验证dst路径父目录
    validate_dst(task_args, &dst_path)
}

/// 解析dst参数并转化为路径
/// 为[DST]自动追加<SRC>名称、拓展名都在这实现
pub fn parse_args_dst(task_args: &LinkTaskArgs) -> MyResult<PathBuf> {
    let src_path = Path::new(&task_args.src);
    let mut final_dst = match &task_args.dst {
        Some(d) => {
            // SRC是文件而DST是目录的情况: 为DST追加SRC文件名
            let dst_path = Path::new(&d);
            let is_dst_dir_intended = d.ends_with('/') || d.ends_with('\\');
            // 规范化
            if src_path.is_file() && (dst_path.is_dir() || is_dst_dir_intended) {
                canonicalize_path(&dst_path.join(default_dst_path(src_path)))?
            } else {
                canonicalize_path(&PathBuf::from(d))?
            }
        }
        // 没有传入DST: 使用SRC文件名
        None => default_dst_path(src_path),
    };

    // 处理keep_extension: 是否保留拓展名
    final_dst = process_extension(src_path, final_dst, task_args.keep_extention);

    Ok(final_dst.clean())
}

/// 返回规范化后的dst绝对路径
/// 若其父目录不存在且make_dir为false，则将返回Err
pub fn validate_dst(task_args: &LinkTaskArgs, dst: &Path) -> Result<PathBuf, MyError> {
    // dst: &Path, make_dir: bool
    log::debug!("validate_dst/dst: {}", dst.display());

    let dst_parent_option = dst.parent();
    // 参数--md不为true时，若dst父目录不存在，或其本身是目录且不存在，则报错
    let dst = handle_validate_dst_parent_not_exist(dst, task_args.make_dir, dst_parent_option)?;
    handle_validate_dst_dir_not_exists(dst, task_args.make_dir)
}

/// 生成默认目标路径
fn default_dst_path(src: &Path) -> PathBuf {
    let base_name = src.file_stem().unwrap_or_else(|| {
        src.file_name().unwrap_or_else(|| {
            log::warn!("无法解析src名称，已设置dst名称为unnamed-fastlink");
            OsStr::new("unnamed-fastlink")
        })
    });

    // 输出日志信息
    log::info!(
        "已由<SRC>确定目标名 {} → {}",
        src.display(),
        base_name.to_string_lossy()
    );

    PathBuf::from(base_name)
}

/// 扩展名处理逻辑（统一处理相对/绝对路径）, 调用后直接修改传入的dst
fn process_extension(src: &Path, mut dst: PathBuf, keep_extention: bool) -> PathBuf {
    if keep_extention {
        if let Some(src_ext) = src.extension() {
            // 仅处理文件路径（通过文件名存在判断）
            if let Some(file_name) = dst.file_name() {
                let dst_path = Path::new(file_name);

                let dst_str = dst.to_str().unwrap_or_default();

                // 不用std::path::MAIN_SEPARATOR判断是因为用户经常混用`\`与`/`
                #[cfg(windows)]
                let is_dir = dst.is_dir()
                    || (!dst_str.is_empty() && (dst_str.ends_with('/') || dst_str.ends_with('\\')));
                #[cfg(not(windows))]
                let is_dir = dst.is_dir()
                    || (!dst_str.is_empty() && dst_str.ends_with(std::path::MAIN_SEPARATOR));

                if !is_dir && dst_path.extension().is_none() && !src_ext.is_empty() {
                    let new_name = format!(
                        "{}.{}",
                        dst_path.to_string_lossy(),
                        src_ext.to_string_lossy()
                    );
                    dst.set_file_name(new_name);
                    log::info!("get extension `.{}` from <SRC>", src_ext.to_string_lossy());
                }
            }
        }
    }
    dst
}

/// 路径规范化
fn canonicalize_path(path: &Path) -> Result<PathBuf, MyError> {
    if path.is_absolute() {
        Ok(path.to_path_buf().clean())
    } else {
        // let curdir = std::env::current_dir();
        // match curdir {
        //     Ok(curdir) => Ok(curdir.join(path.clean())),
        //     Err(e) => Err(MyError::new(
        //         ErrorCode::Unknown,
        //         format!("Failed to get current directory: {}", e),
        //     )),
        // }
        Ok(crate::WORK_DIR.join(path.clean()))
    }
}

/// validate_dst函数辅助函数
/// 参数--md不为true时，若其本身是目录且不存在，则报错
fn handle_validate_dst_dir_not_exists(dst: PathBuf, make_dir: bool) -> Result<PathBuf, MyError> {
    if dst.is_dir() && !dst.exists() {
        if make_dir {
            // 创建目录
            mkdirs(&dst)?;
            Ok(dst)
        } else {
            Err(MyError::new(
                ErrorCode::InvalidInput,
                format!("目录 {} 不存在，若需自动创建请添加--md参数", dst.display()),
            ))
        }
    } else {
        Ok(dst)
    }
}

/// validate_dst函数辅助函数
/// 参数--md不为true时，若dst父目录不存在，则报错
fn handle_validate_dst_parent_not_exist(
    dst: &Path,
    make_dir: bool,
    dst_parent_option: Option<&Path>,
) -> Result<PathBuf, MyError> {
    if dst_parent_option.is_some() && !dst_parent_option.unwrap().exists() {
        let dst_parent = dst_parent_option.unwrap().clean();
        if make_dir {
            // 创建目录并处理错误
            Ok(handle_validate_dst_mkdirs(dst, dst_parent)?)
        } else {
            Err(MyError::new(
                ErrorCode::ParentNotExist,
                format!(
                    "[DST]父目录: {} 不存在，若需自动创建请添加参数--make-dir或--md",
                    dst_parent.display()
                ),
            ))
        }
    } else {
        Ok(dst.to_path_buf())
    }
}

/// 为validate_dst函数（handle_validate_dst_parent_not_exist函数）处理创建目录及相关错误
fn handle_validate_dst_mkdirs(dst: &Path, dst_parent: PathBuf) -> Result<PathBuf, MyError> {
    match mkdirs(&dst_parent) {
        Ok(_) => {
            log::warn!("[DST]父目录不存在，已创建: {}", dst_parent.display());
            // 重新组合dst路径
            let dst_path = if let Some(dst_filename) = dst.file_name() {
                dst_parent.join(dst_filename)
            } else {
                dst.to_path_buf()
            };
            log::debug!("validate_dst/dst return: {}", dst_path.display());
            Ok(dst_path)
        }
        Err(e) => Err(MyError::new(
            ErrorCode::Unknown,
            format!(
                "[DST]父目录: {} 创建失败\n\tErrorMsg: {}",
                dst_parent.display(),
                e
            ),
        )),
    }
}

/// 为检查src是否是损坏符号链接进行错误处理
pub fn handle_mklink_pre_check_error_for_src(res: Result<(), MyError>) -> Result<(), MyError> {
    if let Some(e) = res.err() {
        match e.code {
            ErrorCode::FileNotExist => Ok(()),
            ErrorCode::TargetExistsAndNotLink => Ok(()),
            ErrorCode::TargetLinkExists => Ok(()),
            // // 目标路径已存在且损坏的符号链接
            // ErrorCode::BrokenSymlink => {
            //     e.log();
            //     Err(e)
            // }
            _ => Err(e),
        }
    } else {
        Ok(())
    }
}
