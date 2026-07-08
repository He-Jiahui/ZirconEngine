use super::super::{assert_contains_all, sources::OwnerBudgetSources};

pub(super) fn assert_artifact_render_diagnostics_routes(sources: &OwnerBudgetSources) {
    assert_contains_all(
        "artifact/render diagnostics route",
        sources.artifact_render_diagnostics,
        &[
            "#[path = \"artifact_render_diagnostics_splits/artifact_cache_payload.rs\"]",
            "#[path = \"artifact_render_diagnostics_splits/render_product_diagnostics.rs\"]",
            "#[path = \"artifact_render_diagnostics_splits/split_layout.rs\"]",
        ],
    );
    assert_contains_all(
        "artifact/render diagnostics support children",
        &format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}",
            sources.artifact_render_diagnostics_artifact_cache_payload,
            sources.artifact_render_diagnostics_render_product_diagnostics,
            sources.artifact_render_diagnostics_split_layout,
            sources.artifact_render_diagnostics_split_layout_route,
            sources.artifact_render_diagnostics_split_layout_source_inventory,
            sources.artifact_render_diagnostics_split_layout_sources,
            sources.artifact_render_diagnostics_split_layout_status_docs
        ),
        &[
            "fn runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
            "fn runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed",
            "fn runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split",
            "runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_split",
            "artifact_render_diagnostics_splits/split_layout/source_inventory.rs",
            "assert_artifact_render_diagnostics_split_docs",
        ],
    );
}
