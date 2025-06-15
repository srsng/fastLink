use crate::types::args::Args;
use crate::types::err::{ErrorCode, MyError, MyResult};
use crate::types::link_task_args::{LinkTaskArgs, LinkTaskOpMode};
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
        log::debug!("已从Args构建LinkTaskPre");
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
/// 不存在，或是损坏的符号链接都将返回Err
pub fn check_src(task_args: &LinkTaskArgs) -> MyResult<PathBuf> {
    let s = &task_args.src;
    // 验证存在、或是有效的符号链接
    let src = Path::new(s);
    let res = crate::utils::func::mklink_pre_check(src);
    match res {
        Ok(_) => Ok(()),
        Err(e) if e.code == ErrorCode::TargetLinkExists => Ok(()),
        Err(e) if e.code == ErrorCode::TargetExistsAndNotLink => Ok(()),
        Err(e) if e.code == ErrorCode::FileNotExist => Err(e),
        Err(mut e) if e.code == ErrorCode::BrokenSymlink => {
            e.msg = format!("\n损坏的符号链接不可以作为src: {}", e.msg);
            Err(e)
        }
        Err(e) => Err(e),
    }?;
    let src_abs_res = dunce::canonicalize(s);
    if let Err(e) = src_abs_res {
        Err(MyError::new(
            ErrorCode::InvalidInput,
            format!(
                "请检查<SRC>'{}'是否存在 (或是否为损坏的符号链接). Fail to canonicalize <SRC>: {}",
                s, e
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
    let log = task_args.op_mode == LinkTaskOpMode::Make;

    let mut final_dst = match &task_args.dst {
        Some(d) => {
            // SRC是文件而DST是目录的情况: 为DST追加SRC文件名
            let dst_path = Path::new(&d);
            let is_dst_dir_intended = d.ends_with('/') || d.ends_with('\\');

            // bug todo: 目录/文件判定有问题
            // 如` fastlink "...AppData\Roaming\com.resources-manager.app\" ./123/  -k --debug`
            // (src为目录)
            // dst被修改为123.app

            // 规范化
            if src_path.is_file() && (dst_path.is_dir() || is_dst_dir_intended) {
                crate::utils::path::canonicalize_path(
                    &dst_path.join(default_dst_path(src_path, log)),
                )?
            } else {
                crate::utils::path::canonicalize_path(&PathBuf::from(d))?
            }
        }
        // 没有传入DST: 使用SRC文件名
        None => default_dst_path(src_path, log),
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

    if task_args.op_mode != LinkTaskOpMode::Make {
        Ok(dst.to_path_buf())
    } else {
        // todo: dst为空时传入自动从src获取名称的dst的parent为Some("")
        let dst_parent_option = dst.parent();
        log::debug!("{:?}", dst_parent_option);
        // 参数--md不为true时，若dst父目录不存在，或其本身是目录且不存在，则报错
        let dst = handle_validate_dst_parent_not_exist(dst, task_args.make_dir, dst_parent_option)?;
        handle_validate_dst_dir_not_exists(dst, task_args.make_dir)
    }
}

/// 生成默认目标路径
fn default_dst_path(src: &Path, log: bool) -> PathBuf {
    let base_name = src.file_stem().unwrap_or_else(|| {
        src.file_name().unwrap_or_else(|| {
            log::warn!("无法解析src名称，已设置dst名称为unnamed-fastlink");
            OsStr::new("unnamed-fastlink")
        })
    });

    if log {
        // 输出日志信息
        log::info!(
            "已由<SRC>确定目标名 {} → {}",
            src.display(),
            base_name.to_string_lossy()
        );
    } else {
        log::debug!(
            "已由<SRC>确定目标名 {} → {}",
            src.display(),
            base_name.to_string_lossy()
        );
    }

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

/// validate_dst函数辅助函数
/// 参数--md不为true时，若其本身是目录且不存在，则报错
fn handle_validate_dst_dir_not_exists(dst: PathBuf, make_dir: bool) -> Result<PathBuf, MyError> {
    if dst.is_dir() && !dst.exists() {
        if make_dir {
            // 创建目录
            crate::utils::fs::mkdirs(&dst)?;
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
    if let Some(parent) = dst_parent_option {
        // log::debug!("validate_dst/parent.exists(): {}", parent.exists());

        // dst未传入的情况
        if parent == Path::new("") {
            Ok(crate::WORK_DIR.to_path_buf().join(dst))
        } else if !parent.exists() {
            if make_dir {
                // 创建目录并处理错误
                Ok(handle_validate_dst_mkdirs(dst, parent)?)
            } else {
                Err(MyError::new(
                    ErrorCode::ParentNotExist,
                    format!(
                        "[DST]父目录: {} 不存在，若需自动创建请添加参数--make-dir或--md",
                        parent.display()
                    ),
                ))
            }
        } else {
            Ok(dst.to_path_buf())
        }
    } else {
        Ok(dst.to_path_buf())
    }
}

/// 为validate_dst函数（handle_validate_dst_parent_not_exist函数）处理创建目录及相关错误
fn handle_validate_dst_mkdirs(dst: &Path, dst_parent: &Path) -> Result<PathBuf, MyError> {
    match crate::utils::fs::mkdirs(dst_parent) {
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
