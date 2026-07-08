use super::super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_scene_project_routes(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "scene/project split route",
        sources.scene_project_splits,
        &[
            "#[path = \"scene_project_splits/dynamic_session_event.rs\"]",
            "#[path = \"scene_project_splits/project_io.rs\"]",
            "#[path = \"scene_project_splits/scene_asset.rs\"]",
            "#[path = \"scene_project_splits/split_layout.rs\"]",
        ],
    );
    assert_contains_all(
        "scene/project support children",
        &format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            sources.scene_project_splits_dynamic_session_event,
            sources.scene_project_splits_project_io,
            sources.scene_project_splits_scene_asset,
            sources.scene_project_splits_split_layout,
            sources.scene_project_splits_split_layout_route,
            sources.scene_project_splits_split_layout_source_inventory,
            sources.scene_project_splits_split_layout_sources,
            sources.scene_project_splits_split_layout_status_docs
        ),
        &[
            "fn runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner",
            "fn runtime_07_project_io_folder_split_keeps_entry_and_converter_owners",
            "fn runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner",
            "fn runtime_15_runtime_07_scene_project_guard_child_owner_split",
            "runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_split",
            "scene_project_splits/split_layout/source_inventory.rs",
            "assert_scene_project_split_docs",
        ],
    );
}
