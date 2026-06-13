#[test]
fn runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation() {
    let runtime_01_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let tech_stack =
        include_str!("../../../../../../docs/engine-architecture/runtime-tech-stack.md");
    let text_doc = include_str!("../../../../../../docs/zircon_runtime/ui/text.md");
    let physics_options =
        include_str!("../../../../../../docs/zircon_plugins/physics-plugin-options.md");
    let editor_backlog = include_str!(
        "../../../../../../docs/editor-and-tooling/runtime-editor-only-dependency-backlog.md"
    );
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_01_plan),
        Some("in_progress"),
        "Runtime 01 should stay in progress until tech_stack/text_shaper/plugin validation closes"
    );

    for row_name in [
        "1.1 选型文档",
        "1.2 winit/notify 策略",
        "1.3 zr_vm 治理决策",
        "1.4 依赖守卫测试",
        "2.1 三层职责矩阵",
        "2.2 cosmic-text 决策",
        "2.3 fontdue 裁决",
        "3.1 物理选型 spike",
        "3.2 导出归档决策",
        "3.3 rfd/arboard 裁决",
    ] {
        let row = runtime_01_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 01 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 01 pending status row", row, &["Cargo", "待"]);
    }

    assert_contains_all(
        "Runtime 01 validation gate commands",
        runtime_01_plan,
        &[
            "cargo test -p zircon_runtime --lib tech_stack --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib extensions --locked",
            "cargo test -p zircon_runtime --lib text_shaper --locked -- --nocapture",
            "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked",
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index",
            "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
            "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
            "runtime_text_doc_records_three_layer_stack_and_cross_reference",
            "complex_text_backends_can_only_enter_through_ui_text_shaper",
            "fontdue_editor_retained_host_dependency_has_migration_owner",
            "physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned",
            "export_archive_policy_is_documented_without_manifest_container_dependency",
            "editor_only_dependency_candidates_have_editor_backlog_owner",
        ],
    );

    let runtime_01_index_row =
        runtime_index_row_for(runtime_index, "01-tech-stack-and-dependency-governance.md");
    assert_contains_all(
        "Runtime 01 index row",
        runtime_01_index_row,
        &[
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "tech_stack/text_shaper/plugin physics Cargo gates",
            "Cargo 待 active lanes 清空",
        ],
    );

    let runtime_01_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P10 |"))
        .expect("Runtime index should keep the P10 tech-stack completeness problem row");
    assert_contains_all(
        "Runtime index P10 row",
        runtime_01_problem_row,
        &[
            "physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned",
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "tech_stack/text_shaper/plugin physics Cargo gates",
        ],
    );

    assert_contains_all(
        "Runtime tech-stack authority",
        tech_stack,
        &[
            "runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index",
            "runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate",
            "zr_vm_path_dependency_gate_is_documented_with_version_pairing",
            "export_archive_policy_is_documented_without_manifest_container_dependency",
            "editor_only_dependency_candidates_have_editor_backlog_owner",
        ],
    );
    assert_contains_all(
        "Runtime UI text doc",
        text_doc,
        &[
            "Backend Responsibility Matrix",
            "runtime_text_doc_records_three_layer_stack_and_cross_reference",
            "text_shaper_stack_uses_current_heuristic_backend_until_font_backends_land",
        ],
    );
    assert_contains_all(
        "Runtime physics option doc",
        physics_options,
        &[
            "only executable V1 backend",
            "Jolt native backend",
            "physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned",
        ],
    );
    assert_contains_all(
        "Editor-only dependency backlog",
        editor_backlog,
        &[
            "editor_only_dependency_candidates_have_editor_backlog_owner",
            "fontdue_editor_retained_host_dependency_has_migration_owner",
            "rfd",
            "arboard",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 01 gate",
        review,
        &[
            "Runtime 01 Tech Stack Guard",
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "tech_stack/text_shaper/plugin physics",
        ],
    );
}

