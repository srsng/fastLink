// use log::info;
// use clap::builder::Str;
use clap::Parser;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
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

#[derive(Parser)]
#[command(
    version,
    about = "A tool to make symlink fastly and smartly",
    long_about = r#"
Example：
  fastlink document.txt
  fastlink image.jpg backup -k
  fastlink data.csv /tmp/output --keep-name
"#
)]
struct Args {
    /// Source file or directory
    #[arg(required = true)]
    src: String,

    /// Destination path (optional)
    dst: Option<String>,

    /// 是否保留src的文件拓展名
    #[arg(short, long)]
    keep_extention: bool,

    #[arg(short, long)]
    quiet: bool,
}

fn init_log(quiet: bool) {
    // 初始化日志系统
    env_logger::Builder::new()
        .filter_level(if quiet {
            log::LevelFilter::Off
        } else {
            log::LevelFilter::Info
        })
        .init();
}

fn main() {
    let args = Args::parse();

    init_log(args.quiet);
    let keep_extention = args.keep_extention;

    let src_path = PathBuf::from(&args.src);
    let dst_path = parse_args_dst(&args.src, args.dst.as_deref(), keep_extention);

    let src_abs = canonicalize_path(&src_path);
    let dst_abs = canonicalize_path(&dst_path);

    mklink(src_abs, dst_abs);
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
            log::warn!("未知错误：src文件名为空，已设置dst名称为unnamed-fastlink");
            OsStr::new("unnamed-fastlink")
        })
    });

    // 输出日志信息
    log::info!(
        "已由src确定目标名 {} → {}",
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
                    log::info!("get extension {} from src", src_ext.to_string_lossy());
                }
            }
        }
    }
}

/// 路径规范化
fn canonicalize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .expect("Failed to get current directory")
            .join(path)
    }
}

fn mklink(src: PathBuf, dst: PathBuf) {
    match create_symlink(&src, &dst) {
        Ok(_) => log::info!(
            "符号链接创建成功\n
src: {}
dst: {}\n
New-Item -Path '{}' -ItemType SymbolicLink -Target '{}'",
            src.display(),
            dst.display(),
            dst.display(),
            src.display()
        ),
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
