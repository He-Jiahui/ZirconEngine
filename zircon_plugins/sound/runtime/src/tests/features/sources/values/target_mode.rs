// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../sources/values/target_mode/list.rs"]
mod list;
#[path = "../../../sources/values/target_mode/module.rs"]
mod module;
#[path = "../../../sources/values/target_mode/raw.rs"]
mod raw;

pub(in super::super::super) use list::*;
pub(in super::super::super) use module::*;
pub(in super::super::super) use raw::*;
