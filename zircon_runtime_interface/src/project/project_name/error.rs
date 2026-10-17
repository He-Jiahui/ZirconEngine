use thiserror::Error;

/// 项目名称不满足跨平台单文件名组件规则时返回的错误。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ProjectNameError {
    #[error("project name cannot be empty")]
    Empty,
    #[error("project name {value:?} must not contain leading or trailing whitespace")]
    SurroundingWhitespace { value: String },
    #[error("project name {value:?} must be one filename component")]
    NotSingleComponent { value: String },
    #[error("project name {value:?} contains a character forbidden in portable filenames")]
    ForbiddenCharacter { value: String },
    #[error("project name {value:?} cannot end with a dot or space")]
    WindowsTrailingAlias { value: String },
    #[error("project name {value:?} is a reserved Windows filename")]
    WindowsReserved { value: String },
}
