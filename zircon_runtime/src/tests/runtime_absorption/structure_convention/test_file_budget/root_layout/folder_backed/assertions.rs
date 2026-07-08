use super::{guard_names::GuardNames, sources::GuardSources};

#[path = "assertions/asset_children.rs"]
mod asset_children;
#[path = "assertions/parent_mounts.rs"]
mod parent_mounts;
#[path = "assertions/render_status_children.rs"]
mod render_status_children;
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
    render_status_children::assert_render_status_children(sources, guards);
    ui_children::assert_ui_children(sources);
}
