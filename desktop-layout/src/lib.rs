pub mod handler;
pub mod layout;
pub mod layout2;
pub mod utils;

use fastlink_core::types::err::{ErrorCode, MyError, MyResult};

fn win_err_to_myerr(e: windows::core::Error) -> MyError {
    MyError {
        code: ErrorCode::Unknown,
        msg: e.message(),
    }
}
