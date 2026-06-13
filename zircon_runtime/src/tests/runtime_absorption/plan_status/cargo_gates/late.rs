#[test]
fn runtime_10_m1_3_cargo_pending_gate_stays_explicit_until_dynamic_api_validation() {
    let runtime_10_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let dynamic_api_doc =
        include_str!("../../../../../../docs/zircon_runtime/dynamic_api/session.md");

    assert_eq!(
        frontmatter_status(runtime_10_plan),
        Some("in_progress"),
        "Runtime 10 should stay in progress until dynamic_api and loader validation close"
    );

    let m1_3_row = runtime_10_plan
        .lines()
        .find(|line| line.contains("| M1 | 1.3 FFI panic 边界 |"))
        .expect("Runtime 10 should keep the M1.3 FFI panic boundary status row");
    assert_contains_all(
        "Runtime 10 M1.3 status row",
        m1_3_row,
        &[
            "code_static_passed_cargo_pending",
            "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
            "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
            "Cargo 待当前 runtime 编译通道空闲后补跑 `dynamic_api`",
        ],
    );
    for forbidden_claim in ["completed", "cargo_passed", "`dynamic_api` 通过"] {
        assert!(
            !m1_3_row.contains(forbidden_claim),
            "Runtime 10 M1.3 row must not claim `{forbidden_claim}` before dynamic_api Cargo validation passes"
        );
    }

    assert_contains_all(
        "Runtime 10 M1 testing stage",
        runtime_10_plan,
        &["cargo test -p zircon_runtime --lib dynamic_api --locked -- --nocapture"],
    );

    let runtime_10_index_row =
        runtime_index_row_for(runtime_index, "10-dynamic-api-and-interface-convergence.md");
    assert_contains_all(
        "Runtime 10 index row",
        runtime_10_index_row,
        &[
            "M1.3 FFI panic 边界已静态落地",
            "M1.3 rustfmt/锚点/差异检查通过",
            "Cargo 待空闲通道",
        ],
    );

    let runtime_10_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P13 |"))
        .expect("Runtime index should keep the P13 dynamic API problem row");
    assert_contains_all(
        "Runtime index P13 row",
        runtime_10_problem_row,
        &[
            "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
            "panic-boundary Cargo 验证仍待",
        ],
    );

    assert_contains_all(
        "Dynamic API module doc",
        dynamic_api_doc,
        &[
            "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
            "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
            "cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 --message-format short",
        ],
    );
}

