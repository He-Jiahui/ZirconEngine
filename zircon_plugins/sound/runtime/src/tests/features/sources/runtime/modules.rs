// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "modules/entry.rs"]
mod entry;
#[path = "../../../sources/runtime/modules/ordering.rs"]
mod ordering;
#[path = "../../../sources/runtime/modules/projection.rs"]
mod projection;
#[path = "modules/root.rs"]
mod root;
#[path = "../../../sources/runtime/modules/signature.rs"]
mod signature;

pub(in super::super::super) use entry::*;
pub(in super::super::super) use ordering::*;
pub(in super::super::super) use projection::*;
pub(in super::super::super) use root::*;
pub(in super::super::super) use signature::*;
