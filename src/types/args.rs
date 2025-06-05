use crate::types::err::{ErrorCode, MyError};
use clap::Parser;
use path_clean::PathClean;
use regex::Regex;

pub const DEFAULT_RE_MAX_DEPTH: usize = 4;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A tool to make symlink fastly and smartly
一个智能且方便的符号链接创建工具",
    long_about = r#"
Example：
    // 在当前目录创建一个名为document的符号链接
    fastlink document.txt

    // 在当前目录创建一个名为img-link.jpg的符号链接
    fastlink image.jpg img-link -k

    // 在当前目录的子目录tmp中创建名为output.csv的符号链接，若tmp目录不存在将退出
    //（添加--make-dir或--md参数选项则自动创建)
    fastlink data.csv tmp/output --keep-extention

    // 在当前目录的父目录创建名为data符号链接，指向data.csv (不建议, Not Recommended)
    fastlink data.csv ../
"#
)]
pub struct Args {
    /// 源文件/源目录路径
    #[arg(required = true, value_parser = validate_src)]
    pub src: String,

    /// 目标路径，可选，区分文件拓展名。
    /// 为空则自动以<SRC>路径名称填充；当<SRC>为文件，[DST]为目录时，自动以<SRC>路径名称填充
    pub dst: Option<String>,

    /// 自动保留<SRC>的文件拓展名到[DST]。(不会去除)
    /// 保留拓展名之后可以通过对符号链接双击、运行等操作让系统使用默认应用打开或执行。
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

    /// 对<SRC>内容应用正则表达式，匹配项将于[DST]相应创建，
    /// 若启用make_dir参数，则还会尝试对<SRC>的子目录以及更深层(默认最大4层)进行匹配并创建，
    /// 若要限制深度，使用--re-max-depth参数。
    /// 匹配的路径不受--keep_extention参数影响。
    #[arg(long, visible_alias("re"), value_parser = validate_regex)]
    pub regex: Option<Regex>,

    /// 限制regex匹配的最大深度，启用make_dir参数时，默认4层，否则为1层,
    /// 传入0表示没有层数限制.
    /// 该参数数值非负.
    #[arg(long, visible_alias("re-depth"), value_parser = validate_re_max_depth)]
    pub re_max_depth: Option<usize>,

    /// 只处理文件，同时传入only_dir则出错
    #[arg(long, conflicts_with = "only_dir", visible_alias("F"))]
    pub only_file: bool,

    /// 只处理目录，同时传入only_file则出错
    #[arg(long, conflicts_with = "only_file", visible_alias("D"))]
    pub only_dir: bool,

    /// re匹配过程中，深入读取符号链接进行匹配
    #[arg(long, visible_alias("follow_links"), visible_alias("follow_link"))]
    pub re_follow_links: bool,

    /// 取消re匹配后，创建链接前的用户手动检查阶段
    #[arg(long, visible_alias("no_check"))]
    pub re_no_check: bool,

    /// 对于re匹配的后所有内容，不按照原本目录（镜像）创建链接，
    /// 而是直接创建到[DST]中。
    /// 如果匹配的文件名有重复，则会拒绝创建并报错
    #[arg(long, visible_alias("flatten"))]
    pub re_output_flatten: bool,

    /// 覆盖同名已存在的符号链接
    #[arg(long, visible_alias("overwrite"), visible_alias("overwrite_link"))]
    pub overwrite_links: bool,

    /// 在目标路径输出/保存/导出本次处理日志
    /// 若路径不存在，则将当前工作目录并重命名为fastlink-%y-%m-%d-%h-%m-%s.log
    #[arg(long)]
    pub save_log: Option<String>,
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

/// 检查re表达式
pub fn validate_regex(pattern: &str) -> Result<Regex, String> {
    if pattern.trim().is_empty() {
        return Err(
            MyError::new(ErrorCode::InvalidInput, "正则表达式不能为空或纯空格".into()).into(),
        );
    }

    Regex::new(pattern).map_err(|e| {
        MyError::new(
            ErrorCode::InvalidInput,
            format!("无效的正则表达式 '{}': {}", pattern, e),
        )
        .into()
    })
}

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

pub fn get_re_max_depth(make_dir: bool, re_max_depth: usize) -> usize {
    if make_dir {
        re_max_depth
    } else {
        log::warn!("{} is not used for `--make_dir` is not true", re_max_depth);
        1
    }
}

// pub fn handle_onlys(args: &Args) -> Result<(), MyError> {
//     if args.only_dir && args.only_file {
//         Err(MyError::new(
//             ErrorCode::InvalidInput,
//             "only_dir与only_file不应该同时传入！".to_string(),
//         ))
//     } else {
//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_src() {
        // 有效路径测试
        assert!(validate_src(r"C:\valid\path").is_ok());
        assert!(validate_src("test.txt").is_ok());
        assert!(validate_src(r"..\test.txt").is_ok());

        // 无效路径测试
        assert!(validate_src("").is_err());
        assert!(validate_src("   ").is_err());
    }
}
