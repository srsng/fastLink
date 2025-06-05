use crate::types::args::{get_re_max_depth, Args};
use crate::types::err::{ErrorCode, MyError};
use crate::types::link_task_pre::LinkTaskPre;
use crate::utils::func::mkdirs;
use crate::utils::logs::{FILE_STYLE, PARENT_STYLE};
use path_clean::PathClean;
use regex::Regex;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::os::windows::fs::{symlink_dir, symlink_file};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MAIN_SEPARATOR: char = std::path::MAIN_SEPARATOR;

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
    pub overwrite_links: bool,     // 覆盖已存在的符号链接
    pub re_no_check: bool,
    pub re_output_flatten: bool,

    pub src_path: PathBuf,                              // 规范化后的源路径
    pub dst_path: PathBuf,                              // 规范化后的目标目录路径
    pub matched_paths: Option<Vec<(PathBuf, PathBuf)>>, // 匹配的源路径和目标路径对
    pub dirs_to_create: Option<Vec<PathBuf>>,           // 需要创建的目标父目录
}

/// 格式化匹配的路径对
fn format_matched_paths(paths: &[(PathBuf, PathBuf)]) -> String {
    paths
        .iter()
        .enumerate()
        .map(|(i, (src, dst))| {
            format!(
                "{:4}. {PARENT_STYLE}{}{PARENT_STYLE:#}{MAIN_SEPARATOR}{FILE_STYLE}{:?}{FILE_STYLE:#} -> {PARENT_STYLE}{}{PARENT_STYLE:#}{MAIN_SEPARATOR}{FILE_STYLE}{:?}{FILE_STYLE:#}",
                i + 1,
                src.parent().unwrap_or_else(|| Path::new("")).display(),
                src.file_name().unwrap_or(OsStr::new("[error: Unknown]")),
                dst.parent().unwrap_or_else(|| Path::new("")).display(),
                dst.file_name().unwrap_or(OsStr::new("[error: Unknown]")),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Re匹配后、创建连接前的检查：按页展示需要建立符号链接的路径对
fn display_paginated_paths(
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
                if let Some(paths) = self.matched_paths.as_ref() {
                    if paths.is_empty() {
                        log::warn!("当前Re匹配的路径为空");
                        return Ok(());
                    }
                    // Re匹配后、创建连接前的检查：按页展示需要建立符号链接的路径对
                    if !display_paginated_paths(paths, 10, self.re_no_check)? {
                        return Ok(());
                    }
                    // 批量创建所有需要的目录
                    log::info!("创建符号链接需要目录中");
                    if self.make_dir {
                        if let Some(dirs) = self.dirs_to_create.as_ref() {
                            for dir in dirs {
                                mkdirs(dir)?;
                                log::info!("已创建目录: {}", dir.display());
                            }
                        }
                    }
                    log::info!("目录创建完成");

                    log::info!("符号链接创建中");
                    for (src, dst) in paths.iter() {
                        let dst = &self.dst_path.join(dst);
                        log::debug!("{} -> {}", src.display(), dst.display());
                        mklink(src, dst, Some(self.overwrite_links))?;
                    }
                    log::info!("符号链接创建完成！")
                } else {
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
                    mklink(&self.src_path, &self.dst_path, Some(self.overwrite_links))?;
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

    /// 包装apply_re相关逻辑，通过_apply_re完成应用re检查，修改matched_paths, dirs_to_create
    pub fn apply_re(&mut self, force: Option<bool>) -> Result<(), MyError> {
        // todo 优化regex Option检查
        if self.matched_paths.is_none() || force.unwrap_or(false) {
            self._apply_re()?;
        }
        Ok(())
    }

    /// 应用re检查
    /// 使用了参数only_file， only_dir， re_output_flatten
    fn _apply_re(&mut self) -> Result<(), MyError> {
        // todo 优化regex Option检查
        // ---------------------------
        if self.re_pattern.is_none() {
            return Ok(());
        }
        let re = self.re_pattern.clone().unwrap();
        // ---------------------------
        log::info!("Re: {}", re);

        let mut matched_paths: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut dirs_to_create: std::collections::HashSet<PathBuf> =
            std::collections::HashSet::new();
        let mut filename_counts: std::collections::HashMap<String, Vec<PathBuf>> =
            std::collections::HashMap::new();

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
                    let target_path = if self.re_output_flatten {
                        // 展平模式：仅使用文件名
                        PathBuf::from(path.file_name().unwrap_or_else(|| {
                            log::warn!("无法解析文件名称，使用 unnamed-fastlink");
                            OsStr::new("unnamed-fastlink")
                        }))
                    } else {
                        // 镜像模式：保留相对路径
                        relative_path.to_path_buf()
                    };

                    // 收集目标父目录
                    if self.make_dir {
                        let full_dst = self.dst_path.join(&target_path);
                        if let Some(parent) = full_dst.parent() {
                            if !self.re_output_flatten || dirs_to_create.is_empty() {
                                // 在展平模式下，只添加 dst_path 一次
                                dirs_to_create.insert(parent.to_path_buf());
                            }
                        }
                    }

                    // 仅在展平模式下检查重复文件名
                    if self.re_output_flatten {
                        let filename = target_path
                            .file_name()
                            .unwrap_or_else(|| OsStr::new("unnamed-fastlink"))
                            .to_string_lossy()
                            .into_owned();
                        filename_counts
                            .entry(filename)
                            .or_default()
                            .push(path.to_path_buf());
                    }

                    matched_paths.push((path.to_path_buf(), target_path));
                }
            }
        }
        // 检查重复文件名
        if self.re_output_flatten {
            let duplicates: Vec<_> = filename_counts
                .into_iter()
                .filter(|(_, paths)| paths.len() > 1)
                .collect();
            if !duplicates.is_empty() {
                let mut error_msg = String::from("检测到重复文件名，无法创建链接：\n");
                for (filename, paths) in duplicates {
                    error_msg.push_str(&format!(
                        "文件名 '{}' 出现在以下路径：\n{}\n",
                        filename,
                        paths
                            .iter()
                            .map(|p| format!("  - {}", p.display()))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ));
                }
                return Err(MyError::new(ErrorCode::InvalidInput, error_msg));
            }
        }

        self.matched_paths = Some(matched_paths);
        self.dirs_to_create = Some(dirs_to_create.into_iter().collect());
        Ok(())
    }
}

impl TryFrom<&Args> for LinkTask {
    type Error = MyError;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        let mut task_pre = LinkTaskPre::from(args);
        task_pre.parse()?;
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

            src_path,
            dst_path,
            re_pattern: task_pre.re_pattern,
            re_max_depth: task_pre.re_max_depth,
            re_follow_links: task_pre.re_follow_links,
            keep_extention: task_pre.keep_extention,
            make_dir: task_pre.make_dir,
            only_file: task_pre.only_file,
            only_dir: task_pre.only_dir,
            overwrite_links: task_pre.overwrite_links,
            re_no_check: task_pre.re_no_check,
            re_output_flatten: task_pre.re_output_flatten,
            ..Default::default()
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
fn mklink(src: &PathBuf, dst: &PathBuf, overwrite_links: Option<bool>) -> Result<(), MyError> {
    // log::info!(
    //     "符号链接创建中: New-Item -Path '{}' -ItemType SymbolicLink -Target '{}'",
    //     src.display(),
    //     dst.display(),
    // );
    // log::info!("符号链接创建成功")
    // log::error!("符号链接创建失败: {}", e)
    create_symlink(src, dst, overwrite_links.unwrap_or(false))
}

/// 检查overwrite参数选项，删除符号链接等也在此函数中
fn check_overwrite(overwrite_links: bool, dst: &Path) -> Result<(), MyError> {
    if overwrite_links && dst.exists() {
        match fs::symlink_metadata(dst) {
            Err(e) => Err(MyError::new(
                ErrorCode::FailToGetFathParent,
                format!("{} {}", e, &dst.display()),
            )),
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    fs::remove_file(dst).map_err(|e| {
                        MyError::new(
                            ErrorCode::FailAtMakeLink,
                            format!("无法删除已存在的符号链接 {}: {}", dst.display(), e),
                        )
                    })?;
                    log::info!("已删除已存在的符号链接: {}", dst.display());
                    Ok(())
                } else {
                    Err(MyError::new(
                        ErrorCode::FailAtMakeLink,
                        format!("{} 不是符号链接，无法覆盖", dst.display()),
                    ))
                }
            }
        }
    } else {
        Ok(())
    }
}

/// 智能创建符号链接（自动判断文件/目录）
#[cfg(windows)]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
    overwrite_links: bool,
) -> Result<(), MyError> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    check_overwrite(overwrite_links, dst)?;

    // 获取源文件元数据
    // let metadata = fs::metadata(src);
    let metadata = fs::metadata(src).map_err(|e| {
        MyError::new(
            ErrorCode::FailToGetFileMetadata,
            format!("无法获取源文件元数据 {}: {}", src.display(), e),
        )
    })?;

    // 根据类型选择创建方式
    if metadata.is_file() {
        symlink_file(src, dst).map_err(|e| {
            MyError::new(
                ErrorCode::FailAtMakeLink,
                format!(
                    "无法创建文件符号链接 {} -> {}: {}",
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::TempDir;

//     #[test]
//     fn test_validate_dst() {
//         let temp_dir = TempDir::new().unwrap();
//         let base_path = temp_dir.path();

//         // 父目录存在的情况
//         let existing_path = base_path.join("existing_dir");
//         fs::create_dir(&existing_path).unwrap();
//         let test_path = existing_path.join("test.txt");
//         assert!(validate_dst(&test_path, true).is_ok());

//         // 需要创建父目录的情况
//         let new_path = base_path.join("new_dir/test.txt");
//         let result = validate_dst(&new_path, true);
//         assert!(result.is_ok());
//         assert!(new_path.parent().unwrap().exists());

//         // // 无法创建父目录的情况（例如无权限路径）
//         // #[cfg(windows)]
//         // let invalid_path = Path::new("C:\\Windows\\System32\\test\\test.txt");
//         // #[cfg(not(windows))]
//         // let invalid_path = Path::new("/root/test/test.txt");
//         // assert!(validate_dst(invalid_path, true).is_err());
//     }

//     #[test]
//     fn test_process_extension() {
//         let mut path = PathBuf::from("test");
//         let src_file = Path::new("source.txt");
//         let src_dir = Path::new("source_dir");

//         // 保留扩展名（文件）
//         process_extension(src_file, &mut path, true);
//         assert_eq!(path, PathBuf::from("test.txt"));

//         // 不保留扩展名
//         path.set_file_name("test");
//         process_extension(src_file, &mut path, false);
//         assert_eq!(path, PathBuf::from("test"));

//         // 目录应忽略扩展名
//         let mut dir_path = PathBuf::from("dir/");
//         process_extension(src_file, &mut dir_path, true);
//         assert_eq!(dir_path, PathBuf::from("dir/"));

//         // 源没有扩展名
//         let mut path = PathBuf::from("test");
//         process_extension(src_dir, &mut path, true);
//         assert_eq!(path, PathBuf::from("test"));
//     }

//     #[test]
//     fn test_default_dst_path() {
//         let dir_abs = Path::new(r"C:\Windows\System32");
//         let dir_rel = Path::new(r"System32");
//         let dir_rel2 = Path::new(r"\Windows\System32");
//         let dir_rel3 = Path::new(r"..\System32");

//         assert_eq!(PathBuf::from("System32"), default_dst_path(dir_abs));
//         assert_eq!(PathBuf::from("System32"), default_dst_path(dir_rel));
//         assert_eq!(PathBuf::from("System32"), default_dst_path(dir_rel2));
//         assert_eq!(PathBuf::from("System32"), default_dst_path(dir_rel3));

//         let file_abs = Path::new(r"C:\Windows\System32\notepad.exe");
//         let file_rel = Path::new(r"notepad.exe");
//         let file_rel2 = Path::new(r"System32\notepad.exe");
//         let file_rel3 = Path::new(r"..\notepad.exe");
//         assert_eq!(PathBuf::from("notepad"), default_dst_path(file_abs));
//         assert_eq!(PathBuf::from("notepad"), default_dst_path(file_rel));
//         assert_eq!(PathBuf::from("notepad"), default_dst_path(file_rel2));
//         assert_eq!(PathBuf::from("notepad"), default_dst_path(file_rel3));

//         assert_eq!(
//             PathBuf::from("unnamed-fastlink"),
//             default_dst_path(Path::new(""))
//         );
//     }

//     #[test]
//     fn test_parse_args_dst() {
//         let dir_abs = r"C:\Windows\System32";
//         let dir_rel = r"System32";

//         let file_abs = r"C:\Windows\System32\notepad.exe";
//         let file_rel = r"notepad.exe";
//         // block: dst is None
//         {
//             let dir_tar = PathBuf::from("System32");
//             let file_tar_k_t = PathBuf::from("notepad.exe");
//             let file_tar_k_f = PathBuf::from("notepad");

//             // keep_extention true
//             assert_eq!(dir_tar, parse_args_dst(dir_abs, None, true));
//             assert_eq!(dir_tar, parse_args_dst(dir_rel, None, true));
//             assert_eq!(file_tar_k_t, parse_args_dst(file_abs, None, true));
//             assert_eq!(file_tar_k_t, parse_args_dst(file_rel, None, true));
//             // no keep_extention false
//             assert_eq!(dir_tar, parse_args_dst(dir_abs, None, false));
//             assert_eq!(dir_tar, parse_args_dst(dir_rel, None, false));
//             assert_eq!(file_tar_k_f, parse_args_dst(file_abs, None, false));
//             assert_eq!(file_tar_k_f, parse_args_dst(file_rel, None, false));
//         }

//         // block: dst not None, relative path
//         {
//             let some_dst = Some(r"..\some_name");
//             let cur_path = std::env::current_dir().expect("Failed to get current directory");

//             let dir_tar = cur_path.join(PathBuf::from(r"..\some_name"));
//             let file_tar_k_t = cur_path.join(PathBuf::from(r"..\some_name.exe"));
//             let file_tar_k_f = cur_path.join(PathBuf::from(r"..\some_name"));

//             // keep_extention true
//             assert_eq!(dir_tar, parse_args_dst(dir_abs, some_dst, true));
//             assert_eq!(dir_tar, parse_args_dst(dir_rel, some_dst, true));
//             assert_eq!(file_tar_k_t, parse_args_dst(file_abs, some_dst, true));
//             assert_eq!(file_tar_k_t, parse_args_dst(file_rel, some_dst, true));
//             // no keep_extention false
//             assert_eq!(dir_tar, parse_args_dst(dir_abs, some_dst, false));
//             assert_eq!(dir_tar, parse_args_dst(dir_rel, some_dst, false));
//             assert_eq!(file_tar_k_f, parse_args_dst(file_abs, some_dst, false));
//             assert_eq!(file_tar_k_f, parse_args_dst(file_rel, some_dst, false));
//         }
//         // block: dst not None, absolute path
//         {
//             let some_dst = Some(r"C:\some_name");

//             let dir_tar = PathBuf::from(r"C:\some_name");
//             let file_tar_k_t = PathBuf::from(r"C:\some_name.exe");
//             let file_tar_k_f = PathBuf::from(r"C:\some_name");

//             // keep_extention true
//             assert_eq!(dir_tar, parse_args_dst(dir_abs, some_dst, true));
//             assert_eq!(dir_tar, parse_args_dst(dir_rel, some_dst, true));
//             assert_eq!(file_tar_k_t, parse_args_dst(file_abs, some_dst, true));
//             assert_eq!(file_tar_k_t, parse_args_dst(file_rel, some_dst, true));
//             // no keep_extention false
//             assert_eq!(dir_tar, parse_args_dst(dir_abs, some_dst, false));
//             assert_eq!(dir_tar, parse_args_dst(dir_rel, some_dst, false));
//             assert_eq!(file_tar_k_f, parse_args_dst(file_abs, some_dst, false));
//             assert_eq!(file_tar_k_f, parse_args_dst(file_rel, some_dst, false));
//         }
//     }

//     #[test]
//     fn test_create_symlink() {
//         let temp_dir = TempDir::new().unwrap();
//         let src_file = temp_dir.path().join("source.txt");
//         let src_dir = temp_dir.path().join("source_dir");
//         let dst_file = temp_dir.path().join("link.txt");
//         let dst_dir = temp_dir.path().join("link_dir");

//         // 创建测试文件/目录
//         fs::write(&src_file, "fastlink test").unwrap();
//         fs::create_dir(&src_dir).unwrap();

//         // 测试文件符号链接
//         assert!(create_symlink(&src_file, &dst_file).is_ok());
//         assert!(dst_file.exists());

//         // 测试目录符号链接
//         assert!(create_symlink(&src_dir, &dst_dir).is_ok());
//         assert!(dst_dir.exists());

//         // 清理
//         fs::remove_file(dst_file).unwrap();
//         fs::remove_dir(dst_dir).unwrap();
//     }
// }
