// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../sources/runtime/signature/entry.rs"]
mod entry;
#[path = "../../../sources/runtime/signature/root.rs"]
mod root;

pub(in super::super::super) use entry::*;
pub(in super::super::super) use root::*;
