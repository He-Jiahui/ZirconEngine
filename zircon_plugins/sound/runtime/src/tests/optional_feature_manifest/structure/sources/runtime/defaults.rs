// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
mod enabled;
mod packaging;
mod root;

pub(in super::super::super) use enabled::*;
pub(in super::super::super) use packaging::*;
pub(in super::super::super) use root::*;
