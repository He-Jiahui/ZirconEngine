//! 校验可移植的单段项目名称，供清单、模板和路径构造前复用。
//! 此处只验证名称文本；文件系统中的路径解析与创建由调用方负责。

#[path = "error.rs"]
mod error;
#[path = "validate.rs"]
mod validate;

pub use error::ProjectNameError;
pub use validate::validate_project_name;
