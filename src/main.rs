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

use crate::types::link_task::LinkTask;

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
