// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../sources/types/dependency_signature.rs"]
mod dependency_signature;
#[path = "../../../sources/types/module_signature.rs"]
mod module_signature;
#[path = "../../../sources/types/pending_manifest.rs"]
mod pending_manifest;
#[path = "types/root.rs"]
mod root;
#[path = "../../sources/types/static_manifest.rs"]
mod static_manifest;

pub(in super::super) use dependency_signature::*;
pub(in super::super) use module_signature::*;
pub(in super::super) use pending_manifest::*;
pub(in super::super) use root::*;
pub(in super::super) use static_manifest::*;
