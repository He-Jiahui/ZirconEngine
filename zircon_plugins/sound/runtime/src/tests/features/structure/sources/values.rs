// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../sources/values/array.rs"]
mod array;
#[path = "../../sources/values/boolean.rs"]
mod boolean;
#[path = "../../sources/values/module_kind.rs"]
mod module_kind;
#[path = "../../sources/values/packaging.rs"]
mod packaging;
#[path = "../../sources/values/string.rs"]
mod string;
#[path = "../../sources/values/target_mode.rs"]
mod target_mode;

pub(in super::super) use array::*;
pub(in super::super) use boolean::*;
pub(in super::super) use module_kind::*;
pub(in super::super) use packaging::*;
pub(in super::super) use string::*;
pub(in super::super) use target_mode::*;
