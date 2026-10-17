//! 表示持久化的项目资产身份及其可选定位提示。
//! GUID 决定资产身份，路径提示供注册表解析或迁移修复使用；本模块不验证两者指向同一资产。

#[path = "construction.rs"]
mod construction;
#[path = "error.rs"]
mod error;
#[path = "serde.rs"]
mod serde;
#[path = "validation.rs"]
mod validation;
#[path = "value.rs"]
mod value;

pub use error::AssetRefError;
pub use value::AssetRef;
