// todo: 参数实现 overwrite_link save_log

use clap::Parser;
use std::path::Path;
pub mod types;
pub mod utils;
use types::args::Args;
use utils::logs::init_log;

use crate::types::link_task::LinkTask;
// use crate::types::link_task_pre::LinkTaskPre;
// const PATHEXT: &str = ".EXE;.COM;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC";

// /// 获取系统可执行扩展名列表
// fn get_executable_extensions() -> Vec<String> {
//     let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| PATHEXT.to_string());

//     pathext
//         .split(';')
//         .map(|s| s.trim().to_lowercase())
//         .filter(|s| !s.is_empty())
//         .collect()
// }

// // windows隐藏命令行窗口参数
// const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() {
    let args: Args = Args::parse();

    // 初始化日志系统
    init_log(args.quiet, args.debug, &args.save_log);
    log::debug!("{:?}", args);

    special_warn(&args);

    let task_res = LinkTask::try_from(&args);
    task_res
        .map_err(|e| e.log())
        .map(|mut task| task.mklinks().map_err(|e| e.log()));
}

// /// 处理validate_dst的错误
// fn handle_validate_dst_err(validate_dst_res: &Result<PathBuf, MyError>) -> bool {
//     (*validate_dst_res).as_ref().map_or_else(
//         |e| {
//             if let ErrorCode::ParentNotExist = e.code {
//                 log::error!("{}", e.msg);
//             } else {
//                 log::error!(
//                     "[DST]父目录不存在，且创建失败，请尝试手动创建或修改路径: \n{}",
//                     e.msg
//                 );
//             }
//             false
//         },
//         |_| true,
//     )
// }

/// 对一些特殊情况进行警告
fn special_warn(args: &Args) {
    let src = &args.src;
    let src_path = Path::new(src);
    let dst = &args.dst;
    let keep_extention = args.keep_extention;

    if keep_extention && dst.is_none() {
        log::warn!("不给定[DST]的同时使用-k，通常会因该目录下已有同名文件而创建失败！");
    }

    if src == "." && dst.is_none() {
        log::warn!("这样做会在当前目录创建，对该目录本身的符号链接，如果你不清楚自己这样做的后果，请不要这么做!（这个时候可能已经创建成功了，那么就快点删除它！）")
    }

    // dst不为空的情况
    if let Some(dst) = dst {
        let dst_path = Path::new(dst);

        if src_path.is_file() && dst_path.is_dir() {
            log::warn!("<SRC>为文件路径而[DST]为目录路径，将自动使用<SRC>文件名追加到[DST]")
        }

        // if src == ".." && dst.is_relative() {
        //     log::warn!("这样做会在当前目录创建，对该目录的父目录的符号链接，如果你不清楚自己这样做的后果，请不要这么做!（这个时候可能已经创建成功了，那么就快点删除它！）")
        // }
    }
}

// /// 返回规范化后的dst绝对路径
// /// 若其父目录不存在且make_dir为false，则将返回Err
// fn validate_dst(dst: &Path, make_dir: bool) -> Result<PathBuf, MyError> {
//     log::debug!("validate_dst/dst: {}", dst.display());

//     let dst_parent_option = dst.parent();
//     // dst父目录不存在
//     handle_validate_dst_parent_not_exist(dst, make_dir, dst_parent_option)
// }

// /// 为validate_dst函数处理dst父目录不存在的情况
// fn handle_validate_dst_parent_not_exist(
//     dst: &Path,
//     make_dir: bool,
//     dst_parent_option: Option<&Path>,
// ) -> Result<PathBuf, MyError> {
//     if dst_parent_option.is_some() && !dst_parent_option.unwrap().exists() {
//         let dst_parent = dst_parent_option.unwrap().clean();
//         if make_dir {
//             // 创建目录并处理错误
//             Ok(handle_validate_dst_mkdirs(dst, dst_parent)?)
//         } else {
//             Err(MyError::new(
//                 ErrorCode::ParentNotExist,
//                 format!(
//                     "[DST]父目录: {} 不存在，若需自动创建请添加参数--make-dir或--md",
//                     dst_parent.display()
//                 ),
//             ))
//         }
//     } else {
//         Ok(dst.to_path_buf())
//     }
// }

// /// 为validate_dst函数（handle_validate_dst_parent_not_exist函数）处理创建目录及相关错误
// fn handle_validate_dst_mkdirs(dst: &Path, dst_parent: PathBuf) -> Result<PathBuf, MyError> {
//     match mkdirs(&dst_parent) {
//         Ok(_) => {
//             log::warn!("[DST]父目录不存在，已创建: {}", dst_parent.display());
//             // 重新组合dst路径
//             let dst_path = if let Some(dst_filename) = dst.file_name() {
//                 dst_parent.join(dst_filename)
//             } else {
//                 dst.to_path_buf()
//             };
//             log::debug!("validate_dst/dst return: {}", dst_path.display());
//             Ok(dst_path)
//         }
//         Err(e) => Err(MyError::new(
//             ErrorCode::Unknown,
//             format!(
//                 "[DST]父目录: {} 创建失败\n\tErrorMsg: {}",
//                 dst_parent.display(),
//                 e
//             ),
//         )),
//     }
// }

