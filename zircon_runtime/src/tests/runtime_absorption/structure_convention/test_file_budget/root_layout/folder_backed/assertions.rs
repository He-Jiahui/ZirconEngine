//! 保护测试目录根布局的子模块职责、检查入口与源码规模；这些约束读取检出文本，具体业务结果由被列出的行为测试另行验证。
use super::{guard_names::GuardNames, sources::GuardSources};

#[path = "assertions/asset_children.rs"]
mod asset_children;
#[path = "assertions/parent_mounts.rs"]
mod parent_mounts;
#[path = "assertions/render_children.rs"]
mod render_children;
#[path = "assertions/runtime_scene_children.rs"]
mod runtime_scene_children;
#[path = "assertions/ui_children.rs"]
mod ui_children;

pub(super) fn assert_test_file_budget_root_is_folder_backed(
    sources: &GuardSources,
    guards: &GuardNames,
) {
    parent_mounts::assert_parent_mounts_and_moved_guards(sources, guards);
    asset_children::assert_asset_children(sources, guards);
    runtime_scene_children::assert_runtime_scene_children(sources, guards);
    render_children::assert_render_children(sources, guards);
    ui_children::assert_ui_children(sources);
}
