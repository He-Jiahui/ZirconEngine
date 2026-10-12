//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。集中保存路径、锚点或预期清单；消费方负责读取实际源码。
use super::sources::{assert_contains_all, SplitLayoutSources};

pub(super) fn assert_scene_project_source_inventory(sources: &SplitLayoutSources) {
    assert_contains_all(
        "performance hotpath source inventory",
        sources.source_inventory,
        &[
            "RUNTIME_07_TEST_FILES = (",
            "zircon_runtime/src/tests/contracts/performance/scene_project_splits.rs",
            "zircon_runtime/src/tests/contracts/scene_project_splits/dynamic_session_event.rs",
            "zircon_runtime/src/tests/contracts/scene_project_splits/project_io.rs",
            "zircon_runtime/src/tests/contracts/scene_project_splits/scene_asset.rs",
            "zircon_runtime/src/tests/contracts/scene_project_splits/split_layout.rs",
            "zircon_runtime/src/tests/contracts/split_layout/scene_project_splits_route.rs",
            "zircon_runtime/src/tests/split_layout/scene_project_splits_source_inventory.rs",
            "zircon_runtime/src/tests/contracts/split_layout/scene_project_splits_sources.rs",
            "zircon_runtime/src/tests/split_layout/scene_project_splits_status_docs.rs",
            "zircon_runtime/src/tests/contracts/performance/owner_budget/sources/load.rs",
        ],
    );
}
