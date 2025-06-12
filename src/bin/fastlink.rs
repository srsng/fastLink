// todo: 编写tests
// todo：编写文档
// todo: 清理无用代码
// 添加子命令wallpaper/w，并且增加桌面绑定壁纸功能（不兼容wallpaper engine等软件，因此需要手动开启）
// 增加修改桌面后自动刷新（F5效果）桌面
// 其他行内todo

use clap::Parser;
use std::path::Path;

use fastlink::types::args::Args;
use fastlink::utils::logs::init_log;

use fastlink::types::link_task::LinkTask;

fn main() {
    let args: Args = Args::parse();

    // 初始化日志系统

    #[cfg(feature = "save_log")]
    init_log(args.quiet, args.debug, &args.save_log);
    #[cfg(not(feature = "save_log"))]
    init_log(args.quiet, args.debug);
    log::debug!("{:?}", args);

    special_warn(&args);

    let task_res = LinkTask::try_from(&args);

    match task_res {
        Ok(task) => match task.work() {
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
