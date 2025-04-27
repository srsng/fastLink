use std::env;
use std::fs;
use std::io;
use std::os::windows::fs::{symlink_dir, symlink_file};

use std::path::{Path, PathBuf};

// // windows隐藏命令行窗口参数
// const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 参数检查
    if args.len() != 3 {
        #[cfg(debug_assertions)]
        for (i, arg) in args.iter().enumerate() {
            println!("arg {}: {}", i, arg);
        }

        eprintln!("Usage: {} <src> <dst>", args[0]);
        std::process::exit(1);
    }

    let src = Path::new(&args[1]);
    let dst = Path::new(&args[2]);

    // 将路径转换为绝对路径
    let src = canonicalize_path(src);
    let dst = canonicalize_path(dst);

    // 直接使用原生方式
    match create_symlink(&src, &dst) {
        Ok(_) => println!(
            "[Info] 符号链接创建成功\n源: {}\n目标: {}",
            src.display(),
            dst.display()
        ),
        Err(e) => eprintln!("[Error] 创建失败: {}", e),
    }
    // 调试用命令提示
    println!(
        "\nNew-Item -Path '{}' -ItemType SymbolicLink -Target '{}'",
        dst.display(),
        src.display()
    );
}

/// 路径规范化
fn canonicalize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().unwrap().join(path)
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
    } else if metadata.is_dir() {
        symlink_dir(src, dst)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "源路径既不是文件也不是目录",
        ))
    }
}
