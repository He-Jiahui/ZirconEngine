use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 06 Plugin surface/lifecycle 镜像文档守卫",
        [
            "runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts",
            "plugin_surface_lifecycle_boundary",
            "standalone rustc 1/1",
            "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
        ],
    ),
    (
        "Runtime 07 Performance hotpath 镜像文档守卫",
        [
            "runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts",
            "performance_hotpath_boundary",
            "standalone rustc 6/6",
            "extract/ecs_query/performance profiling/FPS Cargo gates pending",
        ],
    ),
    (
        "Runtime 07 scene asset owner split",
        [
            "folder-backed `scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs`",
            "`SceneMobilityAsset`",
            "`management.rs` 432 行",
            "38-hotspot / runtime-other=14 state",
        ],
    ),
    (
        "Runtime 07 scene asset split-drift repair",
        [
            "删除拆分后遗留在 `zircon_runtime/src/asset/assets/scene/physics.rs` 的重复 `SceneMobilityAsset` 定义",
            "`scene/mod.rs` 是唯一 owner",
            "`SceneSpotLightAsset` 公开链",
            "`scene_asset` 与 Runtime 07 Cargo gates 继续 pending",
        ],
    ),
    (
        "Runtime 07 scene asset folder-split public-surface guard",
        [
            "runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner",
            "`SceneSpotLightAsset` 字段/导出链",
            "standalone `rustc --edition 2021 --test ...performance_hotspots.rs` 通过 6/6",
            "包级 `scene_asset` / Runtime 07 Cargo gates 仍待 active lanes 清空后补跑",
        ],
    ),
    (
        "Runtime 07 scene asset guard 纳入 performance_hotpath_boundary",
        [
            "`performance_hotpath_boundary.py`",
            "`hotspot_guard_anchor_count = 20`",
            "`missing_hotspot_guard_anchors = []`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspot_guard_anchor_count=20",
        ],
    ),
    (
        "Runtime 07 project_io folder split",
        [
            "`project_io/{camera,physics,post_process,references,script,transform}.rs`",
            "`runtime_07_project_io_folder_split_keeps_entry_and_converter_owners`",
            "`large_file_hotspot_count = 38`",
            "`runtime-other=14`",
        ],
    ),
    (
        "Runtime 07 owner-budget evidence drift resync",
        [
            "`large_file_ownership_gate`",
            "38 hotspots",
            "runtime-other=14",
            "`runtime_absorption::performance_hotspots`",
        ],
    ),
    (
        "Runtime 07 owner-budget 38-hotspot 漂移同步",
        [
            "`large_file_hotspot_count = 38`",
            "`runtime-framework-render=2`",
            "`runtime-other=14`",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 37-hotspot 漂移同步",
        [
            "`large_file_hotspot_count = 37`",
            "`runtime-other=13`",
            "`hotspot_guard_anchor_count = 20`",
            "standalone `status_output_tables.rs` 2/2",
        ],
    ),
    (
        "Runtime 07 owner-budget 37-hotspot 再同步",
        [
            "`large_file_hotspot_count = 37`",
            "`runtime-other=12`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=37",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 38-hotspot 回漂同步",
        [
            "`large_file_hotspot_count = 38`",
            "`runtime-framework-render=2`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=38",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 07 owner-budget 39-hotspot 漂移同步",
        [
            "`large_file_hotspot_count = 39`",
            "`runtime-framework-render=4`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspots=39",
            "standalone `performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 08 ECS 数据面镜像文档守卫",
        [
            "runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts",
            "ecs_kernel_data_boundary",
            "standalone rustc 1/1",
            "entity/observer/command/messages/change_tick/ecs Cargo gates pending",
        ],
    ),
    (
        "Runtime 08 First-stage event update guard",
        [
            "first_stage_updates_all_registered_event_channels",
            "event_message_anchors = 12/12",
            "runtime_08_guard_anchors = 18/18",
            "standalone ecs_kernel_data 1/1",
        ],
    ),
    (
        "Runtime 08 ECS 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 16",
            "missing_behavior_test_anchors = []",
            "runtime_08_guard_anchors = 18/18",
            "standalone ecs_kernel_data 1/1",
        ],
    ),
    (
        "Runtime 09 UI architecture 镜像文档守卫",
        [
            "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
            "ui_architecture_boundary",
            "standalone rustc 4/4",
            "ui/input/naming_boundary/layout/template Cargo gates pending",
        ],
    ),
];
