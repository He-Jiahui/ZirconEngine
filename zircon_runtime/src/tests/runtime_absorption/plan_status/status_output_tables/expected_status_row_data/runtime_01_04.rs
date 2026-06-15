use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 01 Tech-stack 镜像文档守卫",
        [
            "runtime_01_tech_stack_mirror_docs_match_structure_audit_counts",
            "tech_stack_boundary",
            "standalone rustc 1/1",
            "tech_stack/extensions/text_shaper/plugin physics Cargo gates pending",
        ],
    ),
    (
        "Runtime 01 Tech-stack 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 4",
            "missing_behavior_test_anchors = []",
            "standalone tech_stack 1/1",
            "tech_stack/extensions/text_shaper/plugin physics Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 core/root/generated 镜像文档守卫",
        [
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
            "core_spine_root_generated_boundary",
            "standalone rustc 1/1",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 02 generated template count 审计同步",
        [
            "`template_file_count=10`",
            "generated export templates 10/10",
            "0 migration debt",
            "stale 9/9 scan",
        ],
    ),
    (
        "Runtime 02 guard-test anchors 审计同步",
        [
            "guard_test_anchor_count = 21",
            "missing_guard_test_anchors = []",
            "standalone core_spine_root_generated 1/1",
            "core/root/generated/export_build_plan/app/editor/plugin Cargo gates pending",
        ],
    ),
    (
        "Runtime 03 Schedule/frame-loop 镜像文档守卫",
        [
            "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts",
            "schedule_frame_loop_boundary",
            "standalone rustc 1/1",
            "ecs_schedule/time/session/schedule_parallel Cargo gates pending",
        ],
    ),
    (
        "Runtime 03 Schedule/frame-loop module-doc 镜像元数据",
        [
            "frame schedule module-doc anchors 3/3",
            "guard/test files 8/8",
            "Runtime 03 guard anchors 14/14",
            "ecs_schedule/time/session/schedule_parallel Cargo gates pending",
        ],
    ),
    (
        "Runtime 03 Schedule/frame-loop 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 13",
            "missing_behavior_test_anchors = []",
            "Runtime 03 guard anchors 14/14",
            "standalone schedule_frame_loop 1/1",
        ],
    ),
    (
        "Runtime 03 world bootstrap fixed-loop stage guard sync",
        [
            "world_bootstraps_with_renderable_defaults",
            "SystemStage::ORDER",
            "FixedFirst",
            "FixedPostUpdate",
        ],
    ),
    (
        "Runtime 04 Asset pipeline 镜像文档守卫",
        [
            "runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts",
            "asset_pipeline_boundary",
            "standalone rustc 1/1",
            "broader asset::/worker_pool Cargo gates pending",
        ],
    ),
    (
        "Runtime 04 Asset pipeline 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 18",
            "missing_behavior_test_anchors = []",
            "standalone asset_pipeline 1/1",
            "broader asset::/worker_pool Cargo gates pending",
        ],
    ),
];
