use windows::core::PCWSTR;
use windows::{core::*, Win32::UI::WindowsAndMessaging::*};

/// 显示错误弹窗
pub fn msgbox_error(message: String) {
    unsafe {
        let wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        MessageBoxW(
            None,
            PCWSTR(wide.as_ptr()),
            w!("desks-tray 错误"),
            MB_ICONERROR,
        );
    }
}

/// 显示警告弹窗
pub fn msgbox_warn(message: String) {
    unsafe {
        let wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        MessageBoxW(
            None,
            PCWSTR(wide.as_ptr()),
            w!("desks-tray 警告"),
            MB_ICONWARNING,
        );
    }
}

/// 显示信息弹窗
pub fn msgbox_info(message: String) {
    unsafe {
        let wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        MessageBoxW(None, PCWSTR(wide.as_ptr()), w!("desks-tray 信息"), MB_OK);
    }
}
