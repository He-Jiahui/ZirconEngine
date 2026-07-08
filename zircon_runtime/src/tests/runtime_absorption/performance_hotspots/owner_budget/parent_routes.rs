use super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_performance_hotspots_parent_routes(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "performance_hotspots parent",
        sources.performance_parent,
        &[
            "mod artifact_render_diagnostics_splits;",
            "mod hotspot_inventory;",
            "mod owner_budget;",
            "mod scene_project_splits;",
            "mod submit_context;",
            "mod submit_error_paths;",
        ],
    );

    for moved_guard in [
        "fn runtime_07_submit_context_shares_large_extract_payloads",
        "fn runtime_07_submit_paths_return_errors_for_checked_viewport_records",
        "fn runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        "fn runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner",
        "fn runtime_07_project_io_folder_split_keeps_entry_and_converter_owners",
        "fn runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner",
        "fn runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
        "fn runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
    ] {
        assert!(
            !sources.performance_parent.contains(moved_guard),
            "performance_hotspots.rs should mount child owners instead of defining `{moved_guard}`"
        );
    }
}
