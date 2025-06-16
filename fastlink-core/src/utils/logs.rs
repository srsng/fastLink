use clap::builder::styling::{Color, RgbColor, Style};
use std::io::Write;

#[cfg(feature = "save-log")]
use crate::types::err::{ErrorCode, MyError};
#[cfg(feature = "save-log")]
use std::fs::File;
#[cfg(feature = "save-log")]
use std::io;
#[cfg(feature = "save-log")]
use std::path::PathBuf;

#[cfg(feature = "fastlink-regex")]
// 设置文件颜色（绿色）
pub const FILE_STYLE: Style = Style::new().fg_color(Some(Color::Rgb(RgbColor(19, 161, 14))));
#[cfg(feature = "fastlink-regex")]
// 设置父目录颜色（灰色）
pub const PARENT_STYLE: Style = Style::new().fg_color(Some(Color::Rgb(RgbColor(150, 150, 150))));

#[cfg(feature = "save-log")]
// 实现多目标输出（stdout 和文件）
struct MultiWriter {
    stdout: io::Stdout,
    file: std::sync::Mutex<File>,
}

#[cfg(feature = "save-log")]
impl MultiWriter {
    fn new(file: File) -> Self {
        MultiWriter {
            stdout: io::stdout(),
            file: std::sync::Mutex::new(file),
        }
    }
}

#[cfg(feature = "save-log")]
impl Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use strip_ansi_escapes;
        // 写入 stdout
        let stdout_result = self.stdout.write(buf);

        // 去除 ANSI 颜色代码后写入文件
        let plain_text = strip_ansi_escapes::strip(buf);
        let file_result = self.file.lock().unwrap().write(&plain_text);

        // 返回 stdout 的写入字节数（优先考虑 stdout 的成功写入）
        stdout_result.or(file_result)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()?;
        self.file.lock().unwrap().flush()?;
        Ok(())
    }
}

pub struct LogIniter {
    quiet: bool,
    debug: bool,
    _save_log: Option<String>,
}

impl LogIniter {
    pub fn new(quiet: bool, debug: bool, _save_log: Option<String>) -> Self {
        LogIniter {
            quiet,
            debug,
            _save_log,
        }
    }

    pub fn init(self) {
        // 初始化日志系统
        let mut builder = env_logger::Builder::new();
        // 启用终端颜色输出
        builder
            .is_test(self.debug)
            .write_style(env_logger::WriteStyle::Always);

        #[cfg(feature = "save-log")]
        let mut log_file_path = PathBuf::new();

        #[cfg(feature = "save-log")]
        // 处理日志文件输出
        if let Some(log_path) = self._save_log.clone() {
            log_file_path = match parse_save_path(&log_path) {
                Ok(path) => path,
                Err(e) => {
                    log::warn!("日志路径解析失败: {}, 将使用默认路径", e);
                    default_log_path()
                }
            };

            match File::create(&log_file_path) {
                Ok(file) => {
                    let multi_writer = MultiWriter::new(file);
                    builder.target(env_logger::Target::Pipe(Box::new(multi_writer)));
                    log::info!("日志将保存至: {}", log_file_path.display());
                }
                Err(e) => {
                    log::warn!(
                        "无法创建日志文件 {}: {}, 日志仅输出到终端",
                        log_file_path.display(),
                        e
                    );
                }
            }
        }

        builder
        .format(move |buf, record| {
            let time = chrono::Local::now().format("%H:%M:%S");
            let level = record.level();
            let level_style = buf.default_level_style(level);

            // 设置时间颜色（灰色）
            let time_style = Style::new().fg_color(Some(Color::Rgb(RgbColor(150, 150, 150))));

            // 格式化输出
            if self.debug {
                // let module = record.module_path().unwrap_or("unknown_modul");     // 模块路径
                let file = record.file().unwrap_or("unknown_file");   // 文件名
                let file_line = record.line().unwrap_or(0);            // 行号
                let content = record.args();                        // 日志内容
                writeln!(
                    buf,
                    "{time_style}{time} {file}:{file_line}{time_style:#} {level_style}{level}{level_style:#} {content}",                           
                )
            } else {
                writeln!(
                    buf,
                    "{time_style}{time}{time_style:#} {level_style}{level}{level_style:#} {}",
                    record.args() // 日志内容
                )
            }
        })
        .filter_level(if self.quiet {
            log::LevelFilter::Off
        } else if self.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();
        log::debug!("log init.");

        #[cfg(feature = "save-log")]
        if self._save_log.is_some() {
            log::info!("日志将保存至: {}", log_file_path.display());
        }
    }
}

#[cfg(feature = "save-log")]
fn parse_save_path(save_log: &str) -> Result<PathBuf, MyError> {
    let path = PathBuf::from(save_log);

    // 规范化路径
    let normalized_path = if path.is_absolute() {
        path
    } else {
        crate::WORK_DIR.join(path)
    };

    // 检查路径合法性
    if normalized_path
        .to_string_lossy()
        .contains(['<', '>', ':', '"', '|', '?', '*'])
    {
        return Err(MyError::new(
            ErrorCode::InvalidInput,
            format!("日志路径 {} 包含非法字符", normalized_path.display()),
        ));
    }

    // 检查父目录并尝试创建
    if let Some(parent) = normalized_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                MyError::new(
                    ErrorCode::IoError,
                    format!("无法创建日志文件父目录 {}: {}", parent.display(), e),
                )
            })?;
        }
    }

    Ok(normalized_path)
}

#[cfg(feature = "save-log")]
fn default_log_path() -> PathBuf {
    let timestamp = chrono::Local::now().format("%y-%m-%d-%H-%M-%S");
    crate::WORK_DIR.join(format!("fastlink-{}.log", timestamp))
}
