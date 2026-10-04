// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
mod dependency_signature;
mod module_signature;
mod pending_manifest;
mod root;
mod static_manifest;

pub(in super::super) use dependency_signature::*;
pub(in super::super) use module_signature::*;
pub(in super::super) use pending_manifest::*;
pub(in super::super) use root::*;
pub(in super::super) use static_manifest::*;
