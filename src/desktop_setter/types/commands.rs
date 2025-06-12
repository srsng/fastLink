use std::path::PathBuf;

use crate::{
    types::err::{ErrorCode, MyError},
    utils::path::get_path_type,
};
use clap::Subcommand;

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Commands {
    /// 初始化，可以使用reset恢复
    Init,

    /// 设置一个路径为桌面库，必须是一个目录或指向目录的符号链接
    Set {
        /// 源文件/源目录路径，表示的是符号链接指向的路径(Point at who)。
        #[arg(required = true, value_parser = validate_new_desktop_dir_path)]
        new_desktop_dir_path: PathBuf,

        /// 自动创建不存在的目录
        #[arg(long, visible_alias("md"))]
        make_dir: bool,

        // /// 覆盖同名已存在的符号链接，与--skip-exist-links互斥
        // #[arg(
        //     long,
        //     visible_alias("overwrite"),
        //     visible_alias("overwrite-link"),
        //     conflicts_with = "skip_exist_links"
        // )]
        // overwrite_links: bool,

        // 将给定路径设置为常用快捷名称，需要传入名称<name>，之后可以通过usual <name>快速切换
        #[arg(short, long, value_parser = validate_usual)]
        usual: Option<String>,
    },

    /// 获取当前状态
    State,

    /// 设置Desktop库为原始的目录
    #[clap(visible_alias = "o", visible_alias = "ori")]
    Original,

    /// 快速切换为通过set --usual <name>设置的一些常用路径，使用state命令查看已设置列表
    #[clap(visible_alias = "u", visible_alias = "switch")]
    Usual { name: String },

    /// 重置所有数据，并将Desktop库恢复为原始状态
    Reset,
}
// / 1. 是否存在于文件系统(FileNotExist)
// / 2. 是否是损坏的符号链接(BrokenSymlink)
// / 3. 是否是存在但不是符号链接(TargetExistsAndNotLink)
// / 4. 是否已存在的符号链接(TargetLinkExists)

/// 检查new_desktop_dir_path
/// 需是目录或指向目录的符号链接
fn validate_new_desktop_dir_path(s: &str) -> Result<PathBuf, String> {
    let path = std::path::Path::new(s);

    if s.trim().is_empty() {
        Err(MyError::new(ErrorCode::InvalidInput, "路径不能为空或纯空格".into()).into())
    } else if path.components().count() == 0 {
        Err(MyError::new(ErrorCode::InvalidInput, "无效的路径格式".into()).into())
    // windows最短非盘符目录如C:\A
    } else if s.len() < 4 {
        Err(MyError::new(ErrorCode::InvalidInput, "无效的路径".into()).into())
    } else {
        match get_path_type(path) {
            Ok(_) => Ok(s.into()),
            Err(e) if e.code == ErrorCode::TargetExistsAndNotLink => Ok(s.into()),
            Err(e) if e.code == ErrorCode::TargetLinkExists => {
                if path.is_dir() {
                    Ok(s.into())
                } else {
                    Err("目标是个符号链接，但指向的不是一个目录".into())
                }
            }
            Err(mut e) if e.code == ErrorCode::BrokenSymlink => {
                e.msg = format!("目标是损坏的符号链接：{}", e.msg);
                Err(e.into())
            }
            Err(mut e) if e.code == ErrorCode::FileNotExist => {
                e.msg = format!("目标不存在：{}", e.msg);
                Err(e.into())
            }
            Err(mut e) if e.code == ErrorCode::FileNotExist => {
                e.msg = format!("目标不存在：{}", e.msg);
                Err(e.into())
            }
            Err(e) => Err(format!("检查路径失败：{e}")),
        }
    }
}

fn validate_usual(s: &str) -> Result<String, String> {
    if s.len() > 15 {
        Err("这——么长的东西还有必要算作快捷吗，最长16个字符！".into())
    } else {
        Ok(s.into())
    }
}
