use clap::builder::styling::Color;
use clap::builder::styling::RgbColor;
use clap::builder::styling::Style;
use clap::Parser;
use path_clean::PathClean;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Write;
use std::os::windows::fs::{symlink_dir, symlink_file};
use std::path::{Path, PathBuf};

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

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A tool to make symlink fastly and smartly",
    long_about = r#"
Example：
  fastlink document.txt
  fastlink image.jpg img-link -k
  fastlink data.csv /tmp/output --keep-extention
"#
)]
struct Args {
    /// 源文件/源目录路径
    #[arg(required = true, value_parser = validate_src)]
    src: String,

    /// 目标路径，可选，区分文件拓展名，为空则以自动<SRC>路径名称填充
    dst: Option<String>,

    /// 自动保留src的文件拓展名到dst。保留拓展名之后可以通过对符号链接双击、运行等操作让系统使用默认应用打开或执行。
    #[arg(short, long)]
    keep_extention: bool,

    /// 只输出warn与error level的日志
    #[arg(short, long)]
    quiet: bool,

    /// 输出debug level的日志
    #[arg(long)]
    debug: bool,
}

fn validate_src(s: &str) -> Result<String, String> {
    let path = Path::new(s).clean();

    if s.trim().is_empty() {
        Err("路径不能为空或纯空格".into())
    } else if path.components().count() == 0 {
        Err("无效的路径格式".into())
    } else {
        Ok(s.into())
    }
}

