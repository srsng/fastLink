use crate::types::args::Args;
use crate::MyError;
use fastlink_core::types::{
    link_task::LinkTask,
    link_task_args::{LinkTaskArgs, LinkTaskOpMode},
    link_task_pre::LinkTaskPre,
};

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

impl From<&Args> for LinkTaskPre {
    fn from(args: &Args) -> Self {
        log::debug!("已从Args构建LinkTaskPre");
        LinkTaskPre {
            args: LinkTaskArgs::from(args),
            src_path: None,
            dst_path: None,
        }
    }
}

impl From<&Args> for LinkTaskOpMode {
    fn from(args: &Args) -> Self {
        let check = args.check;
        let rm = args.rm;
        if check {
            if rm {
                log::warn!("rm模式请单独使用");
            }
            Self::Check
        } else if rm {
            Self::Remove
        } else {
            Self::Make
        }
    }
}

impl From<&Args> for LinkTaskArgs {
    fn from(args: &Args) -> Self {
        LinkTaskArgs {
            src: args.src.clone(),
            dst: args.dst.clone(),
            op_mode: LinkTaskOpMode::from(args),
            #[cfg(feature = "fastlink-regex")]
            re_pattern: args.regex.clone(),
            #[cfg(feature = "fastlink-regex")]
            re_max_depth: args
                .re_max_depth
                .unwrap_or(fastlink_core::types::link_task_args::DEFAULT_RE_MAX_DEPTH),
            #[cfg(feature = "fastlink-regex")]
            re_follow_links: args.re_follow_links,
            keep_extention: args.keep_extention,
            make_dir: args.make_dir,
            only_file: args.only_file,
            only_dir: args.only_dir,
            overwrite_links: args.overwrite_links,
            overwrite_broken_link: args.overwrite_broken_link,
            skip_exist_links: args.skip_exist_links,
            skip_broken_src_links: args.skip_broken_src_links,
            #[cfg(feature = "fastlink-regex")]
            re_no_check: args.re_no_check,
            #[cfg(feature = "fastlink-regex")]
            re_output_flatten: args.re_output_flatten,
            allow_broken_src: args.allow_broken_src,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::args::Args;
    use fastlink_core::types::link_task_args::{LinkTaskArgs, LinkTaskOpMode};

    #[test]
    fn test_link_task_op_mode_from_args() {
        let mut args = Args::default();
        assert_eq!(LinkTaskOpMode::from(&args), LinkTaskOpMode::Make);
        args.check = true;
        assert_eq!(LinkTaskOpMode::from(&args), LinkTaskOpMode::Check);
        args.check = false;
        args.rm = true;
        assert_eq!(LinkTaskOpMode::from(&args), LinkTaskOpMode::Remove);
    }

    #[test]
    fn test_link_task_args_from_args() {
        let args = Args {
            src: String::from("/tmp/source"),
            dst: Some(String::from("/tmp/dest")),
            keep_extention: true,
            make_dir: true,
            only_file: true,
            only_dir: false,
            overwrite_links: true,
            overwrite_broken_link: true,
            skip_exist_links: true,
            skip_broken_src_links: true,
            allow_broken_src: true,
            check: false,
            rm: false,
            quiet: false,
            debug: false,
            #[cfg(feature = "fastlink-regex")]
            regex: None,
            #[cfg(feature = "fastlink-regex")]
            re_max_depth: None,
            #[cfg(feature = "fastlink-regex")]
            re_follow_links: false,
            #[cfg(feature = "fastlink-regex")]
            re_no_check: false,
            #[cfg(feature = "fastlink-regex")]
            re_output_flatten: false,
            #[cfg(feature = "save-log")]
            save_log: None,
        };
        let link_args = LinkTaskArgs::from(&args);
        assert_eq!(link_args.src, args.src);
        assert_eq!(link_args.dst, args.dst);
        assert_eq!(link_args.keep_extention, args.keep_extention);
        assert_eq!(link_args.make_dir, args.make_dir);
        assert_eq!(link_args.only_file, args.only_file);
        assert_eq!(link_args.only_dir, args.only_dir);
        assert_eq!(link_args.overwrite_links, args.overwrite_links);
        assert_eq!(link_args.overwrite_broken_link, args.overwrite_broken_link);
        assert_eq!(link_args.skip_exist_links, args.skip_exist_links);
        assert_eq!(link_args.skip_broken_src_links, args.skip_broken_src_links);
        assert_eq!(link_args.allow_broken_src, args.allow_broken_src);
    }
}
