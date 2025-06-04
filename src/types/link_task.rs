use crate::types::args::{get_re_max_depth, Args};
use crate::types::err::{ErrorCode, MyError};
use crate::types::link_task_pre::LinkTaskPre;
use crate::utils::func::{mk_parents, mkdirs};
use path_clean::PathClean;
use regex::Regex;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::os::windows::fs::{symlink_dir, symlink_file};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct LinkTask {
    pub src_ori: String,           // 原始源路径
    pub dst_ori: Option<String>,   // 原始目标路径
    pub re_pattern: Option<Regex>, // 正则表达式模式
    pub re_max_depth: usize,       // 正则表达式模式最大深度
    pub re_follow_links: bool,     // re匹配过程中深入读取符号链接进行匹配
    pub keep_extention: bool,      // 是否自动保留<SRC>的文件拓展名到[DST]
    pub make_dir: bool,            // 是否自动创建不存在的目录
    pub only_file: bool,           //只处理文件
    pub only_dir: bool,            //只处理目录
    pub overwrite_link: bool,      // 覆盖已存在的符号链接

    pub src_path: PathBuf,                              // 规范化后的源路径
    pub dst_path: PathBuf,                              // 规范化后的目标目录路径
    pub matched_paths: Option<Vec<(PathBuf, PathBuf)>>, // 匹配的源路径和目标路径对
}

