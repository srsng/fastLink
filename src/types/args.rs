use crate::types::err::{ErrorCode, MyError};
use clap::Parser;
use path_clean::PathClean;

#[cfg(feature = "fatlink_regex")]
pub const DEFAULT_RE_MAX_DEPTH: usize = 4;

use crate::types::args_example_text::EXAMPLE;

// const EXAMPLE_BASE: &str = r#"
// Example：
//     // 在当前目录创建一个名为document的符号链接
//     fastlink document.txt

//     // 在当前目录创建一个名为img-link.jpg的符号链接
//     fastlink image.jpg img-link -k

//     // 在当前目录的子目录tmp中创建名为output.csv的符号链接，若tmp目录不存在将退出
//     fastlink data.csv tmp/output --keep-extention

//     // 同上，但添加--make-dir或--md参数选项将自动创建目录
//     fastlink data.csv tmp/output --keep-extention --md

//     // 在当前目录的父目录创建名为data符号链接，指向data.csv (不建议, Not Recommended)
//     fastlink data.csv ../
// "#;

// #[cfg(feature = "fatlink_regex")]
// const EXAMPLE_OF_RE: &str = r#"
//     // 为./test-dir目录中所有满足.*\.txt正则表达式的路径（即所有txt文件） 创建链接到output目录中
//     fastlink ./test-dir output --re .*\.txt

//     // ./test-dir目录及其子目录或更深目录中所有txt文件 -> 镜像目录创建链接到output目录中
//     fastlink ./test-dir output --re .*\.txt --only-file

//     // 将./test-dir目录及其子目录或更深目录中所有txt文件 -> 直接创建链接到output目录中，不包含文件夹（可包含对文件夹的符号链接）
//     fastlink ./test-dir output --re .*\.txt --flatten
// "#;

// #[cfg(feature = "save_log")]
// const EXAMPLE_OF_SAVE_LOG: &str = r#"
//     // 保存日志到指定文件
//     fastlink document.txt --save-log my_log.log

//     // 保存日志到默认路径（fastlink-YY-MM-DD-HH-MM-SS.log）
//     fastlink data.csv tmp/output --md --save-log
// "#;

// const EXAMPLE: &str = "" ;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A tool to make symlink fastly and smartly\n一个智能且方便的符号链接创建工具",
    long_about = EXAMPLE
)]
pub struct Args {
    /// 源文件/源目录路径，表示的是符号链接指向的路径(Point at who)。
    #[arg(required = true, value_parser = validate_src)]
    pub src: String,

    /// 目标路径，可选，区分文件拓展名，表示的是要创建在什么位置(Where to create)。
    /// 为空则自动以<SRC>路径名称填充；当<SRC>为文件，[DST]为目录时，自动以<SRC>路径名称填充
    pub dst: Option<String>,

    /// 高优先级参数, 切换为检查模式，添加后不会创建链接: 检查<SRC>属性，包含文件/目录/符号链接，以及符号链接损坏与否
    ///
    /// 如果只给出<SRC>，则检查SRC，若同时传入DST，则检查DST.支持Re.
    #[arg(short, long)]
    pub check: bool,

    /// 高优先级参数, 切换为删除模式，添加后不会创建链接: 若<SRC>是符号链接，则删除。
    ///
    /// 如果只给出<SRC>，则删除SRC，若同时传入DST，则检查DST.支持Re.
    #[arg(long)]
    pub rm: bool,

    /// *追加*<SRC>的文件拓展名到[DST]，不会去除/替换
    /// 保留拓展名之后可以通过对符号链接双击、运行等操作让系统使用默认应用打开或执行。
    ///
    /// src:".jpg", dst: ".jpg" -> dst: ".jpg"; src:".jpg", dst: ".temp" -> dst: ".jpg.temp"
    #[arg(short, long)]
    pub keep_extention: bool,

    /// 自动创建不存在的目录
    #[arg(long, visible_alias("md"))]
    pub make_dir: bool,

    /// 只输出warn与error level的日志
    #[arg(short, long)]
    pub quiet: bool,

