// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../sources/runtime/dependencies/entry.rs"]
mod entry;
#[path = "../../../sources/runtime/dependencies/ordering.rs"]
mod ordering;
#[path = "../../../runtime/dependencies/projection.rs"]
mod projection;
#[path = "../../../sources/runtime/dependencies/root.rs"]
mod root;
#[path = "../../../sources/runtime/dependencies/signature.rs"]
mod signature;

pub(in super::super::super) use entry::*;
pub(in super::super::super) use ordering::*;
pub(in super::super::super) use projection::*;
pub(in super::super::super) use root::*;
pub(in super::super::super) use signature::*;
