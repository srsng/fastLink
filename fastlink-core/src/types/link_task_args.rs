#[cfg(feature = "fastlink-regex")]
pub const DEFAULT_RE_MAX_DEPTH: usize = 4;

#[derive(Debug, Default, Clone)]
pub struct LinkTaskArgs {
    pub src: String,         // 原始源路径
    pub dst: Option<String>, // 原始目标路径
    pub op_mode: LinkTaskOpMode,
    #[cfg(feature = "fastlink-regex")]
    pub re_pattern: Option<regex::Regex>, // 正则表达式模式
    #[cfg(feature = "fastlink-regex")]
    pub re_max_depth: usize, // 正则表达式模式最大深度
    #[cfg(feature = "fastlink-regex")]
    pub re_follow_links: bool, // re匹配过程中深入读取符号链接进行匹配
    pub keep_extention: bool,        // 是否自动保留<SRC>的文件拓展名到[DST]
    pub make_dir: bool,              // 是否自动创建不存在的目录
    pub only_file: bool,             // 只处理文件
    pub only_dir: bool,              // 只处理目录
    pub overwrite_links: bool,       // 覆盖同名已存在的符号链接
    pub overwrite_broken_link: bool, // 覆盖同名已存在的损坏的符号链接
    pub skip_exist_links: bool,      // 跳过同名已存在的符号链接
    pub skip_broken_src_links: bool, // 跳过src中损坏的符号链接
    #[cfg(feature = "fastlink-regex")]
    pub re_no_check: bool, // 跳过用户Re检查
    #[cfg(feature = "fastlink-regex")]
    pub re_output_flatten: bool, // 展平输出路径
    pub allow_broken_src: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LinkTaskOpMode {
    #[default]
    Make,
    Check,
    Remove,
}

// Builder 结构体
#[derive(Default)]
pub struct LinkTaskArgsBuilder {
    src: String,
    dst: Option<String>,
    op_mode: Option<LinkTaskOpMode>,
    #[cfg(feature = "fastlink-regex")]
    re_pattern: Option<regex::Regex>,
    #[cfg(feature = "fastlink-regex")]
    re_max_depth: Option<usize>,
    #[cfg(feature = "fastlink-regex")]
    re_follow_links: Option<bool>,
    keep_extention: Option<bool>,
    make_dir: Option<bool>,
    only_file: Option<bool>,
    only_dir: Option<bool>,
    overwrite_links: Option<bool>,
    overwrite_broken_link: Option<bool>,
    skip_exist_links: Option<bool>,
    skip_broken_src_links: Option<bool>,
    #[cfg(feature = "fastlink-regex")]
    re_no_check: Option<bool>,
    #[cfg(feature = "fastlink-regex")]
    re_output_flatten: Option<bool>,
    allow_broken_src: Option<bool>,
}

/// 链式设置参数以创建LinkTaskArgs
/// 参数方面默认值与fastlink参数一致
impl LinkTaskArgsBuilder {
    // 初始化 Builder
    pub fn new(src: String) -> Self {
        LinkTaskArgsBuilder {
            src,
            ..Default::default()
        }
    }

    // 构建最终的 LinkTaskArgs
    pub fn build(self) -> LinkTaskArgs {
        LinkTaskArgs {
            src: self.src,
            dst: self.dst,
            op_mode: self.op_mode.unwrap_or_default(),
            #[cfg(feature = "fastlink-regex")]
            re_pattern: self.re_pattern,
            #[cfg(feature = "fastlink-regex")]
            re_max_depth: self.re_max_depth.unwrap_or(DEFAULT_RE_MAX_DEPTH),
            #[cfg(feature = "fastlink-regex")]
            re_follow_links: self.re_follow_links.unwrap_or(false),
            keep_extention: self.keep_extention.unwrap_or(false),
            make_dir: self.make_dir.unwrap_or(false),
            only_file: self.only_file.unwrap_or(false),
            only_dir: self.only_dir.unwrap_or(false),
            overwrite_links: self.overwrite_links.unwrap_or(false),
            overwrite_broken_link: self.overwrite_broken_link.unwrap_or(false),
            skip_exist_links: self.skip_exist_links.unwrap_or(false),
            skip_broken_src_links: self.skip_broken_src_links.unwrap_or(false),
            #[cfg(feature = "fastlink-regex")]
            re_no_check: self.re_no_check.unwrap_or(false),
            #[cfg(feature = "fastlink-regex")]
            re_output_flatten: self.re_output_flatten.unwrap_or(false),
            allow_broken_src: self.allow_broken_src.unwrap_or(false),
        }
    }

