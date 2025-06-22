use std::env;
use tray_icon::Icon;

use crate::msgbox::msgbox_error;
pub fn load_icon() -> Icon {
    if let Some(icon) = load_relative_icon() {
        log::info!("Use relative icon");
        icon
    } else {
        log::info!("Use default icon");
        load_default_icon()
    }
}

/// 加载可执行文件同目录下的icon.ico或icon.png的图标
///
/// 注: 部分图标，可能为图片分辨率较大或不是正方形等，无法读取作为icon
pub fn load_relative_icon() -> Option<Icon> {
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            Icon::from_path(exe_dir.join("icon.ico"), None)
                .ok()
                .map_or(Icon::from_path(exe_dir.join("icon.png"), None).ok(), Some)
        } else {
            None
        }
    } else {
        None
    }
}

/// 加载嵌入二进制的图标
pub fn load_default_icon() -> Icon {
    Icon::from_resource(111, None).unwrap_or_else(|_e| {
        msgbox_error("无法加载默认图标".into());
        Icon::from_rgba(vec![0, 0, 0, 0], 1, 1).unwrap()
    })
}

// pub fn update_icon() -> Result<bool> {

//     Ok(flase)
// }

// pub fn load_icon_from_usual() -> Result<Icon> {

// }
