use std::env;
use std::io;
use std::os::windows::fs::symlink_file;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

// windows隐藏命令行窗口参数
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        #[cfg(debug_assertions)]
        for i in args.iter() {
            println!("arg {}", i);
        }

        eprintln!("Usage: {} <src> <dst>", args[0]);
        std::process::exit(1);
    }

    let src = Path::new(&args[1]);
    let dst = Path::new(&args[2]);
    let src = src.to_str().unwrap();
    let dst = dst.to_str().unwrap();

    // println!("{:?}", try_create_symlink2(src, dst));
    println!("{}", try_create_symlink(src, dst).unwrap());
    println!(
        "New-Item {:?} -ItemType SymbolicLink -Target {:?}",
        dst, src
    );
}

pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    // #[cfg(debug_assertions)]
    println!(
        "[Info]尝试创建符号链接中:\n\tsrc: {:?},\n\tdst: {:?}",
        src.as_ref(),
        dst.as_ref()
    );
    symlink_file(src, dst)
}

pub fn try_create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Option<String> {
    match create_symlink(src, dst.as_ref()) {
        Ok(_) => {
            // Some(dst.as_ref().to_string_lossy().into_owned())
            Some("[Info]创建成功".to_string())
        }
        Err(e) => Some(format!("[Error]创建失败: {:?}", e)),
    }
}

pub fn try_create_symlink2(src: &str, dst: &str) -> Result<std::process::Output, io::Error> {
    let output = Command::new("powershell")
        .args(["New-Item", dst, "-ItemType", "SymbolicLink", "-Target", src])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    output
}
