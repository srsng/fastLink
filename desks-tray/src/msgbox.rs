use windows::core::PCWSTR;
use windows::{core::*, Win32::UI::WindowsAndMessaging::*};

/// 显示错误弹窗
pub fn msgbox_error(message: String) {
    unsafe {
        let wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        MessageBoxW(None, PCWSTR(wide.as_ptr()), w!("错误"), MB_ICONERROR);
    }
}
