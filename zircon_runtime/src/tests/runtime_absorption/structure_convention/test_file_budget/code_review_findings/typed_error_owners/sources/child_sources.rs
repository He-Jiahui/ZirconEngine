//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::*;
use super::*;

#[path = "child_sources/delegation_sources.rs"]
mod delegation_sources;
#[path = "child_sources/root_sources.rs"]
mod root_sources;
#[path = "child_sources/source_blobs.rs"]
mod source_blobs;
#[path = "child_sources/source_helper_sources.rs"]
mod source_helper_sources;
#[path = "child_sources/structure_guard.rs"]
mod structure_guard;

pub(super) use delegation_sources::*;
pub(super) use root_sources::*;
pub(super) use source_blobs::*;
pub(super) use source_helper_sources::*;

#[test]
fn runtime_15_typed_error_source_inventory_child_sources_are_folder_backed() {
    structure_guard::assert_typed_error_source_inventory_child_sources_are_folder_backed();
}
