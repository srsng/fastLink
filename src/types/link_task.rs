use crate::types::args::{get_re_max_depth, Args};
use crate::types::err::{ErrorCode, MyError, MyResult};
use crate::types::link_task_args::LinkTaskArgs;
use crate::types::link_task_pre::{handle_mklink_pre_check_error_for_src, LinkTaskPre};
use crate::utils::func::{mkdirs, mklink_pre_check};

use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs;
use std::os::windows::fs::{symlink_dir, symlink_file};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 只创建链接以及创建时的相关处理
/// 可通过try_new或try_from构建
#[derive(Debug, Default)]
pub struct LinkTask {
    pub args: LinkTaskArgs,
    pub src_path: PathBuf,                              // 规范化后的源路径
    pub dst_path: PathBuf,                              // 规范化后的目标目录路径
    pub matched_paths: Option<Vec<(PathBuf, PathBuf)>>, // 匹配的源相对路径和目标相对路径
    pub dirs_to_create: Option<Vec<PathBuf>>,           // 需要创建的目标父目录
}

impl LinkTask {
    pub fn mklinks(&mut self) -> Result<(), MyError> {
        self.apply_re(None)?;
        match &self.args.re_pattern {
            Some(_) => self._mklinks_re(),
            None => self._mklink(),
        }
    }

    fn _mklinks_re(&self) -> Result<(), MyError> {
        if self
            .matched_paths
            .as_ref()
            .is_none_or(|paths| paths.is_empty())
        {
            log::warn!("当前Re匹配后的路径为空");
            return Ok(());
        }

        if let Some(paths) = self.matched_paths.as_ref() {
            // Re匹配后、创建连接前的检查：按页展示需要建立符号链接的路径对
            // 返回Ok(false)则取消创建
            if !crate::utils::func::display_paginated_paths(paths, 10, self.args.re_no_check)? {
                return Ok(());
            }

            // 批量创建所有需要的目录
            let mut create_dir_cnt: usize = 0;
            if self.args.make_dir {
                if let Some(dirs) = self.dirs_to_create.as_ref() {
                    for dir in dirs {
                        if dir.exists() {
                            continue;
                        }
                        if create_dir_cnt == 0 {
                            log::info!("创建符号链接需要目录中");
                        }
                        mkdirs(dir)?;
                        create_dir_cnt += 1;
                        log::info!("已创建目录: {}", dir.display());
                    }
                }
            }
            if create_dir_cnt == 0 {
                log::info!("没有需要创建的目录");
            } else {
                log::info!("目录创建完成, 共创建{}条目录", create_dir_cnt);
            }

            log::info!("开始创建符号链接");
            for (i, (src, dst)) in paths.iter().enumerate() {
                let src = &self.src_path.join(src);
                let dst = &self.dst_path.join(dst);

                log::debug!(
                    "Try mklink [{}]: \n\tsrc={} \n\tdst={}",
                    i,
                    src.display(),
                    dst.display()
                );
                mklink(
                    src,
                    dst,
                    Some(self.args.overwrite_links),
                    Some(self.args.overwrite_broken_link),
                    Some(self.args.skip_exist_links),
                    Some(self.args.skip_broken_src_links),
                )?;
            }
            log::info!("符号链接创建完成！");

            Ok(())
        } else {
            Err(MyError::new(
                ErrorCode::Unknown,
                "Unknown Error: 初始化后的路径对列表为None".into(),
            ))
        }
    }