// /// 创建目录
// fn mkdirs(path: &Path) -> Result<(), String> {
//     let res = std::fs::create_dir_all(path);
//     match res {
//         Err(e) => Err(format!("{}", e)),
//         _ => Ok(()),
//     }
// }

// /// 解析dst参数并转化为路径
// /// 为[DST]自动追加<SRC>名称、拓展名都在这实现
// fn parse_args_dst(src: &str, dst: Option<&str>, keep_extention: bool) -> PathBuf {
//     let src_path = Path::new(src);
//     let mut final_dst = match dst {
//         Some(d) => {
//             // SRC是文件而DST是目录的情况: 为DST追加SRC文件名
//             let dst_path = Path::new(d);
//             if src_path.is_file() && dst_path.is_dir() {
//                 canonicalize_path(&dst_path.join(default_dst_path(src_path)))
//             } else {
//                 canonicalize_path(&PathBuf::from(d))
//             }
//         }
//         // 没有传入DST: 使用SRC文件名
//         None => default_dst_path(src_path),
//     };

//     // 处理keep_extension: 是否保留拓展名
//     process_extension(src_path, &mut final_dst, keep_extention);

//     final_dst
// }

// /// 生成默认目标路径
// fn default_dst_path(src: &Path) -> PathBuf {
//     let base_name = src.file_stem().unwrap_or_else(|| {
//         src.file_name().unwrap_or_else(|| {
//             log::warn!("无法解析src名称，已设置dst名称为unnamed-fastlink");
//             OsStr::new("unnamed-fastlink")
//         })
//     });

//     // 输出日志信息
//     log::info!(
//         "已由<SRC>确定目标名 {} → {}",
//         src.display(),
//         base_name.to_string_lossy()
//     );

//     PathBuf::from(base_name)
// }

// /// 扩展名处理逻辑（统一处理相对/绝对路径）, 调用后直接修改传入的dst
// fn process_extension(src: &Path, dst: &mut PathBuf, keep_extention: bool) {
//     if keep_extention {
//         if let Some(src_ext) = src.extension() {
//             // 仅处理文件路径（通过文件名存在判断）
//             if let Some(file_name) = dst.file_name() {
//                 let dst_path = Path::new(file_name);

//                 let dst_str = dst.to_str().unwrap_or_default();

//                 // 不用std::path::MAIN_SEPARATOR判断是因为用户经常混用`\`与`/`
//                 #[cfg(windows)]
//                 let is_dir = dst.is_dir()
//                     || (!dst.to_str().unwrap_or_default().is_empty()
//                         && (dst_str.ends_with('/') || dst_str.ends_with('\\')));
//                 #[cfg(not(windows))]
//                 let is_dir = dst.is_dir()
//                     || (!dst.to_str().unwrap_or_default().is_empty()
//                         && dst_str.ends_with(std::path::MAIN_SEPARATOR));

//                 if !is_dir && dst_path.extension().is_none() && !src_ext.is_empty() {
//                     let new_name = format!(
//                         "{}.{}",
//                         dst_path.to_string_lossy(),
//                         src_ext.to_string_lossy()
//                     );
//                     dst.set_file_name(new_name);
//                     log::info!("get extension `.{}` from <SRC>", src_ext.to_string_lossy());
//                 }
//             }
//         }
//     }
// }

// /// 路径规范化
// fn canonicalize_path(path: &Path) -> PathBuf {
//     if path.is_absolute() {
//         path.to_path_buf().clean()
//     } else {
//         std::env::current_dir()
//             .expect("Failed to get current directory")
//             .join(path.clean())
//     }
// }

// fn mklink(src: PathBuf, dst: PathBuf) {
//     // log::info!(
//     //     "符号链接创建中: New-Item -Path '{}' -ItemType SymbolicLink -Target '{}'",
//     //     src.display(),
//     //     dst.display(),
//     // );

//     match create_symlink(&src, &dst) {
//         Ok(_) => log::info!("符号链接创建成功"),
//         Err(e) => log::error!("符号链接创建失败: {}", e),
//     }
// }

// /// 智能创建符号链接（自动判断文件/目录）
// pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
//     let src = src.as_ref();
//     let dst = dst.as_ref();

//     // 获取源文件元数据
//     let metadata = fs::metadata(src)?;

//     // 根据类型选择创建方式
//     if metadata.is_file() {
//         symlink_file(src, dst)
//     } else if metadata.is_dir() {
//         symlink_dir(src, dst)
//     } else {
//         Err(io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "奇怪的错误: <SRC>既不是文件也不是目录",
//         ))
//     }
// }

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
