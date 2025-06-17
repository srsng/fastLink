use fastlink_core::utils::fs::mk_parents;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::{ErrorCode, MyError, MyResult};

fn get_state_path() -> String {
    dirs::config_dir()
        .map(|p| {
            let p = p.join(r"fastlink\desktop_setter\state.json");
            mk_parents(&p).expect("无法创建配置文件目标目录");
            // log::debug!("已创建配置文件目标目录：{}", p.display());
            p
        })
        .expect("无法确定配置目录")
        .to_str()
        .expect("无效的配置路径")
        .to_string()
}

// 全局配置单例
pub static DESKTOP_STATE: Lazy<AutoSaveState> =
    Lazy::new(|| AutoSaveState::new(get_state_path()).expect("无法初始化配置"));

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DesktopState {
    pub initial_path: Option<PathBuf>,
    pub initial_path_temp: Option<PathBuf>,
    pub cur_path: Option<PathBuf>,
    pub cur_target: Option<PathBuf>,
    pub usual_paths: HashMap<String, PathBuf>,
}

impl DesktopState {
    /// 从文件加载状态，如果文件不存在则返回默认状态
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            let data = fs::read_to_string(path)?;
            log::debug!("已从状态文件读取状态");
            if data.is_empty() {
                log::debug!("有状态文件，但为空状态");
                Ok(DesktopState::default())
            } else {
                Ok(serde_json::from_str(&data)?)
            }
        } else {
            log::debug!("无状态文件，使用空状态");
            Ok(DesktopState::default())
        }
    }

    /// 将状态保存到文件
    pub fn save(&self, path: impl AsRef<Path>) -> MyResult<()> {
        let path = path.as_ref();
        // log::debug!("序列化 {}", path.display());
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| MyError::new(ErrorCode::Unknown, format!("无法序列化状态数据: {e}")))?;
        // log::debug!("打开文件");
        let mut file = File::create(path)
            .map_err(|e| MyError::new(ErrorCode::IoError, format!("无法打开状态文件: {e}")))?;
        // log::debug!("写入");
        file.write_all(data.as_bytes())
            .map_err(|e| MyError::new(ErrorCode::Unknown, format!("无法序列化状态数据: {e}")))?;
        Ok(())
    }
}

pub struct AutoSaveState {
    state: Mutex<DesktopState>,
    path: String,
}

impl AutoSaveState {
    pub fn new(path: impl Into<String>) -> io::Result<Self> {
        let path = path.into();
        let state = DesktopState::load(&path)?;
        Ok(AutoSaveState {
            state: Mutex::new(state),
            path,
        })
    }

    pub fn state(&self) -> std::sync::MutexGuard<DesktopState> {
        self.state.lock().unwrap()
    }

    pub fn state_mut(&self) -> std::sync::MutexGuard<DesktopState> {
        self.state.lock().unwrap()
    }

    pub fn save(&self) -> MyResult<()> {
        log::debug!("保存状态中");
        let res = self.state().save(&self.path);
        if let Err(e) = res {
            log::error!("保存状态失败: {}", e);
            Err(e)
        } else {
            log::debug!("保存状态完成");
            Ok(())
        }
    }

    pub fn reset(&self) {
        let mut state = self.state_mut();
        state.initial_path = None;
        state.initial_path_temp = None;
        state.cur_path = None;
        state.cur_target = None;
        state.usual_paths = HashMap::new();
    }
}

impl Drop for AutoSaveState {
    fn drop(&mut self) {
        log::debug!("自动保存状态中");
        if let Err(e) = self.state().save(&self.path) {
            log::error!("保存状态失败: {}", e);
        }
        log::debug!("自动保存状态完成");
    }
}

impl fmt::Display for DesktopState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        // 前四个字段每行一个，字段名称左对齐，占用20字符
        writeln!(
            f,
            "{:<20}    {}",
            "initial_path",
            self.initial_path
                .as_ref()
                .map_or("None".to_string(), |p| p.display().to_string())
        )?;
        writeln!(
            f,
            "{:<20}    {}",
            "initial_path_temp",
            self.initial_path_temp
                .as_ref()
                .map_or("None".to_string(), |p| p.display().to_string())
        )?;
        writeln!(
            f,
            "{:<20}    {}",
            "cur_path",
            self.cur_path
                .as_ref()
                .map_or("None".to_string(), |p| p.display().to_string())
        )?;
        writeln!(
            f,
            "{:<20}    {}",
            "cur_target",
            self.cur_target
                .as_ref()
                .map_or("None".to_string(), |p| p.display().to_string())
        )?;

        // 处理 usual_paths 字段
        writeln!(f, "\n常用快捷名称-路径(usual_paths):")?;
        if self.usual_paths.is_empty() {
            writeln!(f, "    空，使用set目录携带参数-u添加")?;
        } else {
            // 找到最长的键的长度
            // let max_key_len = self.usual_paths.keys().map(|k| k.len()).max().unwrap_or(0);
            for (key, value) in &self.usual_paths {
                writeln!(f, "    {:<16}    {}", key, value.display(),)?;
            }
        }
        Ok(())
    }
}