    fn _mklink(&self) -> Result<(), MyError> {
        if self.args.only_dir && self.src_path.is_file() {
            log::warn!("only_dir: {} is FILE", &self.src_path.display())
        } else if self.args.only_file && self.src_path.is_dir() {
            log::warn!("only_file: {} is DIR", &self.src_path.display())
        } else {
            log::info!(
                "符号链接创建中\n\tsrc: {}\n\tdst: {}",
                &self.src_path.display(),
                &self.dst_path.display()
            );
            mklink(
                &self.src_path,
                &self.dst_path,
                Some(self.args.overwrite_links),
                Some(self.args.overwrite_broken_link),
                Some(self.args.skip_exist_links),
                Some(self.args.skip_broken_src_links),
            )?;
            log::info!("符号链接创建成功");
        }
        Ok(())
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
        if self.args.re_pattern.is_none() {
            return Ok(());
        }
        let re = self.args.re_pattern.clone().unwrap();
        // ---------------------------
        log::info!("Re: {}", re);

        let mut matched_paths: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut dirs_to_create: std::collections::HashSet<PathBuf> =
            std::collections::HashSet::new();
        let mut target_paths: std::collections::HashMap<PathBuf, Vec<PathBuf>> =
            std::collections::HashMap::new();

        // 构建路径遍历器，re_max_depth、re_follow_links相关参数于此使用
        // 直接兼容src_path是单文件或目录
        let walker = WalkDir::new(&self.src_path)
            .max_depth(get_re_max_depth(self.args.make_dir, self.args.re_max_depth))
            .follow_links(self.args.re_follow_links);
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            // 处理参数 only_file 和 only_dir
            let is_file = entry.file_type().is_file();
            let is_dir = entry.file_type().is_dir();
            if (self.args.only_file && !is_file) || (self.args.only_dir && !is_dir) {
                continue;
            }

            let path_str = path.to_string_lossy();
            if re.is_match(&path_str) {
                if let Ok(relative_path) = path.strip_prefix(&self.src_path) {
                    // 使用相对路径节省内存空间，使用时再拼接
                    let target_path = if self.args.re_output_flatten {
                        // 展平模式：仅使用文件名
                        PathBuf::from(path.file_name().unwrap_or_else(|| {
                            log::warn!("无法解析文件名称，使用 unnamed-fastlink");
                            OsStr::new("unnamed-fastlink")
                        }))
                    } else {
                        // 镜像模式：保留相对路径
                        relative_path.to_path_buf()
                    };

                    // 计算完整的目标路径
                    let full_dst = self.dst_path.join(&target_path);

                    // 收集目标路径以检查重复
                    target_paths
                        .entry(full_dst.clone())
                        .or_default()
                        .push(path.to_path_buf());
                    // 收集目标父目录
                    if self.args.make_dir {
                        let full_dst = self.dst_path.join(&target_path);
                        if let Some(parent) = full_dst.parent() {
                            if !self.args.re_output_flatten || dirs_to_create.is_empty() {
                                // 在展平模式下，只添加 dst_path 一次
                                dirs_to_create.insert(parent.to_path_buf());
                            }
                        }
                    }

                    matched_paths.push((relative_path.to_path_buf(), target_path));
                }
            }
        }