    /// 输出debug level的日志
    #[arg(long)]
    pub debug: bool,

    #[cfg(feature = "fatlink_regex")]
    /// 对<SRC>内容应用正则表达式，匹配项将于[DST]相应创建。对于程序如何处理多层级的目录见only_dir参数
    ///
    /// 注：若启用make_dir参数，则还会尝试对<SRC>的子目录以及更深层(默认最大4层)进行匹配并创建，
    /// 若要限制深度，使用--re-max-depth参数。
    ///
    /// 注：匹配的路径不受--keep_extention参数影响。
    ///
    /// 注：只会为最深层的目录创建符号链接，其他层次目录则会正常创建文件夹
    #[arg(long, visible_alias("re"), value_parser = validate_regex)]
    pub regex: Option<regex::Regex>,

    #[cfg(feature = "fatlink_regex")]
    /// 限制regex匹配的最大深度，启用make_dir参数时，默认4层，否则为1层,
    /// 传入0表示没有层数限制.
    /// 该参数数值非负.
    #[arg(long, visible_alias("re-depth"), value_parser = validate_re_max_depth)]
    pub re_max_depth: Option<usize>,

    /// 只为文件创建符号链接，但仍然会创建目录
    #[arg(long, conflicts_with = "only_dir", visible_alias("F"))]
    pub only_file: bool,

    /// 只为最深目录创建符号链接，其他目录则会创建文件夹，受re-depth参数约束
    ///
    /// 程序将如何为目录创建符号链接？
    /// e.g.1 给定src的子目录最深为5层，re-depth参数默认为4层，会为层级为4的目录创建符号链接，
    /// 1、2、3层只会创建文件夹
    ///
    /// e.g.2 给定src的子目录最深为3层，re-depth参数默认为4层，会为层级为3的目录创建符号链接，
    /// 1、2层只会创建文件夹
    #[arg(long, conflicts_with = "only_file", visible_alias("D"))]
    pub only_dir: bool,

    #[cfg(feature = "fatlink_regex")]
    /// re匹配过程中，深入读取符号链接进行匹配
    #[arg(long, visible_alias("follow-links"), visible_alias("follow-link"))]
    pub re_follow_links: bool,

    #[cfg(feature = "fatlink_regex")]
    /// 取消re匹配后，创建链接前的用户手动检查阶段
    #[arg(long, visible_alias("no-check"))]
    pub re_no_check: bool,

    #[cfg(feature = "fatlink_regex")]
    /// 对于re匹配的后所有内容，不按照原本目录（镜像）创建链接，
    /// 而是直接创建到[DST]中。
    /// 如果匹配的文件名有重复，则会拒绝创建并报错
    #[arg(long, visible_alias("flatten"))]
    pub re_output_flatten: bool,

    /// 覆盖同名已存在的符号链接，与--skip-exist-links互斥
    #[arg(
        long,
        visible_alias("overwrite"),
        visible_alias("overwrite-link"),
        conflicts_with = "skip_exist_links"
    )]
    pub overwrite_links: bool,

    /// --overwrite-links的较弱版本，但优先级高于--skip-exist-link，只覆盖损坏的符号链接.
    /// 默认为true, 暂不支持关闭
    #[arg(long, visible_alias("overwrite-broken"), default_value_t = true)]
    pub overwrite_broken_link: bool,

    /// 针对[DST]，跳过同名已存在的符号链接，与--overwrite-links互斥
    #[arg(
        long,
        visible_alias("skip-exist"),
        visible_alias("skip-exists"),
        visible_alias("skip-exist-link"),
        visible_alias("skip-exists-links"),
        conflicts_with = "overwrite_links"
    )]
    pub skip_exist_links: bool,

    /// 针对<SRC>，跳过损坏的符号链接.
    /// 默认为true, 暂不支持关闭
    #[arg(
        long,
        visible_alias("skip-broken"),
        visible_alias("skip-broken-link"),
        visible_alias("skip-broken-links"),
        default_value_t = true
    )]
    pub skip_broken_src_links: bool,

    #[cfg(feature = "save_log")]
    /// 在目标路径输出/保存/导出本次处理日志
    /// 若路径不存在，则将当前工作目录并重命名为fastlink-%y-%m-%d-%h-%m-%s.log
    #[arg(long)]
    pub save_log: Option<String>,

    /// 允许使用损坏的符号链接作为src (开了也不行，想都别想)
    #[arg(long)]
    pub allow_broken_src: bool,
}

