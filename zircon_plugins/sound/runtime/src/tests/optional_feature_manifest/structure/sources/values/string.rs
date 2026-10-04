// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
mod dependency;
mod feature;
mod module;
mod raw;

pub(in super::super::super) use dependency::*;
pub(in super::super::super) use feature::*;
pub(in super::super::super) use module::*;
pub(in super::super::super) use raw::*;
