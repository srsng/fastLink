use crate::types::args::Args;

#[derive(Debug, Default, Clone)]
pub struct LinkTaskArgs {
    pub src: String,         // 原始源路径
    pub dst: Option<String>, // 原始目标路径
    #[cfg(feature = "regex")]
    pub re_pattern: Option<regex::Regex>, // 正则表达式模式
    #[cfg(feature = "regex")]
    pub re_max_depth: usize, // 正则表达式模式最大深度
    #[cfg(feature = "regex")]
    pub re_follow_links: bool, // re匹配过程中深入读取符号链接进行匹配
    pub keep_extention: bool, // 是否自动保留<SRC>的文件拓展名到[DST]
    pub make_dir: bool,      // 是否自动创建不存在的目录
    pub only_file: bool,     // 只处理文件
    pub only_dir: bool,      // 只处理目录
    pub overwrite_links: bool, // 覆盖同名已存在的符号链接
    pub overwrite_broken_link: bool, // 覆盖同名已存在的损坏的符号链接
    pub skip_exist_links: bool, // 跳过同名已存在的符号链接
    pub skip_broken_src_links: bool, // 跳过src中损坏的符号链接
    #[cfg(feature = "regex")]
    pub re_no_check: bool, // 跳过用户Re检查
    #[cfg(feature = "regex")]
    pub re_output_flatten: bool, // 展平输出路径
}

impl From<&Args> for LinkTaskArgs {
    fn from(args: &Args) -> Self {
        LinkTaskArgs {
            src: args.src.clone(),
            dst: args.dst.clone(),
            #[cfg(feature = "regex")]
            re_pattern: args.regex.clone(),
            #[cfg(feature = "regex")]
            re_max_depth: args
                .re_max_depth
                .unwrap_or(crate::types::args::DEFAULT_RE_MAX_DEPTH),
            #[cfg(feature = "regex")]
            re_follow_links: args.re_follow_links,
            keep_extention: args.keep_extention,
            make_dir: args.make_dir,
            only_file: args.only_file,
            only_dir: args.only_dir,
            overwrite_links: args.overwrite_links,
            overwrite_broken_link: args.overwrite_broken_link,
            skip_exist_links: args.skip_exist_links,
            skip_broken_src_links: args.skip_broken_src_links,
            #[cfg(feature = "regex")]
            re_no_check: args.re_no_check,
            #[cfg(feature = "regex")]
            re_output_flatten: args.re_output_flatten,
        }
    }
}

impl LinkTaskArgs {
    pub fn new() -> Self {
        // todo
        unimplemented!("暂不支持外部自定义link task args")
    }
}
