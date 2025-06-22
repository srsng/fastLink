use crate::{layout::IconEntry, win_err_to_myerr};
use fastlink_core::types::err::MyResult;
use std::collections::HashMap;

use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::{
            Controls::{LVM_GETITEMCOUNT, LVM_REDRAWITEMS, LVM_SETITEMPOSITION},
            // Shell::{IShellFolder, SHGetDesktopFolder},
            WindowsAndMessaging::{FindWindowExW, FindWindowW, SendMessageW},
        },
    },
};

#[cfg(feature = "system-com")]
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

#[cfg(feature = "system-com")]
struct ComGuard;
#[cfg(feature = "system-com")]
impl ComGuard {
    pub fn new() -> Option<Self> {
        let res = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if res.is_err() {
            log::debug!("fail to CoInitializeEx");
            None
        } else {
            Some(ComGuard {})
        }
    }
}
#[cfg(feature = "system-com")]
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

/// 使用桌面 ListView 句柄获取桌面图标数量
fn get_item_count(hwnd_sys_list_view32: HWND) -> isize {
    unsafe {
        SendMessageW(
            hwnd_sys_list_view32,
            LVM_GETITEMCOUNT,
            Some(WPARAM(0)),
            Some(LPARAM(0)),
        )
        .0
    }
}

pub fn totally_fresh() -> windows::core::Result<()> {
    let hwnd_sys_list_view32 = get_desktop_listview()?;
    redraw_items(hwnd_sys_list_view32);
    #[cfg(feature = "update-window")]
    update_window(hwnd_sys_list_view32);
    Ok(())
}

/// redraw 桌面 ListView 的项
fn redraw_items(hwnd_sys_list_view32: HWND) -> bool {
    unsafe {
        SendMessageW(
            hwnd_sys_list_view32,
            LVM_REDRAWITEMS,
            Some(WPARAM(0)),
            Some(LPARAM(get_item_count(hwnd_sys_list_view32) - 1)),
        )
        .0 != 0
    }
}

#[cfg(feature = "update-window")]
fn update_window(hwnd: HWND) -> bool {
    use windows::Win32::Graphics::Gdi::UpdateWindow;
    unsafe { UpdateWindow(hwnd).as_bool() }
}

/// 设置桌面图标位置的辅助函数
fn set_icon_pos(hwnd_sys_list_view32: HWND, idx: usize, point: (i32, i32)) -> bool {
    unsafe {
        // 构造 LPARAM：低 16 位为 x 坐标，高 16 位为 y 坐标
        let lparam = ((point.1 as u32) << 16) | (point.0 as u32 & 0xFFFF);
        let res = SendMessageW(
            hwnd_sys_list_view32,
            LVM_SETITEMPOSITION,
            Some(WPARAM(idx)),
            Some(LPARAM(lparam as isize)),
        );
        res.0 != 0
    }
}

/// 获取桌面 ListView 句柄
///
/// C/C++ 参考： https://www.cnblogs.com/marszhw/p/11087886.html
pub fn get_desktop_listview() -> windows::core::Result<HWND> {
    unsafe {
        // 获取 Program 句柄
        // A
        // let window = w!("Program Manager");
        // B
        let window = PCWSTR::null();
        let hwnd_parent = FindWindowW(w!("Progman"), window)?;

        // 获取 SHELLDLL_DefView 子窗口
        let hwnd_shelldll_def_view = FindWindowExW(
            Some(hwnd_parent),
            Some(HWND::default()),
            w!("SHELLDLL_DefView"),
            PCWSTR::null(),
        )?;

        // 获取 SysListView32 子窗口
        // A
        let window = w!("FolderView");
        // B
        // let window = PCWSTR::null();
        let hwnd_sys_list_view32 = FindWindowExW(
            Some(hwnd_shelldll_def_view),
            Some(HWND::default()),
            w!("SysListView32"),
            window,
        )?;

        Ok(hwnd_sys_list_view32)
    }
}

pub fn set_icon_layout(
    icons: &Vec<&IconEntry>,
    name2idx: Option<HashMap<&String, usize>>,
) -> MyResult<()> {
    let listview = get_desktop_listview().map_err(win_err_to_myerr)?;

    #[cfg(debug_assertions)]
    println!("{:?}", icons);

    let mut res = Vec::with_capacity(icons.len());
    for (i, entriy) in icons.iter().enumerate() {
        let idx = if let Some(ref name2idx) = name2idx {
            *name2idx.get(&entriy.name).unwrap_or(&i)
        } else {
            i
        };
        #[cfg(debug_assertions)]
        println!("idx {idx} - {:?}", entriy.point());

        let b = set_icon_pos(listview, idx, entriy.point());
        res.push((idx, b));
    }
    log::debug!("设定图标位置：{res:?}");
    Ok(())
}
