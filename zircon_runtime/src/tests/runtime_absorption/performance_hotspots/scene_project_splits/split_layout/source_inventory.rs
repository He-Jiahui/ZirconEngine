//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。集中保存路径、锚点或预期清单；消费方负责读取实际源码。
use super::sources::{assert_contains_all, SplitLayoutSources};

pub(super) fn assert_scene_project_source_inventory(sources: &SplitLayoutSources) {
    assert_contains_all(
        "performance hotpath source inventory",
        sources.source_inventory,
        &[
            "RUNTIME_07_TEST_FILES = (",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/dynamic_session_event.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/project_io.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/scene_asset.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/route.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/source_inventory.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/sources.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout/status_docs.rs",
            "zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/sources/load.rs",
        ],
    );
}
