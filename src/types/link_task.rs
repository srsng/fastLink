use crate::types::args::Args;
use crate::types::err::{ErrorCode, MyError, MyResult};
use crate::types::link_task_args::{LinkTaskArgs, LinkTaskOpMode};
use crate::types::link_task_pre::LinkTaskPre;
use crate::utils::func::mklink_pre_check;
use crate::utils::link::{del_exists_link, mklink};
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

/// 负责创建/移除/检查前的Re匹配与实际处理
/// 可通过try_new或try_from构建
#[derive(Debug)]
pub struct LinkTask {
    pub args: LinkTaskArgs,
    pub src_path: PathBuf,                              // 规范化后的源路径
    pub dst_path: PathBuf,                              // 规范化后的目标目录路径
    pub matched_paths: Option<Vec<(PathBuf, PathBuf)>>, // 匹配的源相对路径和目标相对路径
    pub dirs_to_create: Option<Vec<PathBuf>>,           // 需要创建的目标父目录相对路径
}

impl LinkTask {
    pub fn work(mut self) -> MyResult<()> {
        match self.args.op_mode {
            LinkTaskOpMode::Check => {
                log::info!("[check模式 (--check)]");
                self.check_links()
            }
            LinkTaskOpMode::Remove => {
                log::info!("[rm模式 (--rm)]");
                self.remove_links()
            }
            LinkTaskOpMode::Make => self.mklinks(),
        }
    }

    pub fn remove_links(mut self) -> MyResult<()> {
        let log = |b: bool| {
            if b {
                log::info!("删除符号链接成功: {}", &self.src_path.display());
            } else {
                log::warn!("删除符号链接失败: {}", &self.src_path.display());
            }
        };
        // 没有传入dst，使用src，（不用apply re后的）
        if self.args.dst.is_none() {
            del_exists_link(&self.src_path, true, Some(false)).map(log)
            // 有dst用dst
        } else {
            #[cfg(feature = "regex")]
            {
                if self.args.re_pattern.is_some() {
                    self.apply_re(None)?;

                    // 错误数据与跳过的路径
                    let mut errs = Vec::new();
                    let mut skip = Vec::new();

                    // 删除链接并记录数据
                    for (_src, dst) in self.matched_paths.as_deref().unwrap() {
                        let dst = self.dst_path.join(dst);
                        match del_exists_link(&dst, true, Some(false)) {
                            Ok(b) => {
                                if !b {
                                    skip.push(dst)
                                }
                            }
                            Err(e) => errs.push(e),
                        }
                    }
                    // 日志输出信息
                    self.remove_links_log(skip, errs);
                } else {
                    del_exists_link(&self.dst_path, true, Some(false)).map(log)?;
                }
            }
            #[cfg(not(feature = "regex"))]
            {
                del_exists_link(&self.dst_path, true, Some(false)).map(log)?;
            }
            Ok(())
        }
    }

