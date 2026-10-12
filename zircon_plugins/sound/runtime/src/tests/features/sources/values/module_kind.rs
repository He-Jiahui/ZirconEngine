// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../sources/values/module_kind/module.rs"]
mod module;
#[path = "../../../sources/values/module_kind/raw.rs"]
mod raw;

pub(in super::super::super) use module::*;
pub(in super::super::super) use raw::*;
