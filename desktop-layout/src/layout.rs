use crate::{ErrorCode, MyError, MyResult};
use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

// #[derive(Default)]
// pub enum IconLayoutFileExt {
//     #[default]
//     Dsv,
//     Bin,
// }

// // 桌面布局数据文件拓展名
// impl ToString for IconLayoutFileExt {
//     fn to_string(&self) -> String {
//         match self {
//             Self::Bin => "bin".into(),
//             Self::Dsv => "dsv".into(),
//         }
//     }
// }

/// 桌面图标条目结构
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconEntry {
    pub name: String, // 图标名称
    pub x: i32,       // X 坐标
    pub y: i32,       // Y 坐标
}

impl IconEntry {
    pub fn point(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

/// 桌面布局结构
///
/// 可以转为dsv格式字符串，或由其转来
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IconLayout {
    pub icon_count: usize,       // 图标数量
    pub entries: Vec<IconEntry>, // 图标条目
}

impl IconLayout {
    /// 将 IconLayout 格式化为 dsv文本
    pub fn fmt2dsv(self) -> String {
        self.into()
    }

    /// 保存layout到文件
    pub fn dump<P: AsRef<Path>>(&self, path: P) -> MyResult<()> {
        let formatted: String = self.into();
        // 创建文件并写入
        let mut file = File::create(&path)
            .map_err(|e| MyError::new(ErrorCode::IoError, format!("创建桌面布局文件失败: {e}")))?;
        file.write_all(formatted.as_bytes())
            .map_err(|e| MyError::new(ErrorCode::IoError, format!("写入桌面布局文件失败: {e}")))
    }

    /// 过滤，以icon的name为key，对两个layout求交集：过滤掉在self但不在another的 icon name,
    ///
    /// 返回过滤后的 Vec<&IconEntry> 以及name映射到self原始idx的HashMap
    pub fn filter(&self, another: &IconLayout) -> (Vec<&IconEntry>, HashMap<&String, usize>) {
        let old_names: HashSet<&String> = another.entries.iter().map(|e| &e.name).collect();

        // 以icon的name为key，对两个layout求交集：过滤掉在self但不在another的 icon name, 并记录 idx
        let mut name2idx: HashMap<&String, usize> = HashMap::new();
        for (i, e) in self.entries.iter().enumerate() {
            if old_names.contains(&&e.name) {
                name2idx.insert(&e.name, i);
            }
        }

        (
            self.entries
                .iter()
                .filter(|e| name2idx.contains_key(&e.name))
                .collect(),
            name2idx,
        )
    }
}

/// 将 IconLayout 格式化为 dsv文本
impl From<&IconLayout> for String {
    fn from(val: &IconLayout) -> Self {
        let mut output = String::new();
        // 头部信息
        output.push_str("Desks (github/fastLink)\n");
        // todo 分辨率
        output.push_str("# no res info\n");
        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
        output.push_str(&format!("# {}\n", timestamp));

        // 图标条目
        for entry in &val.entries {
            output.push_str(&format!("{:.0} {:.0} {}\n", entry.x, entry.y, entry.name));
        }

        output
    }
}

impl From<IconLayout> for String {
    fn from(val: IconLayout) -> Self {
        (&val).into()
    }
}

impl TryFrom<&str> for IconLayout {
    type Error = MyError;

    /// 从 DSV 文件解析 IconLayout
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lines: Vec<&str> = value.lines().collect();
        // 验证头部 (skipped)

        let mut entries = Vec::new();
        for line in lines.iter() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            } else if trimmed.starts_with('#') {
                // process `# info`  (skipped)
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() != 3 {
                log::debug!("跳过无效条目: {}", line);
                continue;
            }
            let name = parts[2];
            let x = parts[0].parse::<i32>().inspect_err(|e| {
                MyError::new(ErrorCode::Unknown, format!("{name} 解析 X 坐标失败: {e}")).debug();
            });

            let y = parts[1].parse::<i32>().map_err(|e| {
                MyError::new(ErrorCode::Unknown, format!("{name} 解析 Y 坐标失败: {e}")).debug();
            });

            if x.is_err() || y.is_err() {
                continue;
            }
            let (name, x, y) = (name.to_string(), x.unwrap(), y.unwrap());
            // log::debug!("解析条目: {} ({}, {})", name, x, y);
            entries.push(IconEntry { name, x, y });
        }

        Ok(IconLayout {
            icon_count: entries.len(),
            entries,
        })
    }
}

impl TryFrom<&String> for IconLayout {
    type Error = MyError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let str = &value[..];
        Self::try_from(str)
    }
}

impl TryFrom<String> for IconLayout {
    type Error = MyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let str = &value[..];
        Self::try_from(str)
    }
}

/// 从文件读取布局数据
///
/// - `path` 应该是经过 `get_layout_data_file_path` 处理的路径
pub fn read_layout_from_path<P: AsRef<Path>>(path: P) -> MyResult<IconLayout> {
    let path = path.as_ref();
    log::debug!("try reading layout from {}", path.display());

    let extension = path.extension().and_then(|s| s.to_str());
    match extension {
        Some("dsv") => read_layout_from_dsv(path),
        // Some("bin") => read_layout_from_bin(path),
        _ => Err(MyError::new(
            ErrorCode::InvalidInput,
            format!("不支持的格式: {}", path.display()),
        )),
    }
}

/// 通过路径读取 IconLayout （DSV 格式）
fn read_layout_from_dsv<P: AsRef<Path>>(path: P) -> MyResult<IconLayout> {
    let path = path.as_ref();
    let mut text = String::new();
    let mut file = File::open(path)
        .map_err(|e| MyError::new(ErrorCode::IoError, format!("打开桌面布局数据文件失败: {e}")))?;
    file.read_to_string(&mut text).map_err(|e| {
        MyError::new(
            ErrorCode::IoError,
            format!("从文件中读取桌面布局数据失败: {e}"),
        )
    })?;
    let icon_layouts = IconLayout::try_from(&text)?;
    Ok(icon_layouts)
}
