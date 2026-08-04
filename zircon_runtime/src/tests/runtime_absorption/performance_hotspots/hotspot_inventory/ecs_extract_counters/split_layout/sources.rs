pub(super) struct SplitLayoutSources {
    pub(super) parent: &'static str,
    pub(super) query_change: &'static str,
    pub(super) extract_cache: &'static str,
    pub(super) asset_animation: &'static str,
    pub(super) frame_diagnostics: &'static str,
    pub(super) split_layout: &'static str,
    pub(super) split_layout_route: &'static str,
    pub(super) split_layout_source_inventory: &'static str,
    pub(super) split_layout_sources: &'static str,
    pub(super) split_layout_status_docs: &'static str,
    pub(super) source_inventory: &'static str,
    pub(super) runtime_15_plan: &'static str,
    pub(super) runtime_index: &'static str,
    pub(super) runtime_07_plan: &'static str,
    pub(super) review_findings: &'static str,
    pub(super) structure_convention: &'static str,
    pub(super) module_doc: &'static str,
    pub(super) hotspot_doc: &'static str,
    pub(super) runtime_07_archive: &'static str,
}

impl SplitLayoutSources {
    pub(super) fn load() -> Self {
        Self {
            parent: include_str!("../../ecs_extract_counters.rs"),
            query_change: include_str!("../query_change.rs"),
            extract_cache: include_str!("../extract_cache.rs"),
            asset_animation: include_str!("../asset_animation.rs"),
            frame_diagnostics: include_str!("../frame_diagnostics.rs"),
            split_layout: include_str!("../split_layout.rs"),
            split_layout_route: include_str!("route.rs"),
            split_layout_source_inventory: include_str!("source_inventory.rs"),
            split_layout_sources: include_str!("sources.rs"),
            split_layout_status_docs: include_str!("status_docs.rs"),
            source_inventory: include_str!(
                "../../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
            ),
            runtime_15_plan: include_str!(
                "../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
            ),
            runtime_index: include_str!(
                "../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
            ),
            runtime_07_plan: include_str!(
                "../../../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
            ),
            review_findings: include_str!(
                "../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
            ),
            structure_convention: include_str!(
                "../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
            ),
            module_doc: include_str!(
                "../../../../../../../../docs/zircon_runtime/structure/module-convention.md"
            ),
            hotspot_doc: include_str!(
                "../../../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md"
            ),
        runtime_07_archive: include_str!(
            "../../../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
        ),
        }
    }
}

pub(super) fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should retain ECS/extract counter split-layout anchor `{anchor}`"
        );
    }
}