/// Formats a slice of path pairs into a readable string.
fn format_matched_paths(paths: &[(PathBuf, PathBuf)]) -> String {
    paths
        .iter()
        .enumerate()
        .map(|(i, (src, dst))| format!("{:4}. {} -> {}", i + 1, src.display(), dst.display()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Displays matched paths with pagination and returns whether to proceed with linking.
fn display_paginated_paths(
    paths: &[(PathBuf, PathBuf)],
    page_size: usize,
) -> Result<bool, MyError> {
    if paths.is_empty() {
        log::warn!("没有匹配的路径。");
        return Ok(true); // Proceed with no links to create
    }

    let total_paths = paths.len();
    let mut start = 0;

    loop {
        let end = (start + page_size).min(total_paths);
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
            return Ok(true); // All paths shown, proceed
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
            "" => start += page_size, // Enter: next page
            "a" => {
                println!("\n所有匹配的路径:\n{}", format_matched_paths(paths));
                return Ok(true); // Show all and proceed
            }
            "q" => return Ok(true), // Quit pagination and proceed
            _ => println!("无效输入，请按 Enter、'a' 或 'q'。"),
        }
    }
}

impl LinkTask {
    pub fn parse(&mut self) -> Result<(), MyError> {
        // 获取src_path
        self.check_src()?;
        // 获取dst_path
        self.check_dst()?;

        Ok(())
    }

    pub fn mklinks(&mut self) -> Result<(), MyError> {
        self.apply_re(None)?;
        match &self.re_pattern {
            Some(_) => {
                // todo 格式化输出日志与批量创建链接
                if let Some(paths) = self.matched_paths.as_ref() {
                    if paths.is_empty() {
                        log::warn!("当前Re匹配的路径为空");
                        return Ok(());
                    }
                    // 按页展示需要建立符号链接的路径对
                    if !display_paginated_paths(paths, 10)? {
                        return Ok(());
                    }

                    log::info!("符号链接创建中");
                    for (src, dst) in paths.iter() {
                        let dst = &self.dst_path.join(dst);
                        log::debug!("{} -> {}", src.display(), dst.display());
                        // todo：优化创建目录
                        mk_parents(dst)?;
                        mklink(src, dst)?;
                    }
                    log::info!("符号链接创建完成！")
                } else {
                    // log::warn!("当前Re匹配的路径为空");
                    log::error!("未应用Re检查")
                }
                Ok(())
            }
            None => {
                if self.only_dir && self.src_path.is_file() {
                    log::warn!("only_dir: {} is FILE", &self.src_path.display())
                } else if self.only_file && self.src_path.is_dir() {
                    log::warn!("only_file: {} is DIR", &self.src_path.display())
                } else {
                    log::info!(
                        "符号链接创建中\n\tsrc: {}\n\tdst: {}",
                        &self.src_path.display(),
                        &self.dst_path.display()
                    );
                    mklink(&self.src_path, &self.dst_path)?;
                    log::info!("符号链接创建成功");
                }
                Ok(())
            }
        }
    }

    /// 解析、规范、验证src, 并将结果存于src_path
    pub fn check_src(&mut self) -> Result<(), MyError> {
        let src_abs_res = dunce::canonicalize(&self.src_ori);
        if let Err(e) = src_abs_res {
            // self.error = Some(format!(
            //     "请检查<SRC>'{}'是否存在. Fail to canonicalize <SRC>: {}",
            //     &self.src_ori, e
            // ));
            // log::error!("{:?}", self.error);
            Err(MyError::new(
                ErrorCode::Unknown,
                format!(
                    "请检查<SRC>'{}'是否存在. Fail to canonicalize <SRC>: {}",
                    &self.src_ori, e
                ),
            ))
        } else {
            self.src_path = src_abs_res.unwrap();
            Ok(())
        }
    }

    pub fn check_dst(&mut self) -> Result<(), MyError> {
        // 解析dst
        let dst_path = self.parse_args_dst()?;
        // 验证dst路径父目录
        self.dst_path = self.validate_dst(&dst_path)?;
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
                // 规范化
                if src_path.is_file() && dst_path.is_dir() {
                    canonicalize_path(&dst_path.join(default_dst_path(src_path)))?
                } else {
                    canonicalize_path(&PathBuf::from(d))?
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

    /// 包装apply_re相关逻辑，通过_apply_re完成应用re检查，修改matched_paths
    pub fn apply_re(&mut self, force: Option<bool>) -> Result<(), MyError> {
        // todo 优化re Option检查
        if self.matched_paths.is_none() || force.unwrap_or(false) {
            self._apply_re();
        }
        Ok(())
    }

    /// 应用re检查
    fn _apply_re(&mut self) {
        self.matched_paths = Some(Vec::new());
        // todo 优化re Option检查
        // ---------------------------
        if self.re_pattern.is_none() {
            return;
        }
        let re = &self.re_pattern.clone().unwrap();
        // ---------------------------
        log::info!("Re: {}", re);

        // 构建路径遍历器，re_max_depth、re_follow_links相关参数于此使用
        // 直接兼容src_path是单文件或目录
        let walker = WalkDir::new(&self.src_path)
            .max_depth(get_re_max_depth(self.make_dir, self.re_max_depth))
            .follow_links(self.re_follow_links);
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            // 处理only-file与only-dir
            let is_file = entry.file_type().is_file();
            let is_dir = entry.file_type().is_dir();
            if (self.only_file && !is_file) || (self.only_dir && !is_dir) {
                continue;
            }
            let path_str = path.to_string_lossy();
            if re.is_match(&path_str) {
                if let Ok(relative_path) = path.strip_prefix(&self.src_path) {
                    // 使用相对路径节省内存空间，使用时再拼接
                    let target_path = relative_path.to_path_buf();
                    // let target_path = self.dst_path.join(relative_path);

                    if let Some(matched_paths) = self.matched_paths.as_mut() {
                        matched_paths.push((path.to_path_buf(), target_path));
                    }
                }
            }
        }
    }

    // fn get_re_max_depth(&self) -> usize {
    //     if self.make_dir {
    //         self.re_max_depth
    //     } else {
    //         log::warn!(
    //             "{} is not used for `--make_dir` is not true",
    //             self.re_max_depth
    //         );
    //         1
    //     }
    // }
}

impl TryFrom<&Args> for LinkTask {
    type Error = MyError;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        let mut task_pre = LinkTaskPre::from(args);
        task_pre.main()?;
        let task = LinkTask::try_from(task_pre)?;
        Ok(task)
    }
}

impl TryFrom<LinkTaskPre> for LinkTask {
    type Error = MyError;

    fn try_from(task_pre: LinkTaskPre) -> Result<Self, Self::Error> {
        let unwrap = |t: Option<PathBuf>| -> Result<PathBuf, Self::Error> {
            t.ok_or_else(|| {
                MyError::new(
                    ErrorCode::Unknown,
                    "Unknown error: src/dst parse fail but did not raise error".to_string(),
                )
            })
        };
        let src_path = unwrap(task_pre.src_path)?;
        let dst_path = unwrap(task_pre.dst_path)?;

        Ok(LinkTask {
            src_ori: task_pre.src_ori,
            dst_ori: task_pre.dst_ori,
            matched_paths: None,
            src_path,
            dst_path,
            re_pattern: task_pre.re_pattern,
            re_max_depth: task_pre.re_max_depth,
            re_follow_links: task_pre.re_follow_links,
            keep_extention: task_pre.keep_extention,
            make_dir: task_pre.make_dir,
            only_file: task_pre.only_file,
            only_dir: task_pre.only_dir,
            overwrite_link: task_pre.overwrite_link,
        })
    }
}

impl LinkTask {
    // pub fn new(src_ori: String, dst_ori: String, re_pattern: Option<String>) -> Self {
    //     LinkTask {
    //         src_ori,
    //         dst_ori,
    //         src_path: None,
    //         dst_path: None,
    //         matched_paths: Vec::new(),
    //         re_pattern,
    //         error: None,
    //     }
    // }
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
fn canonicalize_path(path: &Path) -> Result<PathBuf, MyError> {
    if path.is_absolute() {
        Ok(path.to_path_buf().clean())
    } else {
        let curdir = std::env::current_dir();
        match curdir {
            Ok(curdir) => Ok(curdir.join(path.clean())),
            Err(e) => Err(MyError::new(
                ErrorCode::Unknown,
                format!("Failed to get current directory: {}", e),
            )),
        }
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

/// 创建符号链接并处理错误
fn mklink(src: &PathBuf, dst: &PathBuf) -> Result<(), MyError> {
    // log::info!(
    //     "符号链接创建中: New-Item -Path '{}' -ItemType SymbolicLink -Target '{}'",
    //     src.display(),
    //     dst.display(),
    // );
    // log::info!("符号链接创建成功")
    // log::error!("符号链接创建失败: {}", e)
    create_symlink(src, dst)
}

/// 智能创建符号链接（自动判断文件/目录）
#[cfg(windows)]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), MyError> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    // 获取源文件元数据
    // let metadata = fs::metadata(src);
    let metadata = fs::metadata(src).map_err(|e| {
        MyError::new(
            ErrorCode::IoError,
            format!("无法获取源文件元数据 {}: {}", src.display(), e),
        )
    })?;

    // 根据类型选择创建方式
    if metadata.is_file() {
        symlink_file(src, dst).map_err(|e| {
            MyError::new(
                ErrorCode::FailAtMakeLink,
                format!(
                    "无法创建目录符号链接 {} -> {}: {}",
                    src.display(),
                    dst.display(),
                    e
                ),
            )
        })?;
        Ok(())
    } else if metadata.is_dir() {
        symlink_dir(src, dst).map_err(|e| {
            MyError::new(
                ErrorCode::FailAtMakeLink,
                format!(
                    "无法创建目录符号链接 {} -> {}: {}",
                    src.display(),
                    dst.display(),
                    e
                ),
            )
        })?;
        Ok(())
    } else {
        Err(MyError::new(
            ErrorCode::Unknown,
            "奇怪的错误: <SRC>既不是文件也不是目录".into(),
        ))
    }
}

#[cfg(unix)]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), MyError> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    std::os::unix::fs::symlink(src, dst).map_err(|e| {
        MyError::new(
            ErrorCode::FailAtMakeLink,
            format!(
                "无法创建符号链接 {} -> {}: {}",
                src.display(),
                dst.display(),
                e
            ),
        )
    })?;

    Ok(())
}
