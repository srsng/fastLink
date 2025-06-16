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

/// 解析dst, 验证dst路径的父目录是否存在
///
/// 得到的 Ok(PathBuf) 是父目录存在的一个绝对路径，且不包含`..`等
pub fn check_dst(task_args: &LinkTaskArgs) -> MyResult<PathBuf> {
    // 解析dst，相对路径或绝对路径不保证
    let dst_path = parse_args_dst(task_args)?;
    // 转为绝对路径，并尽可能去除`./`等
    let dst_path = crate::utils::path::canonicalize_path(&dst_path);
    // 验证dst路径父目录存在性，并规范化dst，
    // 得到的路径可能以`\\?\`开头以突破最大路径长度限制，但需要注意无`./``..`等
    validate_dst(task_args, &dst_path)
}

/// 根据src参数解析dst参数并转化为路径,
/// 为[DST]自动追加<SRC>名称、拓展名都在这实现
///
/// 标准：
/// 1. dst为None时
///    采用src名称
/// 2. 给出dst，但没有目录倾向（不以`/`或`\`结尾）
///    名称不变
/// 3. 给出dst，且带有目录倾向
///    在后续追加src名称
///
/// 必要时追加拓展名
pub fn parse_args_dst(task_args: &LinkTaskArgs) -> MyResult<PathBuf> {
    let src_path = Path::new(&task_args.src).clean();
    let log = task_args.op_mode == LinkTaskOpMode::Make;

    let mut final_dst = match &task_args.dst {
        None => default_dst_name(&src_path, log),
        Some(d) => {
            let dst_path = Path::new(&d);
            let is_dst_dir_intended = d.ends_with('/') || d.ends_with('\\');

            if is_dst_dir_intended {
                dst_path.join(default_dst_name(&src_path, log))
            } else {
                dst_path.to_path_buf()
            }
        }
    };

    // 只有明确为文件时才处理扩展名
    if src_path.is_file() && task_args.keep_extention {
        final_dst = process_extension(&src_path, final_dst);
    }

    Ok(final_dst)
}

/// 扩展名处理，为dst追加src后缀
fn process_extension(src: &Path, mut dst: PathBuf) -> PathBuf {
    if let Some(src_ext) = src.extension() {
        if !dst.ends_with(src_ext) {
            // 用户已经输入程序相应后缀则跳过
            // 不用set_extension防止src原本带有多个点，或用户输入dst已经携带点
            let new_name = format!("{}.{}", dst.display(), src_ext.display());
            dst.set_file_name(new_name);
            log::info!("get extension `.{}` from <SRC>", src_ext.display());
        }
    }
    dst
}