#[test]
fn runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation() {
    let runtime_02_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let root_surface_doc =
        include_str!("../../../../../../docs/zircon_runtime/core/root_surface.md");
    let generated_boundary_doc =
        include_str!("../../../../../../docs/engine-architecture/generated-code-boundary.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );
    let runtime_05_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );

    assert_eq!(
        frontmatter_status(runtime_02_plan),
        Some("in_progress"),
        "Runtime 02 should stay in progress until core/root/generated validation and render-owner alias cutover close"
    );

    for (row_name, required_anchors) in [
        ("M2 | 测试阶段", &["进行中", "Cargo", "render/graphics"][..]),
        (
            "M2 | P2 总表状态复核",
            &["M2 测试阶段仍保持", "Cargo", "render owner"][..],
        ),
        (
            "M3 | 3.1 模块名别名清除",
            &[
                "pre_m3_root_surface_guard_static_passed_pending_render_owner",
                "Cargo/rustc",
                "render owner",
            ][..],
        ),
        (
            "M3 | 3.2 类型别名清除",
            &[
                "pre_m3_type_alias_guard_static_passed_pending_render_owner",
                "actual type alias deletion not executed",
                "render owner",
            ][..],
        ),
        (
            "M4 | 4.2 行为迁回与守卫",
            &[
                "代码完成，Cargo 测试待重跑",
                "generated",
                "export_build_plan",
            ][..],
        ),
    ] {
        let row = runtime_02_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 02 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 02 pending status row", row, required_anchors);
    }

    assert_contains_all(
        "Runtime 02 validation gate commands",
        runtime_02_plan,
        &[
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib core:: --locked",
            "cargo test -p zircon_runtime --lib runtime_absorption --locked -- --nocapture",
            "cargo test -p zircon_app --locked",
            "cargo check -p zircon_editor --lib --locked",
            "cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked",
            "cargo test -p zircon_runtime --lib generated --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib export_build_plan --locked",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
        ],
    );

    let runtime_02_index_row =
        runtime_index_row_for(runtime_index, "02-core-spine-and-root-surface.md");
    assert_contains_all(
        "Runtime 02 index row",
        runtime_02_index_row,
        &[
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "M2/M4 全量 Cargo 回归",
            "M3 lib.rs graphics alias 清理需等待 render owner",
        ],
    );

    let runtime_02_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P2 |"))
        .expect("Runtime index should keep the P2 core spine/root surface problem row");
    assert_contains_all(
        "Runtime index P2 row",
        runtime_02_problem_row,
        &[
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "pre_m3_type_alias_guard_static_passed_pending_render_owner",
            "Cargo default/lib-test",
        ],
    );

    let runtime_02_generated_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P8 |"))
        .expect("Runtime index should keep the P8 generated-code problem row");
    assert_contains_all(
        "Runtime index P8 row",
        runtime_02_generated_problem_row,
        &[
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "m1_gate_status=classified-and-clear",
            "Cargo 测试阶段仍受",
        ],
    );

    assert_contains_all(
        "Runtime root surface doc",
        root_surface_doc,
        &[
            "pre_m3_type_alias_guard_static_passed_pending_render_owner",
            "crate-private type alias debt",
            "render owner window",
        ],
    );

    assert_contains_all(
        "Runtime generated boundary doc",
        generated_boundary_doc,
        &[
            "classified-and-clear",
            "generated_code_boundary.m1_gate_status",
            "migration_debt_location_count = 0",
        ],
    );

    assert_contains_all(
        "Runtime 05 closeout plan",
        runtime_05_plan,
        &[
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "Runtime 02 `core/root/generated/export_build_plan/app/editor/plugin` gate",
        ],
    );

    assert_contains_all(
        "Runtime architecture review Runtime 02 gate",
        review,
        &[
            "Runtime 02 Core/Root/Generated Gate",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "core/root/generated/export_build_plan/app/editor/plugin",
        ],
    );
}

