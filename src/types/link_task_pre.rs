use crate::types::args::Args;
use crate::types::args::DEFAULT_RE_MAX_DEPTH;
use crate::types::err::{ErrorCode, MyError};
use crate::utils::func::{mkdirs, mklink_pre_check};
use path_clean::PathClean;
use regex::Regex;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct LinkTaskPre {
    pub src_ori: String,             // 原始源路径
    pub dst_ori: Option<String>,     // 原始目标路径
    pub re_pattern: Option<Regex>,   // 正则表达式模式
    pub re_max_depth: usize,         // 正则表达式模式最大深度
    pub re_follow_links: bool,       // re匹配过程中深入读取符号链接进行匹配
    pub keep_extention: bool,        // 是否自动保留<SRC>的文件拓展名到[DST]
    pub make_dir: bool,              // 是否自动创建不存在的目录
    pub only_file: bool,             //只处理文件
    pub only_dir: bool,              //只处理目录
    pub overwrite_links: bool,       // 覆盖已存在的符号链接
    pub overwrite_broken_link: bool, // 覆盖同名已存在的损坏的符号链接
    pub skip_exist_links: bool,
    pub skip_broken_src_links: bool,
    pub re_no_check: bool,
    pub re_output_flatten: bool,

    pub src_path: Option<PathBuf>, // 规范化后的源路径
    pub dst_path: Option<PathBuf>, // 规范化后的目标目录路径
}

impl From<&Args> for LinkTaskPre {
    fn from(args: &Args) -> Self {
        LinkTaskPre {
            src_ori: args.src.clone(),
            dst_ori: args.dst.clone(),
            re_pattern: args.regex.clone(),
            re_max_depth: args.re_max_depth.unwrap_or(DEFAULT_RE_MAX_DEPTH),
            re_follow_links: args.re_follow_links,
            keep_extention: args.keep_extention,
            make_dir: args.make_dir,
            only_file: args.only_file,
            only_dir: args.only_dir,
            overwrite_links: args.overwrite_links,
            overwrite_broken_link: args.overwrite_broken_link,
            skip_exist_links: args.skip_exist_links,
            skip_broken_src_links: args.skip_broken_src_links,
            re_no_check: args.re_no_check,
            re_output_flatten: args.re_output_flatten,
            ..Default::default()
        }
    }
}

impl LinkTaskPre {
    pub fn parse(&mut self) -> Result<(), MyError> {
        // 获取src_path
        self.check_src()?;
        // 获取dst_path
        self.check_dst()?;
        Ok(())
    }

    /// 解析、规范、验证src
    /// 并将结果存于src_path
    pub fn check_src(&mut self) -> Result<(), MyError> {
        let src_abs_res = dunce::canonicalize(&self.src_ori);
        if let Err(e) = src_abs_res {
            Err(MyError::new(
                ErrorCode::InvalidInput,
                format!(
                    "请检查<SRC>'{}'是否存在 (或是否为损坏的符号链接). Fail to canonicalize <SRC>: {}",
                    &self.src_ori, e
                ),
            ))
        } else {
            let src_path = src_abs_res.unwrap();
            let res = mklink_pre_check(&src_path);
            handle_mklink_pre_check_error_for_src(res)?;
            self.src_path = Some(src_path);
            Ok(())
        }
    }

    /// 解析dst, 验证dst路径的父目录是否存在,
    /// 并将结果存放在dst_path
    pub fn check_dst(&mut self) -> Result<(), MyError> {
        // 解析dst
        let dst_path = self.parse_args_dst()?;
        // 验证dst路径父目录
        self.dst_path = Some(self.validate_dst(&dst_path)?);
        Ok(())
    }

    /// 解析dst参数并转化为路径
    /// 为[DST]自动追加<SRC>名称、拓展名都在这实现
    pub fn parse_args_dst(&mut self) -> Result<PathBuf, MyError> {
        let src_path = Path::new(&self.src_ori);
        let mut final_dst = match &self.dst_ori {
            Some(d) => {
                // SRC是文件而DST是目录的情况: 为DST追加SRC文件名
                let dst_path = Path::new(&d);
                if src_path.is_file() && dst_path.is_dir() {
                    canonicalize_path(&dst_path.join(default_dst_path(src_path)))
                } else {
                    canonicalize_path(&PathBuf::from(d))
                }
            }
            // 没有传入DST: 使用SRC文件名
            None => default_dst_path(src_path),
        };

        // 处理keep_extension: 是否保留拓展名
        process_extension(src_path, &mut final_dst, self.keep_extention);

        Ok(final_dst.clean())
    }

    /// 返回规范化后的dst绝对路径
    /// 若其父目录不存在且make_dir为false，则将返回Err
    pub fn validate_dst(&mut self, dst: &Path) -> Result<PathBuf, MyError> {
        // dst: &Path, make_dir: bool
        log::debug!("validate_dst/dst: {}", dst.display());

        let dst_parent_option = dst.parent();
        // dst父目录不存在
        handle_validate_dst_parent_not_exist(dst, self.make_dir, dst_parent_option)
    }
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
fn process_extension(src: &Path, dst: &mut PathBuf, keep_extention: bool) {
    if keep_extention {
        if let Some(src_ext) = src.extension() {
            // 仅处理文件路径（通过文件名存在判断）
            if let Some(file_name) = dst.file_name() {
                let dst_path = Path::new(file_name);

                let dst_str = dst.to_str().unwrap_or_default();

                // 不用std::path::MAIN_SEPARATOR判断是因为用户经常混用`\`与`/`
                #[cfg(windows)]
                let is_dir = dst.is_dir()
                    || (!dst.to_str().unwrap_or_default().is_empty()
                        && (dst_str.ends_with('/') || dst_str.ends_with('\\')));
                #[cfg(not(windows))]
                let is_dir = dst.is_dir()
                    || (!dst.to_str().unwrap_or_default().is_empty()
                        && dst_str.ends_with(std::path::MAIN_SEPARATOR));

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
}

/// 路径规范化
fn canonicalize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf().clean()
    } else {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join(path.clean())
    }
}

/// 为validate_dst函数处理dst父目录不存在的情况
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
