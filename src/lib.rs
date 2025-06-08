#[cfg(feature = "desktop_setter")]
pub mod desktop_setter;
pub mod types;
pub mod utils;

lazy_static::lazy_static! {
    pub static ref WORK_DIR: std::path::PathBuf = std::env::current_dir().expect("Failed to get initial work directory");
}
