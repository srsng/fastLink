use tray_icon::menu::AboutMetadataBuilder;
use tray_icon::menu::MenuItem;
use tray_icon::menu::PredefinedMenuItem;

pub fn menu_item_about() -> PredefinedMenuItem {
    let text = Some("关于");
    let metadata = Some(
        AboutMetadataBuilder::new()
            .name(Some("Desks - Tray Edition"))
            .authors(Some(vec!["srsnng (github)".into()]))
            .comments(Some("Desks的托盘GUI版本"))
            .website(Some("github.com/srsng/fastLink"))
            .build(),
    );
    PredefinedMenuItem::about(text, metadata)
}

#[allow(unused)]
pub fn menu_item_help() -> MenuItem {
    todo!()
}