    #[cfg(feature = "regex")]
    fn remove_links_log(self, skip: Vec<PathBuf>, errs: Vec<MyError>) {
        log::info!(
            "删除完成：{}条成功，{}条跳过，{}条失败",
            self.matched_paths.unwrap().len() - skip.len() - errs.len(),
            errs.len(),
            skip.len(),
        );
        if !errs.is_empty() {
            log::info!(
                "错误如下:\n{}",
                errs.iter()
                    .map(|e| { e.to_string() })
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
        }
        if !skip.is_empty() {
            log::info!(
                "跳过的路径如下:\n{}",
                skip.iter()
                    .map(|t| format!("{}", t.display()))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        }
    }

    pub fn check_links(mut self) -> MyResult<()> {
        // 没有传入dst，使用src
        if self.args.dst.is_none() {
            check_link(&self.src_path)
        // 有dst用dst
        } else {
            #[cfg(feature = "regex")]
            {
                if self.args.re_pattern.is_some() {
                    self.apply_re(None)?;
                    for (_src, dst) in self.matched_paths.unwrap() {
                        check_link(&self.dst_path.join(dst))?;
                    }
                } else {
                    check_link(&self.dst_path)?;
                }
            }
            #[cfg(not(feature = "regex"))]
            check_link(&self.dst_path)?;

            Ok(())
        }
    }

    #[cfg(not(feature = "regex"))]
    pub fn mklinks(&mut self) -> Result<(), MyError> {
        self._mklink()
    }

    #[cfg(feature = "regex")]
    pub fn mklinks(&mut self) -> Result<(), MyError> {
        self.apply_re(None)?;
        match &self.args.re_pattern {
            Some(_) => self._mklinks_re(),
            None => self._mklink(),
        }
    }

    #[cfg(feature = "regex")]
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
                    for dir in dirs.iter().filter(|dir| !dir.exists()) {
                        if create_dir_cnt == 0 {
                            log::info!("创建符号链接需要目录中");
                        }
                        let full_dir = self.dst_path.join(dir);
                        crate::utils::fs::mkdirs(&full_dir)?;
                        create_dir_cnt += 1;
                        log::info!("已创建目录: {}", full_dir.display());
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
                    Some(self.args.allow_broken_src),
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
            log::debug!(
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
                Some(self.args.allow_broken_src),
            )?;
            log::info!("符号链接创建成功");
        }
        Ok(())
    }

    #[cfg(feature = "regex")]
    /// 包装apply_re相关逻辑，通过_apply_re完成应用re检查，修改matched_paths, dirs_to_create
    pub fn apply_re(&mut self, force: Option<bool>) -> Result<(), MyError> {
        // todo 优化regex Option检查
        if self.matched_paths.is_none() || force.unwrap_or(false) {
            self._apply_re()?;
        }
        // log::debug!("{:?}", self.matched_paths);
        // if let Some(paths) = self.matched_paths.as_mut() {
        //     if paths[0] == (PathBuf::new(), PathBuf::new()) {
        //         paths.remove(0);
        //         log::warn!("经过re匹配后的路径对包含src与dst，已排除");
        //     }
        // }
        Ok(())
    }

    // #[cfg(not(feature = "regex"))]
    // /// 为了统一函数调用的非regex特性下的apply_re，空操作且不需要可变借用
    // pub fn apply_re(&self, _force: Option<bool>) -> Result<(), MyError> {
    //     Ok(())
    // }

    #[cfg(feature = "regex")]
    /// 应用re检查，更新 matched_paths 和 dirs_to_create
    /// 使用了参数only_file， only_dir， re_output_flatten
    fn _apply_re(&mut self) -> Result<(), MyError> {
        // todo 优化regex Option检查
        // ---------------------------
        if self.args.re_pattern.is_none() {
            return Ok(());
        }
        let re = self.args.re_pattern.as_ref().unwrap();
        // ---------------------------
        log::info!("Re: {}", re);

        let mut matched_paths: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut dirs_to_create: std::collections::HashSet<PathBuf> =
            std::collections::HashSet::new();
        let mut target_paths: std::collections::HashMap<PathBuf, Vec<PathBuf>> =
            std::collections::HashMap::new();
        let mut matched_paths_dir: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut max_observed_depth: usize = 0;

        let max_depth =
            crate::types::args::get_re_max_depth(self.args.make_dir, self.args.re_max_depth);
        // 构建路径遍历器，re_max_depth、re_follow_links相关参数于此使用
        // 直接兼容src_path是单文件或目录
        let walker = walkdir::WalkDir::new(&self.src_path)
            .max_depth(max_depth)
            .follow_links(self.args.re_follow_links);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let depth = entry.depth();

            let is_file = entry.file_type().is_file();
            let is_dir = entry.file_type().is_dir();

            // 处理 only_file 和 only_dir 参数
            if (self.args.only_file && !is_file) || (self.args.only_dir && !is_dir) {
                continue;
            }

            let path_str = path.to_string_lossy();
            if re.is_match(&path_str) {
                if let Ok(relative_path) = path.strip_prefix(&self.src_path) {
                    // 使用相对路径节省内存空间，使用时再拼接
                    let target_path = if self.args.re_output_flatten {
                        // 展平模式：仅使用文件名
                        if let Some(file_name) = path.file_name() {
                            PathBuf::from(file_name)
                        } else {
                            log::warn!("无法解析文件名称，已跳过: {}", path.display());
                            continue;
                        }
                    } else {
                        // 镜像模式：保留相对路径
                        relative_path.to_path_buf()
                    };
                    // dst完全路径
                    let full_dst = self.dst_path.join(&target_path);

                    if is_file {
                        // 添加创建文件夹
                        if let Some(parent) = target_path.parent() {
                            dirs_to_create.insert(parent.to_path_buf());
                        }

                        // 文件直接加入 matched_paths
                        matched_paths.push((relative_path.to_path_buf(), target_path.clone()));
                        // 收集目标路径以检查重复
                        target_paths
                            .entry(full_dst.clone())
                            .or_default()
                            .push(path.to_path_buf());
                    } else if is_dir {
                        // 发现更深的层级，清空之前的符号链接记录
                        if depth > max_observed_depth {
                            // log::debug!("matched_paths_dir clear before {:?}", matched_paths_dir);
                            // 发现更深的层级，将之前的 matched_paths_dir 转移到 dirs_to_create
                            for (_, target) in &matched_paths_dir {
                                dirs_to_create.insert(target.clone());
                            }
                            // log::debug!("dirs_to_create clear after {:?}", dirs_to_create);
                            // 清空 matched_paths_dir 并更新 max_observed_depth
                            matched_paths_dir.clear();
                            max_observed_depth = depth;
                        }

                        // 当前层级是最深层，为目录创建符号链接
                        if depth == max_observed_depth {
                            matched_paths_dir
                                .push((relative_path.to_path_buf(), target_path.clone()));
                            target_paths
                                .entry(full_dst.clone())
                                .or_default()
                                .push(path.to_path_buf());
                        }
                        // else if self.args.make_dir {
                        // 非最深层目录，创建文件夹
                        // log::debug!("is dir={} : {}", is_dir, path.display());
                        // dirs_to_create.insert(target_path.clone());
                        // }
                    }
                }
            }
        }

        dirs_to_create.remove(&PathBuf::new());
        // log::debug!("dirs_to_create {:?}", dirs_to_create);

        // 将最深层目录的符号链接添加到 matched_paths
        matched_paths.extend(matched_paths_dir);
        // 处理展平模式下的重复目标路径
        if self.args.re_output_flatten {
            let duplicates: Vec<_> = target_paths
                .iter()
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
        log::debug!("已从LinkTaskPre构建LinkTask");
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

fn check_link(src: &Path) -> MyResult<()> {
    match mklink_pre_check(src) {
        Ok(_) => (),
        Err(e) if e.code == ErrorCode::TargetExistsAndNotLink => {
            let filetype = if src.is_dir() {
                "DIR "
            } else if src.is_file() {
                "FILE"
            } else {
                "UNKOWN"
            };
            log::info!("{:7} {}", filetype, src.display(),);
        }
        Err(e) if e.code == ErrorCode::BrokenSymlink => {
            log::warn!("SymLink(损坏) {}", src.display())
        }
        Err(e) if e.code == ErrorCode::FileNotExist => {
            log::warn!("不存在 {}", src.display())
        }
        Err(e) if e.code == ErrorCode::TargetLinkExists => {
            let target = std::fs::read_link(src);
            match target {
                Ok(dst) => log::info!("SymLink {:7} 指向 {}", src.display(), dst.display()),
                Err(e) => log::error!("SymLink {} 指向未知，获取时出错：{}", src.display(), e),
            };
        }
        Err(e) => log::warn!("错误：检查 {} 时发生未知错误: {}", src.display(), e),
    };
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
