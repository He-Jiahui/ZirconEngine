// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../sources/runtime/capabilities/entry.rs"]
mod entry;
#[path = "../../../sources/runtime/capabilities/ordering.rs"]
mod ordering;
#[path = "../../../runtime/capabilities/projection.rs"]
mod projection;
#[path = "../../../sources/runtime/capabilities/root.rs"]
mod root;

pub(in super::super::super) use entry::*;
pub(in super::super::super) use ordering::*;
pub(in super::super::super) use projection::*;
pub(in super::super::super) use root::*;
