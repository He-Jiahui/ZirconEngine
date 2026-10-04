// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
mod capabilities;
mod defaults;
mod dependencies;
mod identity;
mod modules;
mod root;
mod signature;

pub(in super::super) use capabilities::*;
pub(in super::super) use defaults::*;
pub(in super::super) use dependencies::*;
pub(in super::super) use identity::*;
pub(in super::super) use modules::*;
pub(in super::super) use root::*;
pub(in super::super) use signature::*;
