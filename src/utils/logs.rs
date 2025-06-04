// use crate::types::err::MyError;
use clap::builder::styling::{Color, RgbColor, Style};
use std::{
    io::Write,
    // path::PathBuf
};

pub fn init_log(quiet: bool, debug: bool, _save_log: &Option<String>) {
    // 初始化日志系统
    let mut builder = env_logger::Builder::new();

    // todo 实装日志输出
    // if save_log.is_some() {
    //     let log_file = File::create("app.log").unwrap();
    //     let path = parse_save_path(save_log);
    //     builder = builder.target(target).target(Target::Stdout);
    // }

    builder
        .format(|buf, record| {
            let time = chrono::Local::now().format("%H:%M:%S");
            let level = record.level();
            let level_style = buf.default_level_style(level);

            // 设置时间颜色（灰色）
            let time_style = Style::new().fg_color(Some(Color::Rgb(RgbColor(150, 150, 150))));

            // 格式化输出
            writeln!(
                buf,
                "{time_style}{time}{time_style:#} {level_style}{level}{level_style:#} {}",
                record.args()
            )
        })
        .filter_level(if quiet {
            log::LevelFilter::Off
        } else if debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();
    log::debug!("log init.")
}

// fn parse_save_path(save_log: Option<String>) -> Result<PathBuf, MyError> {}