#[test]
fn runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff() {
    let runtime_10_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_05_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let convergence_doc =
        include_str!("../../../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_10_plan),
        Some("in_progress"),
        "Runtime 10 should stay in progress until dynamic_api, UI contract, and loader validation close"
    );

    for (row_name, required_anchors) in [
        (
            "2.1 重复定义消化",
            &[
                "待开始",
                "Runtime 09/editor UI owner",
                "`interface/ui` 与 `runtime/ui` 重复定义清单",
            ][..],
        ),
        (
            "2.2 v2 契约同步",
            &[
                "待开始",
                "v2-replacement-mainline",
                "Runtime 09/editor UI owner",
            ][..],
        ),
    ] {
        let row = runtime_10_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 10 should keep M2 row `{row_name}`"));
        assert_contains_all("Runtime 10 M2 pending status row", row, required_anchors);
    }

    let m2_gate_row = runtime_10_plan
        .lines()
        .find(|line| line.contains("| 横切 | M2 UI 镜像契约 pending gate |"))
        .expect("Runtime 10 should keep the M2 UI contract pending gate status row");
    assert_contains_all(
        "Runtime 10 M2 UI contract gate row",
        m2_gate_row,
        &[
            "code_static_pending_owner_cargo",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "Runtime 09/editor UI owner",
            "未修改 UI/interface 生产类型",
        ],
    );
    for forbidden_claim in ["completed", "cargo_passed", "full_package_passed"] {
        assert!(
            !m2_gate_row.contains(forbidden_claim),
            "Runtime 10 M2 gate must not claim `{forbidden_claim}` before UI contract owner and Cargo validation passes"
        );
    }

    assert_contains_all(
        "Runtime 10 M2 validation commands",
        runtime_10_plan,
        &[
            "cargo test -p zircon_runtime_interface --locked",
            "cargo test -p zircon_runtime --lib ui --locked",
            "cargo check -p zircon_editor --lib --locked",
        ],
    );

    let runtime_10_index_row =
        runtime_index_row_for(runtime_index, "10-dynamic-api-and-interface-convergence.md");
    assert_contains_all(
        "Runtime 10 index row",
        runtime_10_index_row,
        &[
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "M2 UI 镜像契约 owner/Cargo gate",
            "Runtime 09/editor UI owner",
        ],
    );

    let runtime_10_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P13 |"))
        .expect("Runtime index should keep the P13 dynamic API problem row");
    assert_contains_all(
        "Runtime index P13 row",
        runtime_10_problem_row,
        &[
            "interface `ui/` 22 条目镜像契约",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "UI 镜像契约 M2 owner/Cargo gate",
        ],
    );

    assert_contains_all(
        "Runtime 05 closeout plan",
        runtime_05_plan,
        &[
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "Runtime 10 UI 镜像契约 M2 owner/Cargo gate",
        ],
    );
    assert_contains_all(
        "Runtime interface convergence doc",
        convergence_doc,
        &[
            "Runtime 10 UI Contract M2 Gate",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "Runtime 09/editor UI owner",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 10 M2 gate",
        review,
        &[
            "Runtime 10 UI Contract M2 Gate",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "UI 镜像契约 M2",
        ],
    );
}

#[test]
fn runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass() {
    let runtime_11_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let job_system_doc = include_str!("../../../../../../docs/zircon_runtime/core/job_system.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );
    let runtime_05_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );

    assert_eq!(
        frontmatter_status(runtime_11_plan),
        Some("in_progress"),
        "Runtime 11 should stay in progress until tasks/ecs_schedule/worker_pool/rayon validation closes"
    );

    for (row_name, required_anchors) in [
        (
            "1.1 句柄与依赖",
            &["code_static_pending_cargo", "tasks", "Cargo"][..],
        ),
        (
            "1.2 parallel_for",
            &["code_static_pending_cargo", "tasks", "Cargo"][..],
        ),
        (
            "2.1 剔除旁路收编",
            &[
                "pre_m2_1_rayon_render_exception_guard_static_passed_pending_render_owner",
                "render-owner-pending-runtime-11-m2-1-cutover",
                "actual graphics cutover not executed",
            ][..],
        ),
        (
            "2.2 rayon 守卫",
            &[
                "code_static_pending_render_cutover_cargo",
                "M2.2 Cargo 仍待",
                "render owner",
            ][..],
        ),
        (
            "2.3 ECS 批次依赖化",
            &["code_static_pending_cargo", "ecs_schedule", "Cargo"][..],
        ),
        (
            "2.4 asset 线程裁决",
            &["code_static_pending_cargo", "worker_pool", "Cargo"][..],
        ),
        (
            "3.1 调度诊断",
            &["code_static_pending_cargo", "tasks", "Cargo"][..],
        ),
        (
            "3.2 压测锚",
            &["code_static_pending_cargo", "tasks", "Cargo"][..],
        ),
    ] {
        let row = runtime_11_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 11 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 11 pending status row", row, required_anchors);
    }

    assert_contains_all(
        "Runtime 11 validation gate commands",
        runtime_11_plan,
        &[
            "cargo check -p zircon_runtime --lib --locked",
            "cargo test -p zircon_runtime --lib tasks --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib job --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib rayon --locked",
            "cargo test -p zircon_runtime --lib ecs_schedule --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib worker_pool --locked",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
        ],
    );

    let runtime_11_index_row = runtime_index_row_for(runtime_index, "11-job-system-task-model.md");
    assert_contains_all(
        "Runtime 11 index row",
        runtime_11_index_row,
        &[
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "tasks/ecs_schedule/worker_pool/rayon",
            "Cargo",
        ],
    );

    let runtime_11_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P14 |"))
        .expect("Runtime index should keep the P14 JobSystem problem row");
    assert_contains_all(
        "Runtime index P14 row",
        runtime_11_problem_row,
        &[
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "parallel_frustum",
            "tasks/ecs_schedule/worker_pool/rayon",
        ],
    );

    assert_contains_all(
        "Runtime JobSystem module doc",
        job_system_doc,
        &[
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "tasks/ecs_schedule/worker_pool/rayon",
            "parallel_frustum.rs",
        ],
    );

    assert_contains_all(
        "Runtime 05 closeout plan",
        runtime_05_plan,
        &[
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "Runtime 11 `tasks/ecs_schedule/worker_pool/rayon` Cargo gate",
        ],
    );

    assert_contains_all(
        "Runtime architecture review Runtime 11 gate",
        review,
        &[
            "Runtime 11 JobSystem Cargo Gate",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "tasks/ecs_schedule/worker_pool/rayon",
        ],
    );
}

#[test]
fn runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation() {
    let runtime_12_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../../../docs/zircon_runtime/input/input_state.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_12_plan),
        Some("in_progress"),
        "Runtime 12 should stay in progress until input/action/gamepad validation closes"
    );

    for (row_name, status_anchor) in [
        (
            "0.1 链路与帧语义",
            "input_frame_contract_static_passed_cargo_pending",
        ),
        (
            "1.1 动作映射设计",
            "action_contract_static_passed_cargo_pending",
        ),
        (
            "1.2 最小实现",
            "action_evaluator_static_passed_cargo_pending",
        ),
        (
            "2.1 gamepad 桥接",
            "gamepad_bridge_static_passed_cargo_pending",
        ),
    ] {
        let row = runtime_12_plan
            .lines()
            .find(|line| line.contains(row_name))
            .unwrap_or_else(|| panic!("Runtime 12 should keep status row `{row_name}`"));
        assert_contains_all(
            "Runtime 12 pending status row",
            row,
            &[status_anchor, "Cargo"],
        );
    }

    assert_contains_all(
        "Runtime 12 validation gate commands",
        runtime_12_plan,
        &[
            "cargo test -p zircon_runtime --lib input --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib action_map --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib gamepad --locked -- --nocapture",
            "cargo test -p zircon_app --locked",
            "runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation",
        ],
    );

    let runtime_12_index_row =
        runtime_index_row_for(runtime_index, "12-input-stack-and-action-mapping.md");
    assert_contains_all(
        "Runtime 12 index row",
        runtime_12_index_row,
        &[
            "Runtime 12 输入契约/runtime/tests",
            "input/action_map/gamepad/app filters",
            "Cargo 待 active lane 清空",
        ],
    );

    assert_contains_all(
        "Runtime input module doc",
        input_doc,
        &[
            "Frame Input Contract",
            "DefaultInputManager::begin_frame()",
            "InputActionEvaluator",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 12 gate",
        review,
        &[
            "Runtime 12 Input Stack Guard",
            "runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation",
            "input/action_map/gamepad/app",
        ],
    );
}

