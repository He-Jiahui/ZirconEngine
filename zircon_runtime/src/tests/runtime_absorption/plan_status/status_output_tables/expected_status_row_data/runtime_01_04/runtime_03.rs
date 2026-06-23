use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 03 Schedule/frame-loop 镜像文档守卫",
        &[
            "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts",
            "schedule_frame_loop_boundary",
            "standalone rustc 1/1",
            "ecs_schedule/time/session/schedule_parallel Cargo gates pending",
        ],
    ),
    (
        "Runtime 03 Schedule/frame-loop module-doc 镜像元数据",
        &[
            "frame schedule module-doc anchors 3/3",
            "guard/test files 8/8",
            "Runtime 03 guard anchors 14/14",
            "ecs_schedule/time/session/schedule_parallel Cargo gates pending",
        ],
    ),
    (
        "Runtime 03 Schedule/frame-loop 行为测试锚审计同步",
        &[
            "behavior_test_anchor_count = 13",
            "missing_behavior_test_anchors = []",
            "Runtime 03 guard anchors 14/14",
            "standalone schedule_frame_loop 1/1",
        ],
    ),
    (
        "Runtime 03 world bootstrap fixed-loop stage guard sync",
        &[
            "world_bootstraps_with_renderable_defaults",
            "SystemStage::ORDER",
            "FixedFirst",
            "FixedPostUpdate",
        ],
    ),
    (
        "Runtime 03 Schedule/frame-loop current audit recheck",
        &[
            "schedule_frame_loop_current_audit_static_passed_cargo_pending",
            "source files 18/18",
            "standalone `schedule_frame_loop.rs` 1/1",
            "ecs_schedule/time/session/schedule_parallel Cargo gates pending",
        ],
    ),
    (
        "Runtime 03 Schedule/frame-loop inventory split",
        &[
            "schedule_frame_loop_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "schedule_frame_loop_source_inventory.py",
            "schedule_frame_loop_anchor_inventory.py",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 03 Schedule/frame-loop markdown renderer split",
        &[
            "schedule_frame_loop_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "schedule_frame_loop_markdown.py",
            "schedule_frame_loop_boundary.py` now owns only audit read/missing-anchor/risk aggregation at 368 lines",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
];