#[test]
fn runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation() {
    let runtime_03_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let frame_schedule_doc =
        include_str!("../../../../../../docs/zircon_runtime/core/frame_schedule.md");
    let schedule_parallel_doc = include_str!(
        "../../../../../../docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md"
    );
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_03_plan),
        Some("in_progress"),
        "Runtime 03 should stay in progress until schedule/frame-loop Cargo validation closes"
    );

    for row_name in [
        "1.1 隐式顺序显式化",
        "1.2 UI extract 合法旁路契约",
        "2.1 单次 `RuntimeTimeAdvance` 接通",
        "2.2 插值因子",
        "3.1 开关与计数",
        "3.2 一致性与收益",
    ] {
        let row = runtime_03_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 03 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 03 pending status row", row, &["Cargo", "待"]);
    }

    assert_contains_all(
        "Runtime 03 validation gate commands",
        runtime_03_plan,
        &[
            "cargo test -p zircon_runtime --lib ecs_schedule --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib session --locked",
            "cargo test -p zircon_app --locked",
            "cargo test -p zircon_runtime --lib fixed_update --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib time --locked",
            "cargo test -p zircon_runtime --lib schedule_parallel --locked -- --nocapture",
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration",
            "session_ui_extract_remains_documented_dynamic_session_side_path",
            "world_driver_consumes_runtime_time_advance_without_advancing_clocks_again",
            "fixed_step_plan_reports_overstep_fraction_in_unit_range",
            "schedule_parallel_executor_can_run_parallel_batches_serially_with_report",
            "schedule_parallel_execution_report_records_diagnostic_counts",
            "representative_schedule_produces_multi_system_parallel_batches",
            "parallel_and_serial_execution_reach_identical_world_state",
        ],
    );

    let runtime_03_index_row =
        runtime_index_row_for(runtime_index, "03-schedule-and-frame-loop-alignment.md");
    assert_contains_all(
        "Runtime 03 index row",
        runtime_03_index_row,
        &[
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "ecs_schedule/time/session/schedule_parallel Cargo gates",
            "Cargo 待 active lanes 清空",
        ],
    );

    let runtime_03_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P3 |"))
        .expect("Runtime index should keep the P3 schedule/frame-loop problem row");
    assert_contains_all(
        "Runtime index P3 row",
        runtime_03_problem_row,
        &[
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "ecs_schedule/time/session/schedule_parallel",
            "Cargo 回归待运行",
        ],
    );

    assert_contains_all(
        "Runtime frame schedule doc",
        frame_schedule_doc,
        &[
            "Runtime Frame Schedule",
            "session_ui_extract_remains_documented_dynamic_session_side_path",
            "WorldDriver",
            "RuntimeTimeAdvance",
            "fixed_step_plan_reports_overstep_fraction_in_unit_range",
            "schedule.parallel_batches",
        ],
    );
    assert_contains_all(
        "Runtime schedule parallel executor doc",
        schedule_parallel_doc,
        &[
            "ScheduleParallelExecutionReport",
            "schedule_parallel_execution_report_records_diagnostic_counts",
            "parallel_and_serial_execution_reach_identical_world_state",
            "schedule_parallel_batches_chain_through_job_handles",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 03 gate",
        review,
        &[
            "Runtime 03 Schedule Frame-Loop Guard",
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "ecs_schedule/time/session/schedule_parallel",
        ],
    );
}

