//! 将输入解析为规范化的相对路径值，并让 Serde 复用同一词法校验。
//! 解析不访问文件系统；实际目标是否位于项目根目录内仍由使用方验证。

#[path = "deserialize.rs"]
mod deserialize;
#[path = "error.rs"]
mod error;
#[path = "parse.rs"]
mod parse;
#[path = "value.rs"]
mod value;

pub use error::RelPathError;
pub use value::RelPath;