    pub fn dst(mut self, value: impl Into<String>) -> Self {
        self.dst = Some(value.into());
        self
    }

    #[cfg(feature = "fastlink-regex")]
    pub fn re_pattern(mut self, value: regex::Regex) -> Self {
        self.re_pattern = Some(value);
        self
    }

    #[cfg(feature = "fastlink-regex")]
    pub fn re_max_depth(mut self, value: usize) -> Self {
        self.re_max_depth = Some(value);
        self
    }

    pub fn keep_extention(mut self, value: bool) -> Self {
        self.keep_extention = Some(value);
        self
    }

    pub fn make_dir(mut self, value: bool) -> Self {
        self.make_dir = Some(value);
        self
    }

    pub fn only_file(mut self, value: bool) -> Self {
        self.only_file = Some(value);
        self
    }

    pub fn only_dir(mut self, value: bool) -> Self {
        self.only_dir = Some(value);
        self
    }

    pub fn overwrite_links(mut self, value: bool) -> Self {
        self.overwrite_links = Some(value);
        self
    }

    pub fn overwrite_broken_link(mut self, value: bool) -> Self {
        self.overwrite_broken_link = Some(value);
        self
    }

    pub fn skip_exist_links(mut self, value: bool) -> Self {
        self.skip_exist_links = Some(value);
        self
    }

    pub fn skip_broken_src_links(mut self, value: bool) -> Self {
        self.skip_broken_src_links = Some(value);
        self
    }

    #[cfg(feature = "fastlink-regex")]
    pub fn re_no_check(mut self, value: bool) -> Self {
        self.re_no_check = Some(value);
        self
    }

    #[cfg(feature = "fastlink-regex")]
    pub fn re_output_flatten(mut self, value: bool) -> Self {
        self.re_output_flatten = Some(value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_link_task_args_builder_defaults() {
        let src = String::from("/tmp/source");
        let args = LinkTaskArgsBuilder::new(src.clone()).build();
        assert_eq!(args.src, src);
        assert_eq!(args.dst, None);
        assert_eq!(args.op_mode, LinkTaskOpMode::Make);
        assert!(!args.keep_extention);
        assert!(!args.make_dir);
        assert!(!args.only_file);
        assert!(!args.only_dir);
        assert!(!args.overwrite_links);
        assert!(!args.overwrite_broken_link);
        assert!(!args.skip_exist_links);
        assert!(!args.skip_broken_src_links);
        assert!(!args.allow_broken_src);
    }

    #[test]
    fn test_link_task_args_builder_setters() {
        let src = String::from("/tmp/source");
        let dst = String::from("/tmp/dest");
        let args = LinkTaskArgsBuilder::new(src.clone())
            .dst(dst.clone())
            .keep_extention(true)
            .make_dir(true)
            .only_file(true)
            .only_dir(false)
            .overwrite_links(true)
            .overwrite_broken_link(true)
            .skip_exist_links(true)
            .skip_broken_src_links(true)
            .build();
        assert_eq!(args.src, src);
        assert_eq!(args.dst, Some(dst));
        assert!(args.keep_extention);
        assert!(args.make_dir);
        assert!(args.only_file);
        assert!(!args.only_dir);
        assert!(args.overwrite_links);
        assert!(args.overwrite_broken_link);
        assert!(args.skip_exist_links);
        assert!(args.skip_broken_src_links);
    }

    #[test]
    fn test_tempfile_usage_for_src() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, b"test").unwrap();
        let src = file_path.to_str().unwrap().to_string();
        let args = LinkTaskArgsBuilder::new(src.clone()).build();
        assert_eq!(args.src, src);
    }
}
