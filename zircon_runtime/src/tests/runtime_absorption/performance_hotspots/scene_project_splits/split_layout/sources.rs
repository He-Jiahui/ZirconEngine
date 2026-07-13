pub(super) struct SplitLayoutSources {
    pub(super) parent: &'static str,
    pub(super) scene_asset: &'static str,
    pub(super) project_io: &'static str,
    pub(super) dynamic_session_event: &'static str,
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
            parent: include_str!("../../scene_project_splits.rs"),
            scene_asset: include_str!("../scene_asset.rs"),
            project_io: include_str!("../project_io.rs"),
            dynamic_session_event: include_str!("../dynamic_session_event.rs"),
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
            "{label} should retain scene/project split anchor `{anchor}`"
        );
    }
}
