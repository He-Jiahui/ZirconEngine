// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
mod array;
mod boolean;
mod module_kind;
mod packaging;
mod string;
mod target_mode;

pub(in super::super) use array::*;
pub(in super::super) use boolean::*;
pub(in super::super) use module_kind::*;
pub(in super::super) use packaging::*;
pub(in super::super) use string::*;
pub(in super::super) use target_mode::*;