        // 检查重复目标路径
        if self.args.re_output_flatten {
            let duplicates: Vec<_> = target_paths
                .into_iter()
                .filter(|(_, paths)| paths.len() > 1)
                .collect();
            if !duplicates.is_empty() {
                let mut error_msg = String::from("检测到重复目标路径，无法创建链接：\n");
                for (target_path, src_paths) in duplicates {
                    error_msg.push_str(&format!(
                        "目标路径 '{}' 对应以下源路径：\n{}\n",
                        target_path.display(),
                        src_paths
                            .iter()
                            .map(|p| format!("  - {}", p.display()))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ));
                }
                return Err(MyError::new(ErrorCode::DuplicateTarget, error_msg));
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

    fn try_from(mut task_pre: LinkTaskPre) -> Result<Self, Self::Error> {
        // src/dst为None则再次解析一次
        if task_pre.src_path.is_none() || task_pre.dst_path.is_none() {
            match task_pre.parse() {
                Err(mut e) => {
                    e.msg = format!(
                        "Fail to parse src/dst when convert LinkTaskPre to LinkTask: {}",
                        e.msg
                    );
                    Err(e)
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }?;

        Ok(LinkTask {
            args: task_pre.args,
            src_path: task_pre.src_path.unwrap(),
            dst_path: task_pre.dst_path.unwrap(),
            matched_paths: None,
            dirs_to_create: None,
        })
    }
}

impl LinkTask {
    /// 创建一个完成已经解析src、dst的LinkTask
    pub fn try_new(args: LinkTaskArgs) -> MyResult<Self> {
        let task_pre = LinkTaskPre::try_new(args)?;
        let task = LinkTask::try_from(task_pre)?;
        Ok(task)
    }
}

/// 创建符号链接并处理错误
/// Ok(false)表示跳过创建
fn mklink(
    src: &PathBuf,
    dst: &PathBuf,
    overwrite_links: Option<bool>,
    overwrite_broken_links: Option<bool>,
    skip_exist_links: Option<bool>,
    skip_broken_src_links: Option<bool>,
) -> Result<bool, MyError> {
    let overwrite_links = overwrite_links.unwrap_or(false);
    let overwrite_broken_links = overwrite_broken_links.unwrap_or(true);
    let skip_exist_links = skip_exist_links.unwrap_or(false);
    let skip_broken_src_links = skip_broken_src_links.unwrap_or(true);

    // log::info!(
    //     "符号链接创建中: New-Item -Path '{}' -ItemType SymbolicLink -Target '{}'",
    //     src.display(),
    //     dst.display(),
    // );
    // （实际当然不是用上面这个命令创建symlink

    // 检查src
    let res = mklink_pre_check(src);
    match handle_mklink_pre_check_error_for_src(res) {
        Ok(_) => Ok(()),
        Err(e) if e.code == ErrorCode::BrokenSymlink && skip_broken_src_links => {
            log::warn!("src部分为损坏的符号链接，已跳过: {}", src.display());
            return Ok(false);
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
            log::info!("创建符号链接: {} -> {}", src.display(), dst.display());
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
        del_exists_link(dst, overwrite_links)
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
) -> Result<(), MyError> {
    if let Some(mut e) = res.err() {
        match e.code {
            ErrorCode::FileNotExist => Ok(()),
            ErrorCode::TargetExistsAndNotLink => {
                // e.log();
                e.msg = format!("无法创建链接：src部分为损坏的符号链接 {}", e.msg);
                Err(e)
            }
            // 确定目标路径已存在符号链接，需要考虑覆写/跳过
            ErrorCode::TargetLinkExists => {
                e.warn();
                handle_exists_link(dst, overwrite_links, skip_exist_links)
            }
            // 确定目标路径已存在且损坏的符号链接，需要考虑覆写/跳过
            ErrorCode::BrokenSymlink => {
                e.warn();
                // overwrite参数满足其一即可
                let cond = overwrite_links || overwrite_broken_links;
                handle_exists_link(dst, cond, skip_exist_links)
            }
            _ => Err(e),
        }
    } else {
        Ok(())
    }
}

/// 删除符号链接，需要传入overwrite_links参数，避免误用
pub fn del_exists_link(dst: &Path, overwrite_links: bool) -> Result<(), MyError> {
    if overwrite_links {
        // 检查是否是symlink
        match mklink_pre_check(dst) {
            Err(e) if e.code == ErrorCode::TargetLinkExists => (),
            Err(e) if e.code == ErrorCode::BrokenSymlink => (),
            Err(e) if e.code == ErrorCode::FileNotExist => (),
            Err(mut e) => {
                e.msg = format!("尝试删除非符号链接路径 {}", e.msg);
                return Err(e);
            }
            e => return e,
        };
        // 区分符号链接类型
        if dst.is_dir() {
            fs::remove_dir(dst).map_err(|e| {
                MyError::new(
                    ErrorCode::FailToDelLink,
                    format!("(DIR) {}: {}", dst.display(), e),
                )
            })?;
            log::info!("已删除符号链接 {}", dst.display());
            Ok(())
        } else if dst.is_file() {
            fs::remove_file(dst).map_err(|e| {
                MyError::new(
                    ErrorCode::FailToDelLink,
                    format!("(FILE){}: {}", dst.display(), e),
                )
            })?;
            log::info!("已删除符号链接 {}", dst.display());
            Ok(())
        } else {
            log::warn!("损坏的符号链接 {}, 尝试作为文件删除", dst.display());
            let res_file = fs::remove_file(dst);
            if res_file.is_err() {
                log::warn!("删除失败: {}，尝试作为目录删除", dst.display());
                let res_dir = fs::remove_dir(dst).map_err(|_| {
                    MyError::new(
                        ErrorCode::FailToDelLink,
                        format!(
                            "(Unkown){}: 未知类型的符号链接，无法删除，请尝试手动删除",
                            dst.display()
                        ),
                    )
                });
                if res_dir.is_err() {
                    log::warn!("作为目录删除失败: {}", dst.display());
                    res_dir
                } else {
                    log::info!("已删除符号链接 {}", dst.display());
                    Ok(())
                }
            } else {
                log::info!("已删除符号链接 {}", dst.display());
                Ok(())
            }
        }
    } else {
        Ok(())
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
        let res = symlink_file(src, dst);
        convert_create_symlink_res(res, src, dst)
    } else if metadata.is_dir() {
        let res = symlink_dir(src, dst);
        convert_create_symlink_res(res, src, dst)
    } else {
        Err(MyError::new(
            ErrorCode::Unknown,
            "奇怪的错误: <SRC>既不是文件也不是目录，可能是损坏的符号链接或别的什么".into(),
        ))
    }
}

/// 转换create_symlink中创建符号链接的res
pub fn convert_create_symlink_res(
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
