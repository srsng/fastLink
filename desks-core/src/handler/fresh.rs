use windows::Win32::UI::Shell::{SHChangeNotify, SHCNF_FLUSH};
use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST};

pub fn handle_fresh_desktop() {
    unsafe {
        // 通知系统刷新桌面
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
}

pub fn handle_fresh_desktop_force() {
    unsafe {
        // SHCNF_FLUSH强制刷新
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST | SHCNF_FLUSH, None, None);
    }
}
