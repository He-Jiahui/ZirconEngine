pub(super) struct SplitLayoutSources {
    pub(super) parent: &'static str,
    pub(super) submit_sources: &'static str,
    pub(super) source_extract_payloads: &'static str,
    pub(super) camera_loop_sharing: &'static str,
    pub(super) feedback_sidebands: &'static str,
    pub(super) status_docs: &'static str,
    pub(super) split_layout: &'static str,
    pub(super) split_layout_route: &'static str,
    pub(super) split_layout_source_inventory: &'static str,
    pub(super) split_layout_sources: &'static str,
    pub(super) split_layout_status_docs: &'static str,
    pub(super) source_inventory: &'static str,
    pub(super) runtime_15_plan: &'static str,
    pub(super) runtime_index: &'static str,
    pub(super) review_findings: &'static str,
    pub(super) structure_convention: &'static str,
    pub(super) module_doc: &'static str,
    pub(super) runtime_07_plan: &'static str,
    pub(super) hotspot_doc: &'static str,
    pub(super) status_rows: &'static str,
    pub(super) legacy_status_rows: &'static str,
    pub(super) status_slice: &'static str,
    pub(super) date_slice: &'static str,
    pub(super) session_note: &'static str,
}

impl SplitLayoutSources {
    pub(super) fn load() -> Self {
        Self {
            parent: include_str!("../../submit_context.rs"),
            submit_sources: include_str!("../sources.rs"),
            source_extract_payloads: include_str!("../source_extract_payloads.rs"),
            camera_loop_sharing: include_str!("../camera_loop_sharing.rs"),
            feedback_sidebands: include_str!("../feedback_sidebands.rs"),
            status_docs: include_str!("../status_docs.rs"),
            split_layout: include_str!("../split_layout.rs"),
            split_layout_route: include_str!("route.rs"),
            split_layout_source_inventory: include_str!("source_inventory.rs"),
            split_layout_sources: include_str!("sources.rs"),
            split_layout_status_docs: include_str!("status_docs.rs"),
            source_inventory: include_str!(
                "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
            ),
            runtime_15_plan: include_str!(
                "../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
            ),
            runtime_index: include_str!(
                "../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
            ),
            review_findings: include_str!(
                "../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
            ),
            structure_convention: include_str!(
                "../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
            ),
            module_doc: include_str!(
                "../../../../../../../docs/zircon_runtime/structure/module-convention.md"
            ),
            runtime_07_plan: include_str!(
                "../../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
            ),
            hotspot_doc: include_str!(
                "../../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md"
            ),
            status_rows: include_str!(
                "../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance/split_layout_rows.rs"
            ),
            legacy_status_rows: include_str!(
                "../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance/primary_guard_rows.rs"
            ),
            status_slice: include_str!(
                "../../../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/runtime07_script_maps/runtime07_split_layout_maps.rs"
            ),
            date_slice: include_str!(
                "../../../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/runtime07_script_maps/runtime07_split_layout_maps.rs"
            ),
            session_note: include_str!(
                "../../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
            ),
        }
    }
}

pub(super) fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should retain submit-context split anchor `{anchor}`"
        );
    }
}
