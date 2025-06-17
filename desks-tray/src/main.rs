#![windows_subsystem = "windows"]

pub mod msgbox;
use crate::{msgbox::msgbox_error, tray::load_icon};
use anyhow::Result;
use desks_core::DESKTOP_STATE;
use std::collections::HashMap;
pub mod tray;
use desks_core::handler::{original::handle_desktop_origin, usual::handle_desktop_usual};
use fastlink_core::utils::logs::LogIniter;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::{
    menu::{accelerator::Accelerator, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};

use crate::tray::menu_item_about;

fn main() -> Result<()> {
    // 初始化日志
    let debug = false;
    let save_log = None;
    LogIniter::new(false, debug, save_log).init();

    let menu = Menu::new();
    let sep = PredefinedMenuItem::separator();
    let about = menu_item_about();

    menu.append(&about)?;
    // vec.push(&sep);
    // let test_item = MenuItem::new("test", true, Some("shift+alt+KeyC".parse().unwrap()));
    // vec.push(&test_item);
    menu.append(&sep)?;

    // 菜单项 快捷名称
    const MAX_NAME_ITEMS: usize = 10;
    let mut id2name = HashMap::new();
    let names: Vec<_> = {
        let state = DESKTOP_STATE.state();
        let usual_paths = &state.usual_paths;
        usual_paths.keys().take(MAX_NAME_ITEMS).cloned().collect()
    };

    if names.is_empty() {
        let null_item = MenuItem::new("(没有快捷项目，使用命令行工具desks.exe添加)", false, None);
        menu.append(&null_item)?;
    } else {
        for (i, name) in names.into_iter().enumerate() {
            // let acc: Accelerator = format!("F{}", i + 1).parse().unwrap();
            let acc: Accelerator = format!("Digit{}", (i + 1) % 10).parse().unwrap();
            let menu_item = MenuItem::new(&name, true, Some(acc));

            id2name.insert(menu_item.id().0.clone(), name);
            menu.append(&menu_item)?;
        }
    }

    menu.append(&sep)?;
    // 菜单项 退出
    let exit_item = PredefinedMenuItem::quit(Some("退出"));
    menu.append(&exit_item)?;

    // 创建带用户事件的事件循环
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // 加载托盘图标
    let icon = load_icon();

    // 构建托盘图标
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Desks")
        .with_icon(icon)
        .build()
        .inspect_err(|e| msgbox_error(format!("无法构建托盘图标: {}", e)))?;

    // 设置菜单事件处理
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    // 设置托盘图标事件处理
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::TrayEvent(event));
    }));

    let event_handler =
        move |event: Event<'_, UserEvent>,
              _window_target: &tao::event_loop::EventLoopWindowTarget<UserEvent>,
              control_flow: &mut ControlFlow| {
            *control_flow = ControlFlow::Wait;

            match event {
                // 菜单项事件
                Event::UserEvent(UserEvent::MenuEvent(MenuEvent { id })) => {
                    // 点击 快捷名称
                    if let Some(name) = id2name.get(&id.0) {
                        if let Err(e) = handle_desktop_usual(name.clone()) {
                            e.log();
                            msgbox_error(format!("{}", e));
                        }
                    }
                }
                // 双击托盘图标设回原始桌面
                Event::UserEvent(UserEvent::TrayEvent(TrayIconEvent::DoubleClick { .. })) => {
                    if let Err(e) = handle_desktop_origin() {
                        e.log();
                        msgbox_error(format!("{}", e));
                    }
                }

                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        };

    // 运行事件循环
    event_loop.run(event_handler);
}

// 自定义用户事件类型
#[derive(Debug)]
enum UserEvent {
    MenuEvent(MenuEvent),
    TrayEvent(TrayIconEvent),
}
