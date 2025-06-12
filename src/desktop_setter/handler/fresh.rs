use windows::Win32::UI::Shell::SHChangeNotify;
use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST};

pub fn handle_fresh_desktop() {
    unsafe {
        // 通知系统刷新桌面
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
    log::info!("桌面已刷新");
}
