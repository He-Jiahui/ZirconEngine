#[test]
fn runtime_10_m1_3_cargo_pending_gate_stays_explicit_until_dynamic_api_validation() {
    let runtime_10_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let dynamic_api_doc =
        include_str!("../../../../../../../docs/zircon_runtime/dynamic_api/session.md");

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

    let runtime_10_problem_row = runtime_index_problem_row_for(runtime_index, "P13", "dynamic API");
    assert_contains_all(
        "Runtime index P13 row",
        runtime_10_problem_row,
        &[
            "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
            "panic-boundary、完整 app loader 与 runtime UI/editor Cargo 验证仍待活动编译通道空闲后重跑",
            "interface package gate 已 168/168 通过",
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
        "../../../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_05_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let convergence_doc = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-interface-convergence.md"
    );
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
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
                "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
                "UiBindingCodec",
                "UiAssetSchemaVersionPolicy",
                "ui_contract_duplicate_public_types = 0",
            ][..],
        ),
        (
            "2.2 v2 契约同步",
            &[
                "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
                "v2-replacement-mainline",
                "UiComponentApiVersion",
                "ui_v2_contract_sync_anchors = 9/9",
            ][..],
        ),
    ] {
        let row_anchor = format!("| M2 | {row_name} |");
        let row = runtime_10_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
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
            "2.1 静态硬切已删除两个 runtime-local duplicate contract types",
            "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
            "ui_v2_contract_sync_anchors = 9/9",
            "interface package gate 已通过",
            "runtime ui/editor Cargo gates 仍 pending",
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
            "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
            "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
            "ui_contract_duplicate_public_types = 0",
            "ui_v2_contract_sync_anchors = 9/9",
        ],
    );

    let runtime_10_problem_row = runtime_index_problem_row_for(runtime_index, "P13", "dynamic API");
    assert_contains_all(
        "Runtime index P13 row",
        runtime_10_problem_row,
        &[
            "interface `ui/` 22 条目镜像契约",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
            "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
            "ui_contract_duplicate_public_types = 0",
            "ui_v2_contract_sync_anchors = 9/9",
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
            "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
            "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
            "ui_contract_duplicate_public_types = 0",
            "ui_v2_contract_sync_anchors = 9/9",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 10 M2 gate",
        review,
        &[
            "Runtime 10 UI Contract M2 Gate",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
            "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
            "ui_contract_duplicate_public_types = 0",
            "ui_v2_contract_sync_anchors = 9/9",
        ],
    );
}