#[test]
fn runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass() {
    let runtime_13_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let function_ledger =
        include_str!("../../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_13_plan),
        Some("in_progress"),
        "Runtime 13 should stay in progress until script binding validation closes"
    );

    for required_row in [
        "1.1 清册守卫",
        "1.2 capability 测试",
        "2.1 句柄失效语义",
        "2.2 访问路径收束",
    ] {
        let row = runtime_13_plan
            .lines()
            .find(|line| line.contains(required_row))
            .unwrap_or_else(|| panic!("Runtime 13 should keep status row `{required_row}`"));
        assert_contains_all(
            "Runtime 13 pending status row",
            row,
            &["code_static_pending_cargo", "Cargo"],
        );
    }

    assert_contains_all(
        "Runtime 13 validation gate commands",
        runtime_13_plan,
        &[
            "cargo test -p zircon_runtime --lib script --locked -- --nocapture",
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "host_function_registry_matches_documented_ledger",
            "host_capability_representatives_are_declared_on_registered_modules",
            "script_held_entity_handle_reports_invalid_after_despawn",
            "script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi",
        ],
    );

    let runtime_13_index_row =
        runtime_index_row_for(runtime_index, "13-script-binding-and-reflection.md");
    assert_contains_all(
        "Runtime 13 index row",
        runtime_13_index_row,
        &[
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "script filters",
            "Cargo 待 active lane 清空",
        ],
    );

    let runtime_13_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P16 |"))
        .expect("Runtime index should keep the P16 script binding problem row");
    assert_contains_all(
        "Runtime index P16 row",
        runtime_13_problem_row,
        &[
            "function_ledger.md",
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "Cargo 待 active lane 清空",
        ],
    );

    assert_contains_all(
        "Runtime script host ledger doc",
        function_ledger,
        &[
            "6 host modules, 50 fixed host functions, and 2 fixed script type descriptors",
            "host_function_registry_matches_documented_ledger",
            "host_capability_representatives_are_declared_on_registered_modules",
            "script_held_entity_handle_reports_invalid_after_despawn",
            "script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi",
            "pending: cargo test -p zircon_runtime --lib script --locked -- --nocapture",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 13 gate",
        review,
        &[
            "Runtime 13 Script Binding Guard",
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "script filters",
        ],
    );
}

#[test]
fn runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass() {
    let runtime_14_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");

    assert_eq!(
        frontmatter_status(runtime_14_plan),
        Some("in_progress"),
        "Runtime 14 should stay in progress until module-family Cargo/rustc gates close"
    );

    for required_m1_row in [
        "navigation 文件集守卫",
        "engine_module declared-layer 守卫",
        "diagnostic_log 单桥接守卫",
        "animation backlog/非目标守卫",
        "crate 根四族席位总守卫",
    ] {
        let row = runtime_14_plan
            .lines()
            .find(|line| line.contains(required_m1_row))
            .unwrap_or_else(|| panic!("Runtime 14 should keep status row `{required_m1_row}`"));
        assert_contains_all(
            "Runtime 14 M1 status row",
            row,
            &["code_static_pending_cargo", "Cargo"],
        );
    }

    let runtime_14_index_row =
        runtime_index_row_for(runtime_index, "14-runtime-module-family-closeout.md");
    assert_contains_all(
        "Runtime 14 index row",
        runtime_14_index_row,
        &[
            "Runtime 14 计划/四族文档",
            "独立 `root_entries.rs` rustc 守卫曾通过 10/10",
            "Cargo/重跑 rustc 待 active lane 清空",
        ],
    );

    let runtime_14_problem_row = runtime_index
        .lines()
        .find(|line| line.starts_with("| P17 |"))
        .expect("Runtime index should keep the P17 module-family problem row");
    assert_contains_all(
        "Runtime index P17 row",
        runtime_14_problem_row,
        &[
            "runtime_14_module_family_root_seats_match_documented_judgements",
            "Cargo/rustc",
            "待 active lane 清空",
        ],
    );
}
