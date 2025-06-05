use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Unknown = -1,
    ParentNotExist = 1,
    InvalidInput = 2,
    IoError = 3,
    FailToMakeDir = 100,
    FailAtMakeLink = 101,
    FailToGetFathParent = 102,
    FailToGetFileMetadata = 103,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCode::Unknown => write!(f, "Unknown"),
            ErrorCode::ParentNotExist => write!(f, "ParentNotExist"),
            ErrorCode::InvalidInput => write!(f, "InvalidInput"),
            ErrorCode::IoError => write!(f, "IoError"),
            ErrorCode::FailAtMakeLink => write!(f, "Fail At Make Link"),
            ErrorCode::FailToMakeDir => write!(f, "Fail To Make Dir"),
            ErrorCode::FailToGetFathParent => write!(f, "Fail To Get Fath Parent"),
            ErrorCode::FailToGetFileMetadata => write!(f, "Fail To Get File Metadata"),
        }
    }
}

#[derive(Debug)]
pub struct MyError {
    pub code: ErrorCode,
    pub msg: String,
}

#[derive(Debug)]
pub enum MyResult<T> {
    Ok(T),
    Err(MyError),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.code, self.msg)
    }
}

impl From<MyError> for String {
    fn from(err: MyError) -> Self {
        format!("{}: {}", err.code, err.msg)
    }
}

impl MyError {
    pub fn new(code: ErrorCode, msg: String) -> Self {
        MyError { code, msg }
    }
    pub fn log(self) {
        log::error!("{}", self);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     // #[test]
//     // fn test_fmt() {
//     //     println!("{:?}", ErrorCode::InvalidInput);
//     // }
// }
