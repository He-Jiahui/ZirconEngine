use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 recent-static Runtime 02/07 status metadata guard",
        [
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "runtime_02_generated_status_guard_present = true",
            "runtime_07_owner_budget_status_guard_present = true",
            "standalone recent_static 1/1",
        ],
    ),
    (
        "Runtime 05 status-output recent-static metadata row",
        [
            "Runtime 05 recent-static Runtime 02/07 status metadata guard",
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "runtime_02_generated_status_guard_present = true",
            "standalone recent_static 1/1",
        ],
    ),
    (
        "Runtime 05 non-network server UI sortingMode allowlist",
        [
            "sortingMode = \"server\"",
            "allowed_context_count 99",
            "unclassified_location_count 0",
            "aggregate `audit_runtime_structure.py --json` non-network assertions",
        ],
    ),
    (
        "Runtime 05 status-output non-network server allowlist row",
        [
            "Runtime 05 non-network server UI sortingMode allowlist",
            "sortingMode = \"server\"",
            "allowed_context_count 99",
            "unclassified_location_count 0",
        ],
    ),
    (
        "Runtime 05 naming_boundary non-network server Rust guard",
        [
            "runtime_non_network_server_naming_is_classified_by_owner",
            "runtime-ui-component-catalog-editor-controls",
            "standalone naming_boundary 2/2",
            "sortingMode = \"server\"",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 12 gamepad event-owner row",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 12 gamepad event-owner 漂移同步",
            "missing_gamepad_abi_anchors = []",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 12 behavior-test row",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 12 Input stack 行为测试锚审计同步",
            "behavior_test_anchor_count = 6",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 04 behavior-test row",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 04 Asset pipeline 行为测试锚审计同步",
            "behavior_test_anchor_count = 18",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 08 behavior-test row",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 08 ECS 行为测试锚审计同步",
            "behavior_test_anchor_count = 16",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 10 behavior-test row",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 10 Dynamic API 行为测试锚审计同步",
            "behavior_test_anchor_count = 15",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 plan-status output-anchor module split",
        [
            "runtime_plan_status_output_anchors.py",
            "runtime_plan_status_boundary.py` remains orchestration",
            "761 lines",
            "direct `runtime_plan_status_boundary_audit` risks=0",
        ],
    ),
    (
        "Runtime 05 plan-status output-anchor budget guard",
        [
            "plan_status_boundary_line_count = 855",
            "max_plan_status_boundary_lines = 900",
            "status_output_anchor_module_present = true",
            "status_output_anchor_module_line_count = 143",
        ],
    ),
    (
        "Runtime 05 status-output status/date helper split",
        [
            "status_output_tables/expected_slices.rs",
            "expected_status_for_slice",
            "expected_date_for_slice",
            "status_output_tables.rs",
        ],
    ),
    (
        "Runtime 05 status-output expected anchor split",
        [
            "status_output_tables/expected_status_rows.rs",
            "EXPECTED_STATUS_OUTPUT_SLICES",
            "status_output_tables.rs",
            "plan-status support files 21/21",
        ],
    ),
    (
        "Runtime 05 plan-status root module split",
        [
            "plan_status/support.rs",
            "plan_status/index_tables.rs",
            "plan_status.rs",
            "plan-status support files 26/26",
        ],
    ),
    (
        "Runtime 05 plan-status support inventory split",
        [
            "runtime_plan_status_support_inventory.py",
            "PLAN_STATUS_SUPPORT_FILES",
            "plan_status_boundary_line_count = 842",
            "support 26/26",
        ],
    ),
    (
        "Runtime 05 plan-status anchor inventory split",
        [
            "runtime_plan_status_anchor_inventory.py",
            "CORE_GUARD_ANCHORS",
            "PENDING_GATE_ANCHORS",
            "plan_status_boundary_line_count = 789",
        ],
    ),
    (
        "Runtime 05 plan-status markdown renderer split",
        [
            "runtime_plan_status_markdown.py",
            "render_runtime_plan_status_boundary_markdown",
            "plan_status_boundary_line_count = 559",
            "support 26/26",
        ],
    ),
    (
        "Runtime 05 plan-status source helper split",
        [
            "runtime_plan_status_sources.py",
            "runtime_subplans",
            "status_rows",
            "plan_status_boundary_line_count = 454",
        ],
    ),
    (
        "Runtime 05 status-output expected row data split",
        [
            "expected_status_row_data.rs",
            "EXPECTED_STATUS_OUTPUT_SLICES",
            "expected_status_rows.rs",
            "plan-status support files 27/27",
        ],
    ),
    (
        "Runtime 05 status-output row-data group split",
        [
            "EXPECTED_STATUS_OUTPUT_SLICE_GROUPS",
            "expected_status_output_slices",
            "expected_status_row_data/runtime_05.rs",
            "plan-status support files 32/32",
        ],
    ),
    (
        "Runtime 05 full scene closeout failed evidence",
        [
            "cargo test -p zircon_runtime --lib scene:: --locked",
            "880 passed",
            "31 failed",
            "pending_full_scene_cargo",
        ],
    ),
    (
        "Runtime 05 full scene closeout no-result recheck",
        [
            "zircon_runtime_scene_closeout_20260615_1806.log",
            "SCENE_CLOSEOUT_EXIT=-1",
            "无测试结果",
            "zircon-editor-ui-command-registry-0615",
        ],
    ),
    (
        "Runtime 05 scene:: failure support-first triage",
        [
            "runtime_05_full_scene_failure_clusters_keep_support_first_triage_visible",
            "graphics-scene-lower-layer-candidate",
            "scene-asset-project-io-lower-layer-candidate",
            "ecs-scene-lower-layer-candidate",
        ],
    ),
    (
        "Runtime 05 serialization source folder-split guard sync",
        [
            "scene_project_serialization_sources_do_not_store_editor_authoring_state",
            "src/asset/assets/scene/mod.rs",
            "src/scene/world/project_io/{camera,physics,post_process,references,script,transform}.rs",
            "SOURCE_AUTHORING_TOKENS",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 03 split",
        [
            "cargo_gates/early/runtime_03.rs",
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 01 split",
        [
            "cargo_gates/early/runtime_01.rs",
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 02 split",
        [
            "cargo_gates/early/runtime_02.rs",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 04 split",
        [
            "cargo_gates/early/runtime_04.rs",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "early.rs",
            "plan-status support files 20/20",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 06 split",
        [
            "cargo_gates/early/runtime_06.rs",
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "early.rs",
            "plan-status support files 20/20",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 08 split",
        [
            "cargo_gates/early/runtime_08.rs",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 07 split",
        [
            "cargo_gates/early/runtime_07.rs",
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates late Runtime 10 split",
        [
            "cargo_gates/late/runtime_10.rs",
            "runtime_10_m1_3_cargo_pending_gate_stays_explicit_until_dynamic_api_validation",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates late Runtime 11 split",
        [
            "cargo_gates/late/runtime_11.rs",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "late.rs",
            "plan-status support files 15/15",
        ],
    ),
    (
        "Runtime 05 cargo-gates late Runtime 12 split",
        [
            "cargo_gates/late/runtime_12.rs",
            "runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation",
            "late.rs",
            "plan-status support files 16/16",
        ],
    ),
    (
        "Runtime 05 cargo-gates late Runtime 13 split",
        [
            "cargo_gates/late/runtime_13.rs",
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "late.rs",
            "plan-status support files 18/18",
        ],
    ),
    (
        "Runtime 05 cargo-gates late Runtime 14 split",
        [
            "cargo_gates/late/runtime_14.rs",
            "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
            "late.rs",
            "plan-status support files 18/18",
        ],
    ),
    (
        "Runtime 05 plan-status 输出表守卫",
        [
            "runtime_plan_status_output_tables_cover_index_and_all_subplans",
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "status_table_gaps []",
            "full scene:: Cargo gate 仍 pending",
        ],
    ),
    (
        "Runtime 05 plan-status 审计元数据守卫",
        [
            "status_output_table_guard_count = 4",
            "missing_status_output_table_guard_anchors = []",
            "all runtime index status rows",
            "full coverage guard",
        ],
    ),
    (
        "Runtime 05 M0 absorption guard coverage sync",
        [
            "runtime_architecture_review_documents_all_absorption_guards",
            "25 个 mounted",
            "runtime_absorption/ecs_kernel_data.rs",
            "runtime_absorption/script_binding.rs",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 07 scene asset rows",
        [
            "Runtime 07 scene asset owner split",
            "Runtime 07 scene asset split-drift repair",
            "hotspot_guard_anchor_count = 20",
            "`scene_asset` / Runtime 07 Cargo gates pending",
        ],
    ),
    (
        "Runtime 05 Runtime 07 scene status 审计元数据",
        [
            "runtime_07_scene_status_index_anchor_count = 11",
            "runtime_07_scene_status_guard_anchor_count = 10",
            "runtime_07_scene_status_guard_present = true",
            "index 11/11",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 02 generated template row",
        [
            "Runtime 02 generated template count 审计同步",
            "template_file_count=10",
            "generated export templates 10/10",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 Runtime 02 generated status 审计元数据",
        [
            "runtime_02_generated_status_index_anchor_count = 6",
            "runtime_02_generated_status_guard_anchor_count = 5",
            "runtime_02_generated_status_guard_present = true",
            "index 6/6",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 07 owner-budget row",
        [
            "Runtime 07 owner-budget 39-hotspot 漂移同步",
            "large_file_hotspot_count = 39",
            "runtime-framework-render=4",
            "`performance_hotspots.rs` 6/6",
        ],
    ),
    (
        "Runtime 05 Runtime 07 owner-budget status 审计元数据",
        [
            "runtime_07_owner_budget_status_index_anchor_count = 6",
            "runtime_07_owner_budget_status_guard_anchor_count = 5",
            "runtime_07_owner_budget_status_guard_present = true",
            "index 6/6",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 03 module-doc row",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 03 Schedule/frame-loop module-doc 镜像元数据",
            "frame schedule module-doc anchors 3/3",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 03 behavior-test row",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "Runtime 03 Schedule/frame-loop 行为测试锚审计同步",
            "behavior_test_anchor_count = 13",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 05 status-output all-index-row coverage guard",
        [
            "runtime_index_status_output_records_recent_cross_plan_slices",
            "all runtime index status rows",
            "standalone status-output 2/2",
            "full coverage guard",
        ],
    ),
    (
        "Runtime 05 plan-status Cargo attempt 状态审计",
        [
            "runtime_plan_status_boundary",
            "cargo_attempt_status_anchor_count = 20",
            "cargo_attempt_status_guard_present = true",
            "Runtime 14 animation Cargo gate 尝试",
        ],
    ),
    (
        "Runtime 05 plan-status Cargo timeout 状态审计",
        [
            "runtime_plan_status_boundary",
            "cargo_recheck_timeout_no_result",
            "Runtime 14 animation runtime-status focused recheck timeout",
            "runtime_status_reports_player_rig_and_gpu_readiness",
        ],
    ),
];
