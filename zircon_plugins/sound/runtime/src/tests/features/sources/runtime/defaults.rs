// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../sources/runtime/defaults/enabled.rs"]
mod enabled;
#[path = "../../../sources/runtime/defaults/packaging.rs"]
mod packaging;
#[path = "defaults/root.rs"]
mod root;

pub(in super::super::super) use enabled::*;
pub(in super::super::super) use packaging::*;
pub(in super::super::super) use root::*;
