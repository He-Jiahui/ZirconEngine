use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 07 scene asset owner split",
        &[
            "folder-backed `scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs`",
            "`SceneMobilityAsset`",
            "`management.rs` 432 行",
            "38-hotspot / runtime-other=14 state",
        ],
    ),
    (
        "Runtime 07 scene asset split-drift repair",
        &[
            "删除拆分后遗留在 `zircon_runtime/src/asset/assets/scene/physics.rs` 的重复 `SceneMobilityAsset` 定义",
            "`scene/mod.rs` 是唯一 owner",
            "`SceneSpotLightAsset` 公开链",
            "`scene_asset` 与 Runtime 07 Cargo gates 继续 pending",
        ],
    ),
    (
        "Runtime 07 scene asset folder-split public-surface guard",
        &[
            "runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner",
            "`SceneSpotLightAsset` 字段/导出链",
            "standalone `rustc --edition 2021 --test ...performance_hotspots.rs` 通过 6/6",
            "包级 `scene_asset` / Runtime 07 Cargo gates 仍待 active lanes 清空后补跑",
        ],
    ),
    (
        "Runtime 07 scene asset guard 纳入 performance_hotpath_boundary",
        &[
            "`performance_hotpath_boundary.py`",
            "`hotspot_guard_anchor_count = 20`",
            "`missing_hotspot_guard_anchors = []`",
            "direct `performance_hotpath_boundary_audit` risks=0 / hotspot_guard_anchor_count=20",
        ],
    ),
    (
        "Runtime 07 project_io folder split",
        &[
            "`project_io/{camera,physics,post_process,references,script,transform}.rs`",
            "`runtime_07_project_io_folder_split_keeps_entry_and_converter_owners`",
            "`large_file_hotspot_count = 38`",
            "`runtime-other=14`",
        ],
    ),
];
