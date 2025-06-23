#![windows_subsystem = "windows"]

mod msgbox;
mod tray;

use crate::{
    msgbox::{msgbox_error, msgbox_warn},
    tray::{
        info::menu_item_about,
        setup::{setup_icon, setup_keep_layout, setup_usual_names},
    },
};

use desks_core::handler::{original::handle_desktop_origin, usual::handle_desktop_usual_setby};
use fastlink_core::utils::logs::LogIniter;
use fastlink_core::{types::err::MyResult, utils::fs::mk_parents};

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::{
    menu::{Menu, MenuEvent, PredefinedMenuItem},
    TrayIconEvent,
};

fn main() -> anyhow::Result<()> {
    // 初始化日志
    let debug = true;
    let save_log = Some(get_log_path());
    LogIniter::new(false, debug, save_log).init();

    let instance = single_instance::SingleInstance::new("desks-tray").unwrap();
    if !instance.is_single() {
        msgbox_warn("请勿多开".into());
        return Ok(());
    }

    let menu = Menu::new();
    let sep = PredefinedMenuItem::separator();
    let about = menu_item_about();

    menu.append(&about)?;
    // let test_item = MenuItem::new("test", true, Some("shift+alt+KeyC".parse().unwrap()));
    // menu.append(&sep)?;
    // menu.append(&test_item)?;
    menu.append(&sep)?;
    // 管理桌面布局菜单块
    let (_kl_items, keep_layout_id2handler) = setup_keep_layout(&menu, Some(false))?;
    menu.append(&sep)?;
    // 桌面常用名称菜单块
    let (_items, id2name) = setup_usual_names(&menu)?;
    menu.append(&sep)?;
    // 菜单项 退出
    let exit_item = PredefinedMenuItem::quit(Some("退出"));
    menu.append(&exit_item)?;

    // 创建带用户事件的事件循环
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // 构建托盘图标
    let box_menu = Box::new(menu);
    let _tray_icon = setup_icon(box_menu)?;

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

    type WindowTarget<'a> = &'a tao::event_loop::EventLoopWindowTarget<UserEvent>;
    let event_handler = move |event: Event<'_, UserEvent>,
                              _window_target: WindowTarget,
                              control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Wait;

        match event {
            // 菜单项事件
            Event::UserEvent(UserEvent::MenuEvent(MenuEvent { id })) => {
                let res = match id.0 {
                    // 点击 快捷名称
                    id if id2name.contains_key(&id) => {
                        let name = id2name.get(&id).unwrap();
                        handle_desktop_usual_setby(name)
                    }
                    // 手动布局保存/读取操作
                    id if keep_layout_id2handler.contains_key(&id) => {
                        let handler_ = keep_layout_id2handler.get(&id).unwrap();
                        handler_()
                    }
                    _ => {
                        log::debug!("unhandled userevent menuid");
                        Ok(false)
                    }
                };
                handle_result(res);
            }
            // 双击托盘图标设回原始桌面
            Event::UserEvent(UserEvent::TrayEvent(TrayIconEvent::DoubleClick { .. })) => {
                let res = handle_desktop_origin();
                handle_result(res);
            }

            // 不存在的窗口
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

/// 处理错误，Ok里的东西不重要时挺好用
fn handle_result<T: std::fmt::Debug>(res: MyResult<T>) -> Option<T> {
    if let Err(e) = res {
        e.log();
        msgbox_error(format!("{}", e));
        None
    } else {
        let ok_ = res.unwrap();
        log::debug!("{:?}", ok_);
        Some(ok_)
    }
}

/// 获取日志路径
fn get_log_path() -> String {
    let timestamp = chrono::Local::now().format("%y-%m-%d-%H-%M-%S");
    let file_name = format!("desks-tray-log-{}.txt", timestamp);
    dirs::config_dir()
        .map(|p| {
            let p = p.join(r"fastlink\desktop_setter\log").join(file_name);
            mk_parents(&p).expect("无法创建配置文件目标目录");
            p
        })
        .expect("无法确定配置目录")
        .to_str()
        .expect("无效的配置路径")
        .to_string()
}

// 自定义用户事件类型
#[derive(Debug)]
enum UserEvent {
    MenuEvent(MenuEvent),
    TrayEvent(TrayIconEvent),
}