/// 由src名称生成默认dst名称, 携带后缀
fn default_dst_name(src: &Path, log: bool) -> PathBuf {
    let base_name = 
    // if src.is_file() {
    //     src.file_stem().unwrap_or_else(|| {
    //         log::warn!("无法解析src名称，已设置dst名称为unnamed-fastlink");
    //         OsStr::new("unnamed-fastlink")
    //     })
    // } else
     {
        src.file_name().unwrap_or_else(|| {
            log::warn!("无法解析src名称，已设置dst名称为unnamed-fastlink");
            OsStr::new("unnamed-fastlink")
        })
    };

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

/// 返回规范化后的dst绝对路径
/// 若其父目录不存在且make_dir为false，则将返回Err
pub fn validate_dst(task_args: &LinkTaskArgs, dst: &Path) -> Result<PathBuf, MyError> {
    log::debug!("validate_dst/dst: {}", dst.display());

    let dst_parent_option = dst.parent();    
    // 参数--md不为true时，若dst父目录不存在，或其本身是目录且不存在，则报错返回
    handle_validate_dst_parent_not_exist(task_args.make_dir, dst_parent_option)?;
    // 接下来保证dst_parent存在

    // 规范化dst路径
    canonicalize_dst(dst)
}

/// 规范化dst路径
fn canonicalize_dst(dst: &Path) -> Result<PathBuf, MyError>  {
    let dst_name = dst.file_name().unwrap_or_else(|| {
            log::warn!("无法解析dst名称，已设置dst名称为unnamed-fastlink");
            OsStr::new("unnamed-fastlink")
    });  
    let dst_parent = dst.parent().unwrap();
    // 规范化
    let dst_parent = dst.parent().unwrap().canonicalize().map_err(|e| {
        MyError::new(
            ErrorCode::IoError,
            format!("规范化dst父目录时出错: {} {e}", dst_parent.display())
        )
    })?;
    let dst_path =  dst_parent.join(dst_name);
    Ok(dst_path)
}

/// validate_dst函数辅助函数，为dst创建父目录, 
/// 参数--md不为true时，若dst父目录不存在，则报错
fn handle_validate_dst_parent_not_exist(
    make_dir: bool,
    dst_parent_option: Option<&Path>,
) -> Result<(), MyError> {
    if let Some(parent) = dst_parent_option {
        if !parent.exists() {
            if make_dir {
                // 创建目录并处理错误
                handle_validate_dst_mkdirs(parent)
            } else {
                // 不允许创建目录则直接报错
                Err(MyError::new(
                    ErrorCode::ParentNotExist,
                    format!(
                        "[DST]父目录: {} 不存在，若需自动创建请添加参数--make-dir或--md",
                        parent.display()
                    ),
                ))
            }
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// 为validate_dst函数（handle_validate_dst_parent_not_exist函数）处理创建目录及相关错误
fn handle_validate_dst_mkdirs(dst_parent: &Path) -> Result<(), MyError> {
    match crate::utils::fs::mkdirs(dst_parent) {
        Ok(_) => {
            log::info!("[DST]父目录不存在，已创建: {}", dst_parent.display());
            Ok(())
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
            _ => Err(e),
        }
    } else {
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::types::link_task_args::LinkTaskArgsBuilder;
    use crate::types::link_task_pre::parse_args_dst;
    use std::fs;
    use tempfile::tempdir;

    // 覆盖情况：
    // 1. src为文件，dst为None，结果应为src文件名
    // 2. src为文件，dst无目录倾向，结果应为dst原样
    // 3. src为文件，dst有目录倾向，结果应为dst join src文件名
    // 4. src为目录，dst为None，结果应为src目录名
    // 5. src为目录，dst无目录倾向，结果应为dst原样
    // 6. src为目录，dst有目录倾向，结果应为dst join src目录名

    #[test]
    /// 1. src为文件，dst为None，结果应为src文件名
    fn test_file_dst_none() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("a.txt");
        fs::write(&src_file, b"test").unwrap();
        let args = LinkTaskArgsBuilder::new(src_file.to_str().unwrap().to_string()).build();
        let dst = parse_args_dst(&args).unwrap();
        assert_eq!(dst.file_name().unwrap(), "a.txt");
    }

    #[test]
    /// 2. src为文件，dst无目录倾向，结果应为dst原样
    fn test_file_dst_no_dir_intent() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("a.txt");
        fs::write(&src_file, b"test").unwrap();
        let dst_file = dir.path().join("b");
        let args = LinkTaskArgsBuilder::new(src_file.to_str().unwrap().to_string())
            .dst(dst_file.to_str().unwrap())
            .build();
        let dst = parse_args_dst(&args).unwrap();
        assert_eq!(dst, dst_file);
    }

    #[test]
    /// 3. src为文件，dst有目录倾向，结果应为dst join src文件名
    fn test_file_dst_dir_intent() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("a.txt");
        fs::write(&src_file, b"test").unwrap();
        let dst_dir = dir.path().join("dstdir");
        fs::create_dir(&dst_dir).unwrap();
        let dst_dir_slash = format!("{}{}", dst_dir.to_str().unwrap(), std::path::MAIN_SEPARATOR);
        let args = LinkTaskArgsBuilder::new(src_file.to_str().unwrap().to_string())
            .dst(&dst_dir_slash)
            .build();
        let dst = parse_args_dst(&args).unwrap();
        assert_eq!(dst, dst_dir.join("a.txt"));
    }

    #[test]
    /// 4. src为目录，dst为None，结果应为src目录名
    fn test_dir_dst_none() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("srcdir");
        std::fs::create_dir(&src_dir).unwrap();
        let args = LinkTaskArgsBuilder::new(src_dir.to_str().unwrap().to_string()).build();
        let dst = parse_args_dst(&args).unwrap();
        assert_eq!(dst.file_name().unwrap(), "srcdir");
    }

    #[test]
    /// 5. src为目录，dst无目录倾向，结果应为dst原样
    fn test_dir_dst_no_dir_intent() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("srcdir");
        std::fs::create_dir(&src_dir).unwrap();
        let dst_dir = dir.path().join("dstdir");
        let args = LinkTaskArgsBuilder::new(src_dir.to_str().unwrap().to_string())
            .dst(dst_dir.to_str().unwrap())
            .build();
        let dst = parse_args_dst(&args).unwrap();
        assert_eq!(dst, dst_dir);
    }

    #[test]
    /// 6. src为目录，dst有目录倾向，结果应为dst join src目录名
    fn test_dir_dst_dir_intent() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("srcdir");
        std::fs::create_dir(&src_dir).unwrap();
        let dst_dir = dir.path().join("dstdir");
        fs::create_dir(&dst_dir).unwrap();
        let dst_dir_slash = format!("{}{}", dst_dir.to_str().unwrap(), std::path::MAIN_SEPARATOR);
        let args = LinkTaskArgsBuilder::new(src_dir.to_str().unwrap().to_string())
            .dst(&dst_dir_slash)
            .build();
        let dst = parse_args_dst(&args).unwrap();
        assert_eq!(dst, dst_dir.join("srcdir"));
    }
}
