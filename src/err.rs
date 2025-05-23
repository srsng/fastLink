#[derive(Debug)]
pub enum ErrorCode {
    Unknown = -1,
    ParentNotExist = 1,
}

#[derive(Debug)]
pub struct MyError {
    pub code: ErrorCode,
    pub msg: String,
}

impl MyError {
    pub fn new(code: ErrorCode, msg: String) -> Self {
        MyError { code, msg }
    }
}
