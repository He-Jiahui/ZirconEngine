// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
mod defaults;
mod list;
mod raw;

pub(in super::super::super) use defaults::*;
pub(in super::super::super) use list::*;
pub(in super::super::super) use raw::*;