#[test]
fn runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation() {
    let runtime_04_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let asset_facade_doc = include_str!("../../../../../../docs/zircon_runtime/asset/facade.md");
    let asset_worker_doc =
        include_str!("../../../../../../docs/zircon_runtime/asset/worker_pool.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_04_plan),
        Some("in_progress"),
        "Runtime 04 should stay in progress until broader asset validation closes"
    );

    for row_name in [
        "1.1 句柄语义裁决",
        "1.2 转移表测试",
        "2.1 背压策略",
        "2.2 请求去重",
        "2.3 诊断计数",
        "worker pool 当前状态守卫",
    ] {
        let row = runtime_04_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 04 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 04 pending status row", row, &["Cargo", "待"]);
    }

    assert_contains_all(
        "Runtime 04 validation gate commands",
        runtime_04_plan,
        &[
            "cargo test -p zircon_runtime --lib load_state --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib resource --locked",
            "cargo test -p zircon_runtime --lib asset:: --locked",
            "cargo test -p zircon_runtime --lib worker_pool --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib watch --locked -- --nocapture",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
        ],
    );

    let runtime_04_index_row =
        runtime_index_row_for(runtime_index, "04-asset-pipeline-alignment.md");
    assert_contains_all(
        "Runtime 04 index row",
        runtime_04_index_row,
        &[
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "broader `asset::` / `worker_pool` Cargo filters",
            "Cargo 待",
        ],
    );

    let runtime_04_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P7 |"))
        .expect("Runtime index should keep the P7 asset pipeline problem row");
    assert_contains_all(
        "Runtime index P7 row",
        runtime_04_problem_row,
        &[
            "asset_worker_pool_matches_runtime_04_and_11_decisions",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "broader `asset::` / `worker_pool` Cargo filters",
        ],
    );

    assert_contains_all(
        "Runtime asset facade doc",
        asset_facade_doc,
        &[
            "Reference Asset Stack Gap Table",
            "dangling_handle_queries_report_not_loaded_instead_of_panicking",
            "failed_asset_exposes_failure_reason_through_facade",
        ],
    );
    assert_contains_all(
        "Runtime asset worker doc",
        asset_worker_doc,
        &[
            "AssetWorkerPoolOptions",
            "Backpressure",
            "Request De-Duplication",
            "asset.worker.budgeted_threads",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 04 gate",
        review,
        &[
            "Runtime 04 Asset Pipeline Guard",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "asset:: / worker_pool",
        ],
    );
}

#[test]
fn runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation() {
    let runtime_06_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let native_boundary_doc =
        include_str!("../../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let runtime_interface_doc =
        include_str!("../../../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let runtime_05_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_06_plan),
        Some("in_progress"),
        "Runtime 06 should stay in progress until script VM and plugin/native validation closes"
    );

    for (row_name, required_anchors) in [
        (
            "1.1 空参数修复",
            &[
                "代码完成，runtime Cargo 待验证",
                "call_module_export_accepts_empty_argument_slice",
                "sentinel pointer",
            ][..],
        ),
        (
            "1.2 失败路径测试",
            &[
                "部分完成，runtime Cargo 待验证",
                "300s 编译超时",
                "runtime real-backend/fallback",
            ][..],
        ),
        ("2.1 native 收口", &["待开始"][..]),
        ("2.2 测试/文档迁移", &["待开始"][..]),
        ("3.1 V1/V2 处置", &["待开始"][..]),
        ("3.2 回滚失败注入", &["待开始"][..]),
    ] {
        let row = runtime_06_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 06 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 06 pending status row", row, required_anchors);
    }

    assert_contains_all(
        "Runtime 06 validation gate commands",
        runtime_06_plan,
        &[
            "cargo test -p zircon_runtime --lib script::vm --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib vampire_project_session --features zr-vm-real-backend --locked -- --nocapture --test-threads=1",
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib plugin --locked -- --nocapture",
            "cargo test -p zircon_app --locked",
            "cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked",
            "cargo test -p zircon_runtime --lib native_plugin --locked -- --nocapture",
            "cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked",
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
        ],
    );

    let runtime_06_index_row =
        runtime_index_row_for(runtime_index, "06-plugin-surface-and-lifecycle.md");
    assert_contains_all(
        "Runtime 06 index row",
        runtime_06_index_row,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
            "runtime 真实后端验证待重跑",
        ],
    );

    let runtime_06_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P4 |"))
        .expect("Runtime index should keep the P4 plugin surface problem row");
    assert_contains_all(
        "Runtime index P4 row",
        runtime_06_problem_row,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "native_plugin_public_surface.m4_gate_status=migration-debt-present",
            "root_reexport_count = 68",
        ],
    );

    assert_contains_all(
        "Native plugin boundary doc",
        native_boundary_doc,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "m4_gate_status",
            "migration-debt-present",
            "root_reexport_count = 68",
        ],
    );

    assert_contains_all(
        "Runtime interface convergence doc",
        runtime_interface_doc,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "native_plugin_public_surface",
            "migration-debt-present",
            "root-level native loader/ABI re-export symbols",
        ],
    );

    assert_contains_all(
        "Runtime 05 closeout plan",
        runtime_05_plan,
        &[
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "Runtime 06 `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` gate",
        ],
    );

    assert_contains_all(
        "Runtime architecture review Runtime 06 gate",
        review,
        &[
            "Runtime 06 Plugin Surface Lifecycle Gate",
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
        ],
    );
}