fn init_log(quiet: bool, debug: bool) {
    // 初始化日志系统
    let mut builder = env_logger::Builder::new();
    builder
        .format(|buf, record| {
            let time = chrono::Local::now().format("%H:%M:%S");
            let level = record.level();
            let level_style = buf.default_level_style(level);

            // 设置时间颜色（灰色）
            let time_style = Style::new().fg_color(Some(Color::Rgb(RgbColor(150, 150, 150))));

            // 格式化输出
            writeln!(
                buf,
                "{time_style}{time}{time_style:#} {level_style}{level}{level_style:#} {}",
                record.args()
            )
        })
        .filter_level(if quiet {
            log::LevelFilter::Off
        } else if debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();
    log::debug!("log init.")
}

fn main() {
    let args = Args::parse();

    init_log(args.quiet, args.debug);
    log::debug!("{:?}", args);
    special_warn(&args);
    let keep_extention = args.keep_extention;

    // 规范化src
    let src_abs_res = dunce::canonicalize(&args.src);
    if src_abs_res.is_err() {
        log::error!(
            "请检查<SRC>'{}'是否存在. Fail to canonicalize <SRC>",
            &args.src
        );
        return;
    }
    let src_abs = src_abs_res.unwrap();

    // 解析dst
    let dst_path = parse_args_dst(&args.src, args.dst.as_deref(), keep_extention).clean();

    // 验证dst路径父目录
    match validate_dst(&dst_path) {
        Err(e) => {
            log::error!(
                "[DST]父目录不存在，且创建失败，请尝试手动创建或修改路径: \n{}",
                e
            );
        }
        Ok(_) => {
            // 规范化dst，失败则跳过
            let dst_abs_res = dunce::canonicalize(&dst_path);
            let dst_abs = match dst_abs_res {
                Ok(path) => path,
                Err(_e) => dst_path,
            };
            mklink(src_abs, dst_abs);
        }
    };
}

/// 对一些特殊情况进行警告
fn special_warn(args: &Args) {
    let src = &args.src;
    let dst = &args.dst;
    let keep_extention = args.keep_extention;

    if keep_extention && dst.is_none() {
        log::warn!("不给定[DST]的同时使用-k，通常会因该目录下已有同名文件而创建失败！");
    }

    if src == "." && dst.is_none() {
        log::warn!("这样做会在当前目录创建，对该目录本身的符号链接，如果你不清楚自己这样做的后果，请不要这么做!（这个时候可能已经创建成功了，那么就快点删除它！）")
    }

    // if src == ".." && !dst.is_none() {
    //     let dst = Path::new(dst.unwrap());
    //     if dst.is_relative() {
    //         log::warn!("这样做会在当前目录创建，对该目录的父目录的符号链接，如果你不清楚自己这样做的后果，请不要这么做!（这个时候可能已经创建成功了，那么就快点删除它！）")
    //     }
    // }
}

/// 判断dst所在目录是否存在，若不存在，则为其创建，并警告
fn validate_dst(dst: &Path) -> Result<(), String> {
    log::debug!("validate_dst/dst: {}", dst.display());

    let dst = canonicalize_path(dst);
    let dst_parent_option = dst.parent().take();
    // dst父目录不存在
    if dst_parent_option.is_some() && !dst_parent_option.unwrap().exists() {
        let dst_parent = dst_parent_option.unwrap().clean();
        // 创建目录并处理错误
        match mkdirs(&dst_parent) {
            Ok(_) => {
                log::warn!("[DST]父目录不存在，已创建: {}", dst_parent.display());
                Ok(())
            }
            Err(e) => Err(format!(
                "[DST]父目录: {} \nErrorMsg: {}",
                dst_parent.display(),
                e
            )),
        }

        // 规范化dst_parent
        // let dst_parent_res = dunce::canonicalize(&dst_parent);
        // // unused variable
        // let _dst_parent = match dst_parent_res {
        //     Ok(_dst_parent) => _dst_parent,
        //     // 失败则只警告, 保持dst_parent不变
        //     Err(e) => {
        //         log::warn!(
        //             "Fail to canonicalize [DST] parent dir: {}\n{}",
        //             dst_parent.display(),
        //             e
        //         );
        //         dst_parent
        //     }
        // }
        // .clean();
        // log::debug!("validate_dst/dst_parent: {}", dst_parent.display());
    } else {
        Ok(())
    }
}

/// 创建目录
fn mkdirs(path: &Path) -> Result<(), String> {
    let res = std::fs::create_dir_all(path);
    match res {
        Err(e) => Err(format!("{}", e)),
        _ => Ok(()),
    }
}

fn parse_args_dst(src: &str, dst: Option<&str>, keep_extention: bool) -> PathBuf {
    let src_path = Path::new(src);
    let mut final_dst = match dst {
        Some(d) => PathBuf::from(d),
        None => default_dst_path(src_path),
    };

    // 处理keep_extension: 是否保留拓展名
    process_extension(src_path, &mut final_dst, keep_extention);

    final_dst
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

/// 扩展名处理逻辑（统一处理相对/绝对路径）
fn process_extension(src: &Path, dst: &mut PathBuf, keep_extention: bool) {
    if keep_extention {
        if let Some(src_ext) = src.extension() {
            // 仅处理文件路径（通过文件名存在判断）
            if let Some(file_name) = dst.file_name() {
                let dst_path = Path::new(file_name);

                // 忽略目录路径（通过原始路径字符串判断）
                let is_dir = dst.to_string_lossy().ends_with(std::path::MAIN_SEPARATOR);

                if !is_dir && dst_path.extension().is_none() {
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

fn mklink(src: PathBuf, dst: PathBuf) {
    log::info!(
        "符号链接创建中: New-Item -Path '{}' -ItemType SymbolicLink -Target '{}'",
        src.display(),
        dst.display(),
    );

    match create_symlink(&src, &dst) {
        Ok(_) => log::info!("符号链接创建成功"),
        Err(e) => log::error!("符号链接创建失败: {}", e),
    }
}

/// 智能创建符号链接（自动判断文件/目录）
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    // 获取源文件元数据
    let metadata = fs::metadata(src)?;

    // 根据类型选择创建方式
    if metadata.is_file() {
        symlink_file(src, dst)
    } else if metadata.is_dir()
    // && dst
    //     .map(|d| d.ends_with(std::path::MAIN_SEPARATOR))
    //     .unwrap_or(false)
    {
        symlink_dir(src, dst)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "源路径既不是文件也不是目录",
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_dst_path() {
        let dir_abs = Path::new(r"C:\Windows\System32");
        let dir_rel = Path::new(r"System32");
        let dir_rel2 = Path::new(r"\Windows\System32");
        let dir_rel3 = Path::new(r"..\System32");

        assert_eq!(PathBuf::from("System32"), default_dst_path(dir_abs));
        assert_eq!(PathBuf::from("System32"), default_dst_path(dir_rel));
        assert_eq!(PathBuf::from("System32"), default_dst_path(dir_rel2));
        assert_eq!(PathBuf::from("System32"), default_dst_path(dir_rel3));

        let file_abs = Path::new(r"C:\Windows\System32\notepad.exe");
        let file_rel = Path::new(r"notepad.exe");
        let file_rel2 = Path::new(r"System32\notepad.exe");
        let file_rel3 = Path::new(r"..\notepad.exe");
        assert_eq!(PathBuf::from("notepad"), default_dst_path(file_abs));
        assert_eq!(PathBuf::from("notepad"), default_dst_path(file_rel));
        assert_eq!(PathBuf::from("notepad"), default_dst_path(file_rel2));
        assert_eq!(PathBuf::from("notepad"), default_dst_path(file_rel3));

        assert_eq!(
            PathBuf::from("unnamed-fastlink"),
            default_dst_path(Path::new(""))
        );
    }

    #[test]
    fn test_parse_args_dst() {
        let dir_abs = r"C:\Windows\System32";
        let dir_rel = r"System32";

        let file_abs = r"C:\Windows\System32\notepad.exe";
        let file_rel = r"notepad.exe";
        // block: dst is None
        {
            let dir_tar = PathBuf::from("System32");
            let file_tar_k_t = PathBuf::from("notepad.exe");
            let file_tar_k_f = PathBuf::from("notepad");

            // keep_extention true
            assert_eq!(dir_tar, parse_args_dst(dir_abs, None, true));
            assert_eq!(dir_tar, parse_args_dst(dir_rel, None, true));
            assert_eq!(file_tar_k_t, parse_args_dst(file_abs, None, true));
            assert_eq!(file_tar_k_t, parse_args_dst(file_rel, None, true));
            // no keep_extention false
            assert_eq!(dir_tar, parse_args_dst(dir_abs, None, false));
            assert_eq!(dir_tar, parse_args_dst(dir_rel, None, false));
            assert_eq!(file_tar_k_f, parse_args_dst(file_abs, None, false));
            assert_eq!(file_tar_k_f, parse_args_dst(file_rel, None, false));
        }

        // block: dst not None, relative path
        {
            let some_dst = Some(r"..\some_name");

            let dir_tar = PathBuf::from(r"..\some_name");
            let file_tar_k_t = PathBuf::from(r"..\some_name.exe");
            let file_tar_k_f = PathBuf::from(r"..\some_name");

            // keep_extention true
            assert_eq!(dir_tar, parse_args_dst(dir_abs, some_dst, true));
            assert_eq!(dir_tar, parse_args_dst(dir_rel, some_dst, true));
            assert_eq!(file_tar_k_t, parse_args_dst(file_abs, some_dst, true));
            assert_eq!(file_tar_k_t, parse_args_dst(file_rel, some_dst, true));
            // no keep_extention false
            assert_eq!(dir_tar, parse_args_dst(dir_abs, some_dst, false));
            assert_eq!(dir_tar, parse_args_dst(dir_rel, some_dst, false));
            assert_eq!(file_tar_k_f, parse_args_dst(file_abs, some_dst, false));
            assert_eq!(file_tar_k_f, parse_args_dst(file_rel, some_dst, false));
        }
        // block: dst not None, absolute path
        {
            let some_dst = Some(r"C:\some_name");

            let dir_tar = PathBuf::from(r"C:\some_name");
            let file_tar_k_t = PathBuf::from(r"C:\some_name.exe");
            let file_tar_k_f = PathBuf::from(r"C:\some_name");

            // keep_extention true
            assert_eq!(dir_tar, parse_args_dst(dir_abs, some_dst, true));
            assert_eq!(dir_tar, parse_args_dst(dir_rel, some_dst, true));
            assert_eq!(file_tar_k_t, parse_args_dst(file_abs, some_dst, true));
            assert_eq!(file_tar_k_t, parse_args_dst(file_rel, some_dst, true));
            // no keep_extention false
            assert_eq!(dir_tar, parse_args_dst(dir_abs, some_dst, false));
            assert_eq!(dir_tar, parse_args_dst(dir_rel, some_dst, false));
            assert_eq!(file_tar_k_f, parse_args_dst(file_abs, some_dst, false));
            assert_eq!(file_tar_k_f, parse_args_dst(file_rel, some_dst, false));
        }
    }
}
