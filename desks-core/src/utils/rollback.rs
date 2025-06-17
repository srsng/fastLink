use crate::{ErrorCode, MyError, MyResult};
use fastlink_core::utils::link::{del_exists_link, mklink};
use std::path::PathBuf;
use std::{fs, iter::zip};

#[derive(Default)]
pub struct Transaction {
    undo_ops_name: Vec<String>,
    // 存储已执行的操作及其撤销函数
    undo_ops: Vec<Box<dyn FnOnce() -> MyResult<()>>>,
}

impl Transaction {
    pub fn new() -> Self {
        Transaction {
            undo_ops_name: Vec::new(),
            undo_ops: Vec::new(),
        }
    }

    /// 添加一个操作及其撤销函数
    pub fn add_op<F, U>(&mut self, op: F, undo: U, name: Option<String>) -> MyResult<()>
    where
        F: FnOnce() -> MyResult<()>,
        U: FnOnce() -> MyResult<()> + 'static,
    {
        let name = name.unwrap_or_default();
        // 执行操作
        log::debug!("执行操作{}中", name);
        op().inspect_err(|e| {
            log::warn!("执行操作 {} 失败: {}", name, e);
        })?;
        log::debug!("执行操作{}成功", name);
        // 如果成功，记录撤销函数
        self.undo_ops_name.push(name);
        self.undo_ops.push(Box::new(undo));
        Ok(())
    }

    /// 提交事务（清空撤销操作）
    pub fn commit(mut self) -> MyResult<()> {
        self.undo_ops.clear();
        self.undo_ops_name.clear();
        Ok(())
    }

    /// 回滚所有操作
    pub fn rollback(&mut self) -> MyResult<()> {
        // 从后向前执行撤销操作
        for (name, undo) in zip(
            self.undo_ops_name.drain(..).rev(),
            self.undo_ops.drain(..).rev(),
        ) {
            log::debug!("回滚操作 {} 中", name);
            undo().inspect_err(|e| {
                log::warn!("回滚操作 {} 失败: {}", name, e);
            })?;
            log::debug!("回滚操作 {} 成功", name);
        }
        Ok(())
    }

    pub fn add_op_rename_dir(
        &mut self,
        from: PathBuf,
        to: PathBuf,
        name: Option<String>,
    ) -> MyResult<()> {
        let (op, undo) = op_rename_dir(from, to);
        self.add_op(op, undo, name)
    }

    pub fn add_op_mklink(
        &mut self,
        original: PathBuf,
        link: PathBuf,
        name: Option<String>,
    ) -> MyResult<()> {
        let (op, undo) = op_mklink(original, link);
        self.add_op(op, undo, name)
    }

    pub fn add_op_del_link(
        &mut self,
        original: PathBuf,
        link: PathBuf,
        name: Option<String>,
    ) -> MyResult<()> {
        let (op, undo) = op_del_link(original, link);
        self.add_op(op, undo, name)
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        // 如果未显式提交或回滚，自动回滚
        if !self.undo_ops.is_empty() {
            log::warn!("正在回滚操作");
            if let Err(e) = self.rollback() {
                eprintln!("回滚失败: {}", e);
            }
        }
    }
}

fn op_del_link(
    original: PathBuf,
    link: PathBuf,
) -> (impl FnOnce() -> MyResult<()>, impl FnOnce() -> MyResult<()>) {
    let src = original;
    let dst = link;
    // let src_c = src.clone();
    let dst_c = dst.clone();

    let op = move || del_exists_link(&dst_c, true, Some(false)).map(|_| ());
    let undo = move || mklink(&src, &dst, Some(false), None, None, None, Some(true)).map(|_| ());

    (op, undo)
}

// fn op_null() -> (impl FnOnce() -> MyResult<()>, impl FnOnce() -> MyResult<()>) {
//     let op = move || Ok(());
//     let undo = move || Ok(());

//     (op, undo)
// }

fn op_rename_dir(
    from: PathBuf,
    to: PathBuf,
) -> (impl FnOnce() -> MyResult<()>, impl FnOnce() -> MyResult<()>) {
    let from_clone = from.clone();
    let to_clone = to.clone();

    let op = move || {
        fs::rename(&from_clone, &to_clone).map_err(move |e| {
            MyError::new(
                ErrorCode::IoError,
                format!(
                    "尝试将{}重命名为临时名称{}失败：{e}",
                    from_clone.display(),
                    to_clone.display()
                ),
            )
        })
    };
    let undo = || {
        // 预期内：源路径不存在，目标路径存在
        if !from.exists() && to.exists() {
            fs::rename(&to, &from).map_err(move |e| {
                MyError::new(
                    ErrorCode::IoError,
                    format!(
                        "尝试将{}重命名为临时名称{}失败：{e}",
                        to.display(),
                        from.display()
                    ),
                )
            })
        // 问题：两路径都不存在
        } else if !from.exists() && !to.exists() {
            Err(MyError {
                code: ErrorCode::Unknown,
                msg: format!("重大问题：{}与{}不见了！", from.display(), to.display()),
            })
        // 安全：源路径存在，目标路径不存在
        } else if from.exists() && !to.exists() {
            Ok(())
        // 问题：两路径都存在
        } else {
            log::warn!("{}与{}都已存在", &from.display(), &to.display());
            let empty = fs::read_dir(&to)
                .map_err(|e| {
                    MyError::new(
                        ErrorCode::IoError,
                        format!(
                            "{}与{}都已存在，且检查前者是否为空目录失败: {e}",
                            &from.display(),
                            &to.display()
                        ),
                    )
                })?
                .any(|_entry| true);
            // to为空
            if empty {
                fs::rename(&to, &from).map_err(move |e| {
                    MyError::new(
                        ErrorCode::IoError,
                        format!(
                            "尝试将{}重命名为临时名称{}失败：{e}",
                            &to.display(),
                            &from.display(),
                        ),
                    )
                })?;
                Ok(())
            // to非空
            } else {
                Err(MyError::new(
                    ErrorCode::DirectoryNotEmpty,
                    format!(
                        "{}与{}都已存在，且前者非空目录",
                        &from.display(),
                        &to.display()
                    ),
                ))
            }
        }
    };

    (op, undo)
}

fn op_mklink(
    original: PathBuf,
    link: PathBuf,
) -> (impl FnOnce() -> MyResult<()>, impl FnOnce() -> MyResult<()>) {
    let src = original;
    let dst = link;
    // let src_c = src.clone();
    let dst_c = dst.clone();

    let op = move || mklink(&src, &dst, Some(false), None, None, None, Some(true)).map(|_| ());

    let undo = move || del_exists_link(&dst_c, true, Some(true)).map(|_| ());

    (op, undo)
}
