use crate::types::args::Args;
use std::path::Path;

/// 对一些特殊情况进行警告
pub fn special_warn(args: &Args) {
    // rm模式与check模式不做警告
    if args.rm || args.check {
        return;
    }

    let src = &args.src;
    let src_path = Path::new(src);
    let dst = &args.dst;
    // let keep_extention = args.keep_extention;

    // if keep_extention && dst.is_none() {
    //     log::warn!("不给定[DST]的同时使用-k，通常会因该目录下已有同名文件而创建失败！");
    // }

    // dst不为空的情况
    if let Some(dst) = dst {
        let dst_path = Path::new(dst);
        let dst_comps = dst_path.components().into_iter().collect::<Vec<_>>();

        if src_path.is_file() && dst_path.is_dir() {
            log::warn!("<SRC>为文件路径而[DST]为目录路径，将自动使用<SRC>文件名追加到[DST]")
        }

        if dst_path.is_relative() && dst_comps.len() == 1 {
            log::warn!("这样做会在当前目录创建，对该目录本身的符号链接，如果你不清楚自己这样做的后果，请不要这么做!（这个时候可能已经创建成功了，那么就快点删除它！）")
        }

        // if src == ".." && dst.is_relative() {
        //     log::warn!("这样做会在当前目录创建，对该目录的父目录的符号链接，如果你不清楚自己这样做的后果，请不要这么做!（这个时候可能已经创建成功了，那么就快点删除它！）")
        // }
    }
}
