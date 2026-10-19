//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。通过源码文本核对父子路由、状态镜像和文件预算。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
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
            parent: include_str!("../performance/scene_project_splits.rs"),
            scene_asset: include_str!("../scene_project_splits/scene_asset.rs"),
            project_io: include_str!("../scene_project_splits/project_io.rs"),
            dynamic_session_event: include_str!("../scene_project_splits/dynamic_session_event.rs"),
            split_layout: include_str!("../scene_project_splits/split_layout.rs"),
            split_layout_route: include_str!("scene_project_splits_route.rs"),
            split_layout_source_inventory: include_str!("../../split_layout/scene_project_splits_source_inventory.rs"),
            split_layout_sources: include_str!("scene_project_splits_sources.rs"),
            split_layout_status_docs: include_str!("../../split_layout/scene_project_splits_status_docs.rs"),
            source_inventory: include_str!(
                "../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py"
            ),
            runtime_07_archive: history("docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"),
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
