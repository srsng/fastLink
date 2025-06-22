use std::collections::HashMap;

use anyhow::{anyhow, Ok};
use desks_core::DESKTOP_STATE;
use fastlink_core::types::err::MyResult;
use tray_icon::{
    menu::{Menu, MenuItem, Submenu},
    TrayIconBuilder,
};

use crate::{
    msgbox::msgbox_error,
    tray::{handler::keep_layout::*, icon::load_icon},
};

// pub fn setup() -> anyhow::Result<()> {
//     let menu = Menu::new();
//     let (kl_items, keep_layout_id2handler) = setup_keep_layout(&menu)?;
//     Ok(())
// }

pub fn setup_icon(menu: Box<Menu>) -> anyhow::Result<tray_icon::TrayIcon> {
    // 加载托盘图标
    let icon = load_icon();

    // 构建托盘图标
    TrayIconBuilder::new()
        .with_menu(menu)
        .with_tooltip("Desks")
        .with_icon(icon)
        .build()
        .map_err(|e| {
            msgbox_error(format!("无法构建托盘图标: {}", e));
            anyhow!("无法构建托盘图标: {}", e)
        })
}

type KeepLayoutItemsAndHandlers = (Vec<MenuItem>, HashMap<String, fn() -> MyResult<bool>>);

/// 初始化keep_layout部分的菜单
///
/// 错误只会来自菜单添加项`menu.append(item)`
pub fn setup_keep_layout(
    menu: &Menu,
    submenu: Option<bool>,
) -> anyhow::Result<KeepLayoutItemsAndHandlers> {
    let submenu = submenu.unwrap_or(false);

    // 初始化items
    let kl_item1 = MenuItem::new("从文件恢复布局...", true, None);
    let kl_item2 = MenuItem::new("快速备份布局", true, None);
    let kl_item3 = MenuItem::new("保存布局到...", true, None);
    let kl_item4 = MenuItem::new("从备份中恢复布局", true, None);

    // 初始化handlers
    let mut keep_layout_id2handler: HashMap<String, fn() -> MyResult<bool>> = HashMap::new();
    keep_layout_id2handler.insert(kl_item1.id().0.clone(), handle_ask_restore_layout_from_dsv);
    keep_layout_id2handler.insert(kl_item2.id().0.clone(), handle_quick_bakcup_cur_layout);
    keep_layout_id2handler.insert(kl_item3.id().0.clone(), handle_ask_save_layout_to);
    keep_layout_id2handler.insert(
        kl_item4.id().0.clone(),
        handle_ask_restore_layout_at_backup_dir,
    );

    let items = vec![kl_item1, kl_item2, kl_item3, kl_item4];

    // 加入菜单项
    if submenu {
        // 初始化子菜单
        let sub_menu = Submenu::new("管理桌面布局", true);
        for item in &items {
            sub_menu.append(item)?;
        }
        todo!("好像不能添加submenu")
        // menu.append(sub_menu)?;
    } else {
        for item in &items {
            menu.append(item)?;
        }
    }

    Ok((items, keep_layout_id2handler))
}

const MAX_NAME_ITEMS: usize = 10;
type UsualNameItemsAndId2Name = (Vec<MenuItem>, HashMap<String, String>);

/// 初始化usual_names部分的菜单
///
/// 错误只会来自菜单添加项`menu.append(item)`
pub fn setup_usual_names(menu: &Menu) -> anyhow::Result<UsualNameItemsAndId2Name> {
    // 菜单项 快捷名称
    let mut id2name = HashMap::new();
    let names: Vec<_> = {
        let state = DESKTOP_STATE.state();
        let usual_paths = &state.usual_paths;
        usual_paths.keys().take(MAX_NAME_ITEMS).cloned().collect()
    };

    let mut items = Vec::with_capacity(names.len());

    if names.is_empty() {
        let null_item = MenuItem::new("(没有快捷项目，使用命令行工具desks.exe添加)", false, None);
        menu.append(&null_item)?;
        items.push(null_item);
    } else {
        for (i, name) in names.into_iter().enumerate() {
            // let acc = Some(format!("Digit{}", (i + 1) % 10).parse().unwrap());
            let menu_item = MenuItem::new(format!("{}. {}", (i + 1) % 10, name), true, None);
            id2name.insert(menu_item.id().0.clone(), name);
            menu.append(&menu_item)?;
            items.push(menu_item);
        }
    }
    Ok((items, id2name))
}