#[test]
fn runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation() {
    let runtime_07_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let ecs_doc = include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_07_plan),
        Some("in_progress"),
        "Runtime 07 should stay in progress until performance Cargo/profiling/FPS validation closes"
    );

    for (row_name, status_anchor) in [
        ("0.3 帧分解 span", "frame_spans_static_passed_trace_pending"),
        (
            "1.1 计数点",
            "scoped_counter_points_extract_implemented_cargo_blocked",
        ),
        (
            "1.2 计数断言",
            "named_assertions_static_passed_cargo_blocked",
        ),
        (
            "1.3 热点清单",
            "inventory_scaffold_static_passed_pending_authoritative_values",
        ),
    ] {
        let row = runtime_07_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 07 should keep status row `{row_name}`"));
        assert_contains_all(
            "Runtime 07 pending status row",
            row,
            &[status_anchor, "Cargo"],
        );
        assert!(
            !row.contains("completed |"),
            "Runtime 07 row `{row_name}` must not claim completed before performance validation closes"
        );
    }

    assert_contains_all(
        "Runtime 07 validation gate commands",
        runtime_07_plan,
        &[
            "cargo test -p zircon_runtime --lib vampire_project_session_reports_runtime_fps_and_render_work --features zr-vm-real-backend --locked -- --nocapture --test-threads=1",
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib extract --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib ecs_query --locked -- --nocapture",
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
            "query_state_reuses_archetype_matches_across_unchanged_frames",
            "change_detection_scan_skips_unmarked_archetypes",
            "frame_extract_rebuild_skips_unchanged_entities",
            "runtime_frame_schedule_stage.<SystemStage>",
        ],
    );

    let runtime_07_index_row =
        runtime_index_row_for(runtime_index, "07-runtime-performance-hotpath.md");
    assert_contains_all(
        "Runtime 07 index row",
        runtime_07_index_row,
        &[
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "extract/ecs_query/performance profiling/FPS gates",
            "Cargo/profiling/FPS 待",
        ],
    );

    let runtime_07_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P5 |"))
        .expect("Runtime index should keep the P5 runtime performance problem row");
    assert_contains_all(
        "Runtime index P5 row",
        runtime_07_problem_row,
        &[
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "profiling 构建",
            "runtime 真实后端验证",
        ],
    );

    assert_contains_all(
        "Runtime 07 hotspot inventory doc",
        hotspot_doc,
        &[
            "Evidence Gate",
            "No Runtime 07 M2 optimization slice may start from an unmeasured suspicion",
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "extract/ecs_query/performance profiling/FPS gates",
        ],
    );
    assert_contains_all(
        "Runtime dynamic session frame diagnostics doc",
        dynamic_session_doc,
        &[
            "runtime_frame_time_update",
            "runtime_frame_extract",
            "runtime_frame_submit",
            "runtime_frame_schedule_stage.<SystemStage>",
            "frame_extract_rebuild_skips_unchanged_entities",
        ],
    );
    assert_contains_all(
        "Runtime ECS profiling doc",
        ecs_doc,
        &[
            "SceneScheduleRunner::run_stage(...)",
            "runtime_frame_schedule_stage.<SystemStage>",
            "runtime_07_hotspot_inventory_requires_counted_evidence_before_m2",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 07 gate",
        review,
        &[
            "Runtime 07 Performance Hotpath Guard",
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "extract/ecs_query/performance profiling/FPS gates",
        ],
    );
}

#[test]
fn runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation() {
    let runtime_08_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let ecs_doc = include_str!("../../../../../../docs/zircon_runtime/scene/ecs.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_08_plan),
        Some("in_progress"),
        "Runtime 08 should stay in progress until ECS data-kernel validation closes"
    );

    for row_name in [
        "1.1 生命周期测试矩阵",
        "2.1 观察者时序",
        "2.2 命令队列错误路径",
        "3.1 双通道定稿",
        "3.2 tick 回绕",
    ] {
        let row = runtime_08_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 08 should keep status row `{row_name}`"));
        assert_contains_all(
            "Runtime 08 pending status row",
            row,
            &["code_complete_pending_cargo", "Cargo"],
        );
    }

    assert_contains_all(
        "Runtime 08 validation gate commands",
        runtime_08_plan,
        &[
            "cargo test -p zircon_runtime --lib entity --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib observer --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib command --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib change_tick --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib messages --locked",
            "cargo test -p zircon_runtime --lib ecs --locked",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
        ],
    );

    let runtime_08_index_row =
        runtime_index_row_for(runtime_index, "08-ecs-kernel-data-alignment.md");
    assert_contains_all(
        "Runtime 08 index row",
        runtime_08_index_row,
        &[
            "Runtime 08 ECS 数据面守卫",
            "entity/observer/command/messages/change_tick/ecs filters",
            "Cargo 待活动 lanes 清空后运行",
        ],
    );

    let runtime_08_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P11 |"))
        .expect("Runtime index should keep the P11 ECS data-kernel problem row");
    assert_contains_all(
        "Runtime index P11 row",
        runtime_08_problem_row,
        &[
            "测试已落地待 Cargo",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
        ],
    );

    assert_contains_all(
        "Runtime ECS module doc",
        ecs_doc,
        &[
            "Runtime 08 Data-Kernel Alignment Verdict",
            "despawned_entity_handle_is_rejected_by_world_access",
            "lifecycle_observer_fires_immediately_during_component_mutation",
            "command_queue_on_despawned_entity_target_is_reported_not_silently_dropped",
            "events_require_explicit_update_and_keep_next_queue_hidden",
            "change_tick_comparison_survives_wraparound",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 08 gate",
        review,
        &[
            "Runtime 08 ECS Data-Kernel Guard",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
            "entity/observer/command/messages/change_tick/ecs",
        ],
    );
}
