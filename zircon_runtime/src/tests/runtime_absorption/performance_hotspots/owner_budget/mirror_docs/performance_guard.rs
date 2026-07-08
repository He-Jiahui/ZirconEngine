use super::sources::MirrorDocsSources;

pub(super) fn assert_performance_guard_anchors(sources: &MirrorDocsSources) {
    let performance_guard_sources = sources.performance_guard_sources();

    for guard_anchor in [
        "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        "runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit",
        "runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed",
        "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
        "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
        "runtime_07_project_io_folder_split_keeps_entry_and_converter_owners",
        "runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner",
        "runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
        "runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
        "runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split",
        "runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_split",
        "runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_split",
        "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split",
        "runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_split",
        "runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split",
        "runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_split",
        "AnimationSceneFrameDiagnostics",
    ] {
        assert!(
            performance_guard_sources
                .iter()
                .any(|source| source.contains(guard_anchor)),
            "Runtime 07 guard anchor `{guard_anchor}` should stay visible to performance_hotpath_boundary"
        );
    }
}
