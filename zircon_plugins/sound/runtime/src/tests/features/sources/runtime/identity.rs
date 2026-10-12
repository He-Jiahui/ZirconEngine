// 汇集本域源码快照供结构守卫检查职责归属；这些编译时文本不参与运行时清单构造。
#[path = "../../../sources/runtime/identity/display_name.rs"]
mod display_name;
#[path = "identity/id.rs"]
mod id;
#[path = "../../../sources/runtime/identity/owner_plugin.rs"]
mod owner_plugin;
#[path = "identity/root.rs"]
mod root;

pub(in super::super::super) use display_name::*;
pub(in super::super::super) use id::*;
pub(in super::super::super) use owner_plugin::*;
pub(in super::super::super) use root::*;
