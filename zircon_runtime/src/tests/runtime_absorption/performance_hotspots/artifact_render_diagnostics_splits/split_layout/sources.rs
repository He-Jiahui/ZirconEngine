pub(super) struct SplitLayoutSources {
    pub(super) parent: &'static str,
    pub(super) artifact_cache_payload: &'static str,
    pub(super) render_product_diagnostics: &'static str,
    pub(super) split_layout: &'static str,
    pub(super) split_layout_route: &'static str,
    pub(super) split_layout_source_inventory: &'static str,
    pub(super) split_layout_sources: &'static str,
    pub(super) split_layout_status_docs: &'static str,
    pub(super) source_inventory: &'static str,
    pub(super) runtime_07_archive: &'static str,
}

impl SplitLayoutSources {
    pub(super) fn load() -> Self {
        Self {
            parent: include_str!("../../artifact_render_diagnostics_splits.rs"),
            artifact_cache_payload: include_str!("../artifact_cache_payload.rs"),
            render_product_diagnostics: include_str!("../render_product_diagnostics.rs"),
            split_layout: include_str!("../split_layout.rs"),
            split_layout_route: include_str!("route.rs"),
            split_layout_source_inventory: include_str!("source_inventory.rs"),
            split_layout_sources: include_str!("sources.rs"),
            split_layout_status_docs: include_str!("status_docs.rs"),
            source_inventory: include_str!(
                "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
            ),
            runtime_07_archive: include_str!(
                "../../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
            ),
        }
    }
}

pub(super) fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should retain artifact/render diagnostics split anchor `{anchor}`"
        );
    }
}
