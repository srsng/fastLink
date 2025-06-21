// NOTE: 通过写注册表不能响应修改DESKTOP布局
// 一些愚蠢的经验
// *不会编译或使用

use crate::{ErrorCode, MyError, MyResult};
use winreg::enums::*;
use winreg::RegKey;
use winreg::RegValue;

const DESKTOP_LAYOUT_PATH: &str = r"Software\Microsoft\Windows\Shell\Bags\1\Desktop";
const DESKTOP_LAYOUT_KEYNAME: &str = r"IconLayout";
// const DESKTOP_LAYOUT_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Streams\Desktop";
// const DESKTOP_LAYOUT_KEYNAME: &str = r"TaskbarWinXP";

/// 读取注册表获取桌面布局数据
pub fn get_layout_from_reg() -> MyResult<Vec<u8>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(DESKTOP_LAYOUT_PATH)
        .map_err(|e| MyError::new(ErrorCode::IoError, format!("读取注册表失败: {e}")))?;

    let data: Vec<u8> = key
        .get_raw_value(DESKTOP_LAYOUT_KEYNAME)
        .map_err(|e| {
            MyError::new(
                ErrorCode::IoError,
                format!("获取布局信息时获取布局项失败: {e}"),
            )
        })?
        .bytes;
    Ok(data)
}

/// 获取可写的桌面布局注册表键
fn get_writeable_layout_key() -> MyResult<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(DESKTOP_LAYOUT_PATH, KEY_WRITE)
        .map_err(|e| {
            MyError::new(
                ErrorCode::IoError,
                format!("获取可写的注册表桌面布局键失败: {e}"),
            )
        })?;
    Ok(key)
}

/// bytes转注册表图标布局二进制
fn bytes2layout_value(data: Vec<u8>) -> RegValue {
    RegValue {
        vtype: REG_BINARY,
        bytes: data,
    }
}

/// 将布局数据写入注册表
fn write_layout_to_reg(data: Vec<u8>) -> MyResult<()> {
    // 获取注册表布局键并写入
    let key = get_writeable_layout_key()?;
    key.set_raw_value(DESKTOP_LAYOUT_KEYNAME, &bytes2layout_value(data))
        .map_err(|e| {
            MyError::new(
                ErrorCode::IoError,
                format!("写入注册表桌面布局数据失败: {e}"),
            )
        })?;
    Ok(())
}

// /// 解析从注册表中读取的 IconLayout 数据
// fn parse_icon_layout(data: &[u8]) -> MyResult<IconLayout> {
//     let mut entries = Vec::new();
//     let mut cursor = std::io::Cursor::new(data);

//     // 读取头部（16 字节填充 + 8 字节元数据 + 8 字节图标数量）
//     if data.len() < 32 {
//         return Err(MyError::new(
//             ErrorCode::Unknown,
//             "数据太短，无法解析头部".into(),
//         ));
//     }
//     cursor.set_position(16); // 跳过 16 字节填充
//     let metadata = cursor
//         .read_u64::<LittleEndian>()
//         .map_err(|e| MyError::new(ErrorCode::Unknown, format!("读取元数据失败: {e}")))?;
//     let icon_count = cursor
//         .read_u64::<LittleEndian>()
//         .map_err(|e| MyError::new(ErrorCode::Unknown, format!("读取图标数量失败: {e}")))?
//         as usize;

//     log::debug!("图标数量: {}", icon_count);

//     // 读取图标名称
//     let mut names = Vec::new();
//     while cursor.position() < data.len() as u64 {
//         // 读取字符串长度（4 字节）
//         if cursor.position() + 8 > data.len() as u64 {
//             // 数据不足
//             log::warn!("字符串数据不足，终止解析");
//             break;
//         }

//         let str_len = cursor
//             .read_u32::<LittleEndian>()
//             .map_err(|e| MyError::new(ErrorCode::Unknown, format!("读取字符串长度失败: {e}")))?
//             as usize;

//         // 跳过 4 字节填充
//         cursor.set_position(cursor.position() + 4);

//         // 读取 UTF-16 字符串
//         if cursor.position() + (str_len * 2) as u64 > data.len() as u64 {
//             // return Err(MyError::new(ErrorCode::Unknown, "字符串数据不完整".into()));
//             log::warn!("字符串数据不完整，长度: {}", str_len);
//             break;
//         }

//         let mut name_bytes = vec![0u8; str_len * 2];
//         cursor
//             .read_exact(&mut name_bytes)
//             .map_err(|e| MyError::new(ErrorCode::Unknown, format!("读取字符串失败: {e}")))?;
//         let name = String::from_utf16(
//             &name_bytes
//                 .chunks_exact(2)
//                 .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
//                 .collect::<Vec<u16>>(),
//         )
//         .map_err(|e| MyError::new(ErrorCode::Unknown, format!("UTF-16 解码失败: {e}")))?;
//         names.push(name);

//         // 尝试读取终止符
//         if cursor.position() + 4 <= data.len() as u64 {
//             cursor.set_position(cursor.position() + 4);
//         } else {
//             log::warn!("终止符数据不足，跳过");
//             break;
//         }
//     }

//     // 读取坐标（每个图标 8 字节：x 和 y 各 4 字节 i32）
//     let mut coordinates = Vec::new();
//     while cursor.position() + 8 <= data.len() as u64 {
//         let x = cursor
//             .read_i32::<LittleEndian>()
//             .map_err(|e| MyError::new(ErrorCode::Unknown, format!("读取 X 坐标失败: {e}")))?;
//         let y = cursor
//             .read_i32::<LittleEndian>()
//             .map_err(|e| MyError::new(ErrorCode::Unknown, format!("读取 Y 坐标失败: {e}")))?;
//         coordinates.push((x, y));
//         log::debug!("解析坐标: ({}, {})", x, y);
//     }

//     // 确保名称和坐标数量匹配
//     if names.len() != coordinates.len() || names.len() != icon_count {
//         return Err(MyError::new(
//             ErrorCode::Unknown,
//             format!(
//                 "数据不一致: 图标数量={}，名称数量={}，坐标数量={}",
//                 icon_count,
//                 names.len(),
//                 coordinates.len()
//             ),
//         ));
//     }

//     // 组合名称和坐标
//     for (name, (x, y)) in names.into_iter().zip(coordinates) {
//         log::debug!("解析图标: {} ({}, {})", name, x, y);
//         entries.push(IconEntry { name, x, y });
//     }

//     Ok(IconLayout {
//         metadata,
//         icon_count,
//         entries,
//     })
// }

fn read_layout_from_bin<P: AsRef<Path>>(path: P) -> MyResult<IconLayout> {
    let path = path.as_ref();
    let mut data: Vec<u8> = Vec::new();
    let mut file = File::open(path)
        .map_err(|e| MyError::new(ErrorCode::IoError, format!("打开桌面布局数据文件失败: {e}")))?;
    file.read_to_end(&mut data).map_err(|e| {
        MyError::new(
            ErrorCode::IoError,
            format!("从文件中读取桌面布局数据失败: {e}"),
        )
    })?;
    // let regvalue = bytes2layout_value(data);
    not_to_do_now!("data转IconLayouts")
}