/// 仅用于测试的Default实现
#[cfg(test)]
impl Default for Args {
    fn default() -> Self {
        Args {
            src: String::new(),
            dst: None,
            check: false,
            rm: false,
            keep_extention: false,
            make_dir: false,
            quiet: false,
            debug: false,
            #[cfg(feature = "fatlink_regex")]
            regex: None,
            #[cfg(feature = "fatlink_regex")]
            re_max_depth: None,
            only_file: false,
            only_dir: false,
            #[cfg(feature = "fatlink_regex")]
            re_follow_links: false,
            #[cfg(feature = "fatlink_regex")]
            re_no_check: false,
            #[cfg(feature = "fatlink_regex")]
            re_output_flatten: false,
            overwrite_links: false,
            overwrite_broken_link: true,
            skip_exist_links: false,
            skip_broken_src_links: true,
            #[cfg(feature = "save_log")]
            save_log: None,
            allow_broken_src: false,
        }
    }
}

/// 检查src
fn validate_src(s: &str) -> Result<String, String> {
    let path = std::path::Path::new(s).clean();

    if s.trim().is_empty() {
        Err(MyError::new(ErrorCode::InvalidInput, "路径不能为空或纯空格".into()).into())
    } else if path.components().count() == 0 {
        Err(MyError::new(ErrorCode::InvalidInput, "无效的路径格式".into()).into())
    } else {
        Ok(s.into())
    }
}

#[cfg(feature = "fatlink_regex")]
/// 检查re表达式
pub fn validate_regex(pattern: &str) -> Result<regex::Regex, String> {
    if pattern.trim().is_empty() {
        return Err(
            MyError::new(ErrorCode::InvalidInput, "正则表达式不能为空或纯空格".into()).into(),
        );
    }

    regex::Regex::new(pattern).map_err(|e| {
        MyError::new(
            ErrorCode::InvalidInput,
            format!("无效的正则表达式 '{}': {}", pattern, e),
        )
        .into()
    })
}

#[cfg(feature = "fatlink_regex")]
/// 检查re匹配时最大深度
fn validate_re_max_depth(s: &str) -> Result<usize, String> {
    if s.trim().is_empty() {
        return Err(MyError::new(
            ErrorCode::InvalidInput,
            "re匹配最大深度不能为空或纯空格".into(),
        )
        .into());
    }

    match s.parse::<i32>() {
        Ok(depth) if depth >= 0 => Ok(depth as usize),
        Ok(_) => Err(MyError::new(
            ErrorCode::InvalidInput,
            "re匹配最大深度必须为非负，默认4，为0则无限制".into(),
        )
        .into()),
        Err(_) => Err(MyError::new(
            ErrorCode::InvalidInput,
            format!("无效的深度值 '{}': 必须为非负整数", s),
        )
        .into()),
    }
}

// todo: 尽可能早完成，不放到task内
/// 根据make-dir参数、默认depth以及传入depth获取应有的depth
#[cfg(feature = "fatlink_regex")]
pub fn get_re_max_depth(make_dir: bool, re_max_depth: usize) -> usize {
    if make_dir {
        re_max_depth
    } else {
        log::warn!(
            "re匹配最大深度 {} 被设为1，因为没有传入参数`--make_dir`",
            re_max_depth
        );
        1
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_validate_src() {
//         // 有效路径测试
//         assert!(validate_src(r"C:\valid\path").is_ok());
//         assert!(validate_src("test.txt").is_ok());
//         assert!(validate_src(r"..\test.txt").is_ok());

//         // 无效路径测试
//         assert!(validate_src("").is_err());
//         assert!(validate_src("   ").is_err());
//     }
// }
