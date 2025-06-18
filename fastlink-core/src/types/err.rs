use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Unknown = -1,
    ParentNotExist = 1,
    FileNotExist = 4,
    InvalidInput = 2,
    IoError = 3,
    PermissionDenied = 5,
    DirectoryNotEmpty = 6,
    FailToMakeDir = 100,
    FailAtMakeLink = 101,
    FailToGetPathParent = 102,
    FailToGetFileMetadata = 103,
    TargetNotALink = 104,
    TargetLinkExists = 105,
    TargetExistsAndNotLink = 106,
    FailToDelLink = 107,
    SkipExistingLink = 108,

    DuplicateTarget = 201,
    BrokenSymlink = 202,
    SrcEqDst = 203,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCode::Unknown => write!(f, "Unknown"),
            ErrorCode::ParentNotExist => write!(f, "ParentNotExist"),
            ErrorCode::FileNotExist => write!(f, "FileNotExist"),
            ErrorCode::InvalidInput => write!(f, "InvalidInput"),
            ErrorCode::IoError => write!(f, "IoError"),
            ErrorCode::FailAtMakeLink => write!(f, "Fail At Make Link"),
            ErrorCode::FailToMakeDir => write!(f, "Fail To Make Dir"),
            ErrorCode::FailToGetPathParent => write!(f, "Fail To Get Path Parent"),
            ErrorCode::FailToGetFileMetadata => write!(f, "Fail To Get File Metadata"),
            ErrorCode::TargetNotALink => write!(f, "Target is Not A Link"),
            ErrorCode::TargetLinkExists => write!(f, "Target Link Already Exists"),
            ErrorCode::SkipExistingLink => write!(f, "SkipExistingLink"),
            ErrorCode::FailToDelLink => write!(f, "Fail To Del Link"),
            ErrorCode::TargetExistsAndNotLink => write!(f, "Target Exists And is Not a Link"),
            ErrorCode::DuplicateTarget => write!(f, "Duplicate Target"),
            ErrorCode::BrokenSymlink => write!(f, "Broken Symlink"),
            ErrorCode::SrcEqDst => write!(f, "InvalidInput: <SRC> is Equal to [DST]"),
            ErrorCode::PermissionDenied => write!(f, "PermissionDenied"),
            ErrorCode::DirectoryNotEmpty => write!(f, "DirectoryNotEmpty"),
        }
    }
}

#[derive(Debug)]
pub struct MyError {
    pub code: ErrorCode,
    pub msg: String,
}

pub type MyResult<T> = Result<T, MyError>;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.msg)
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
    pub fn log(&self) {
        log::error!("{}", self);
    }
    pub fn warn(&self) {
        log::warn!("{}", self);
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
