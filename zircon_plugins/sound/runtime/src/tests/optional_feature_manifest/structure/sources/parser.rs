// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
mod entry;
mod line;
mod pending;
mod root;
mod section;
mod state;

pub(in super::super) use entry::*;
pub(in super::super) use line::*;
pub(in super::super) use pending::*;
pub(in super::super) use root::*;
pub(in super::super) use section::*;
pub(in super::super) use state::*;
