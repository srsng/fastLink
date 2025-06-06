// todo: 编写tests
// todo：编写文档
// todo: 清理无用代码
// 其他行内todo

use clap::Parser;
use std::path::Path;
pub mod types;
pub mod utils;
use types::args::Args;
use utils::logs::init_log;

use crate::{
    types::{
        err::ErrorCode,
        link_task::{del_exists_link, LinkTask},
    },
    utils::func::mklink_pre_check,
};

lazy_static::lazy_static! {
    pub static ref WORK_DIR: std::path::PathBuf = std::env::current_dir().expect("Failed to get initial work directory");
}

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

    if args.check || args.rm {
        handle_sub_command(args);
    } else {
        special_warn(&args);

        let task_res = LinkTask::try_from(&args);

        match task_res {
            Ok(mut task) => match task.mklinks() {
                Ok(()) => (),
                Err(e) => e.log(),
            },
            Err(e) => e.log(),
        }
    }
}

fn handle_sub_command(args: Args) {
    let check = args.check;
    let rm = args.rm;
    let src_path = Path::new(&args.src);
    if check {
        handle_check_command(src_path);
        if rm {
            log::warn!("rm模式请单独使用");
        }
    } else if rm {
        handle_rm_command(src_path);
    }
}

fn handle_check_command(src: &Path) {
    log::info!("[check模式(--check)]");
    match mklink_pre_check(src) {
        Err(e) if e.code == ErrorCode::TargetExistsAndNotLink => {
            let filetype = if src.is_dir() {
                "DIR"
            } else if src.is_file() {
                "FILE"
            } else {
                "UNKOWN TYPE"
            };
            log::info!("{} 是 {}", src.display(), filetype);
        }
        Err(e) if e.code == ErrorCode::BrokenSymlink => {
            log::warn!("{} 是损坏的符号链接(Broken Symlink)", src.display())
        }
        Err(e) if e.code == ErrorCode::FileNotExist => {
            log::warn!("{} 不存在", src.display())
        }
        Err(e) if e.code == ErrorCode::TargetLinkExists => {
            let target = std::fs::read_link(src);
            match target {
                Ok(dst) => log::info!("{} 是符号链接，指向 {}", src.display(), dst.display()),
                Err(e) => log::error!("{} 是符号链接，指向未知，获取时出错：{}", src.display(), e),
            };
        }
        _ => log::warn!("未知错误"),
    }
}

fn handle_rm_command(src: &Path) {
    log::info!("[rm模式(--rm)]");
    let del_link = true;
    match del_exists_link(src, del_link) {
        Ok(_) => (),
        Err(e) => e.log(),
    };
}

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
