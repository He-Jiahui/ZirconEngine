use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 10 Dynamic API 镜像文档守卫",
        &[
            "runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts",
            "dynamic_runtime_api_boundary",
            "expected_source_file_count = 35",
            "dynamic_api/app/UI gates pending",
        ],
    ),
    (
        "Runtime 10 host-request payload ABI boundary",
        &[
            "host_request_payload_anchors = 38/38",
            "missing_host_request_payload_anchors = []",
            "expected_source_file_count = 35",
            "dynamic_api/app/UI Cargo gates pending",
        ],
    ),
    (
        "Runtime 10 F18 asset manager resolution return shape",
        &[
            "runtime_10_asset_manager_resolution_handle_shape_coremin_check_passed",
            "review_f18_asset_manager_resolution_returns_registered_handle",
            "Result<Arc<AssetManagerHandle>, CoreError>",
            "dynamic project boundary calls `.shared()`",
        ],
    ),
    (
        "Runtime 10 Dynamic API current audit recheck",
        &[
            "dynamic_api_current_audit_static_passed_cargo_pending",
            "source files 35/35",
            "standalone `dynamic_api_session.rs` 9/9",
            "runtime UI/editor Cargo gates",
        ],
    ),
    (
        "Runtime 10 Dynamic API 2026-07-01 current audit recheck",
        &[
            "dynamic_api_20260701_current_audit_static_passed_cargo_deferred",
            "source files 35/35",
            "behavior-test anchors 16/16",
            "full `audit_runtime_structure.py --json` 风险汇总为 `{}`",
        ],
    ),
    (
        "Runtime 10 dynamic_api_session Cargo 验证窗口探测",
        &[
            "604s",
            "未生成 `zircon_runtime` 测试二进制或测试结果",
            "standalone `dynamic_api_session.rs` 9/9",
            "runtime UI/editor Cargo gates",
        ],
    ),
    (
        "Runtime 10 Dynamic API 行为测试锚审计同步",
        &[
            "behavior_test_anchor_count = 16",
            "missing_behavior_test_anchors = []",
            "standalone `dynamic_api_session.rs` 9/9",
            "standalone dynamic_api_session 9/9",
            "dynamic_api/app/UI gates pending",
        ],
    ),
    (
        "Runtime 10 dynamic_api_session 吸收守卫拆分",
        &[
            "dynamic_api_session/{shared,headless_profiles,event_split,test_owner_split,ffi_panic_boundary,runtime_diagnostics,ui_contract,v2_contract,mirror_docs}.rs",
            "expected_source_file_count = 35",
            "cargo test -p zircon_runtime --lib dynamic_api_session",
            "5 passed / 4231 filtered out",
        ],
    ),
    (
        "Runtime 10 runtime diagnostics profile-control snapshot",
        &[
            "runtime_diagnostics_profile_control_static_passed_cargo_deferred_tests_deferred",
            "ProfileControlCommand::RuntimeDiagnosticsSnapshot",
            "runtime_diagnostics_anchors = 15/15",
            "no new `ZrRuntimeApiV1` function pointer",
        ],
    ),
    (
        "Runtime 10 diagnostics inventory split",
        &[
            "runtime_10_dynamic_api_diagnostics_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "dynamic_runtime_api_diagnostics_inventory.py",
            "scene_asset_reload_diagnostic_path_anchors = 21/21",
            "missing_scene_asset_reload_diagnostic_path_anchors = []",
        ],
    ),
    (
        "Runtime 10 host-request inventory split",
        &[
            "runtime_10_host_request_payload_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "dynamic_runtime_api_host_request_inventory.py",
            "host_request_payload_anchors = 38/38",
            "missing_host_request_payload_anchors = []",
        ],
    ),
    (
        "Runtime 10 UI contract inventory split",
        &[
            "runtime_10_ui_contract_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "dynamic_runtime_api_ui_contract_inventory.py",
            "ui_pending_gate_anchors = 8/8",
            "ui_v2_contract_sync_anchors = 9/9",
        ],
    ),
    (
        "Runtime 10 validation inventory split",
        &[
            "runtime_10_dynamic_api_validation_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "dynamic_runtime_api_validation_inventory.py",
            "behavior_test_anchor_count = 16",
            "pending_cargo_gate_anchors = 5/5",
        ],
    ),
    (
        "Runtime 10 session lifecycle inventory split",
        &[
            "runtime_10_session_lifecycle_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "dynamic_runtime_api_session_lifecycle_inventory.py",
            "headless_lifecycle_anchors = 12/12",
            "missing_headless_lifecycle_anchors = []",
        ],
    ),
    (
        "Runtime 10 failure boundary inventory split",
        &[
            "runtime_10_failure_boundary_inventory_split_static_passed_cargo_deferred_tests_deferred",
            "dynamic_runtime_api_failure_inventory.py",
            "ffi_panic_anchors = 9/9",
            "loader_failure_anchors = 10/10",
        ],
    ),
    (
        "Runtime 10 ABI source inventory split",
        &[
            "runtime_10_dynamic_api_abi_inventory_split_static_passed_cargo_timeout_no_result_tests_deferred",
            "dynamic_runtime_api_abi_inventory.py",
            "expected_source_file_count = 35",
            "runtime_session_operation_count = 11",
        ],
    ),
    (
        "Runtime 10 runtime API Markdown renderer split",
        &[
            "runtime_api_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "runtime_api_markdown.py",
            "folder-backed owner modules 6/6",
            "facade 12/20 non-empty lines",
        ],
    ),
    (
        "Runtime 10 dynamic runtime API Markdown renderer split",
        &[
            "dynamic_runtime_api_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "dynamic_runtime_api_markdown.py",
            "dynamic_runtime_api_boundary.py` remains the 330-line audit/risk owner",
            "Markdown owner is 65 lines",
        ],
    ),
    (
        "Runtime 10 session profile owner audit sync",
        &[
            "runtime_10_session_profile_owner_audit_sync_static_passed_cargo_deferred",
            "dynamic_api/session/profile.rs",
            "expected_source_file_count = 35",
            "headless_lifecycle_anchors = 12/12",
            "missing_headless_lifecycle_anchors = []",
        ],
    ),
    (
        "Runtime 10 host-request payload test owner split",
        &[
            "runtime_10_host_request_payload_test_owner_split_static_passed_cargo_timeout_no_result",
            "dynamic_api/tests/host_request_payloads.rs",
            "expected_module_count = 12",
            "oversized_modules = []",
            "focused Cargo `cargo test -p zircon_runtime --lib host_request --locked --jobs 1` timed out after 604s with no test result",
        ],
    ),
    (
        "Runtime 10 dynamic input mouse-wheel event owner guard",
        &[
            "dynamic_input_mouse_wheel_event_owner_guard_focused_cargo_passed_broader_input_pending",
            "include_str!(\"../session/events.rs\")",
            "mouse_wheel_at_events_decode_delta_bits_for_dynamic_session",
            "1 passed; 0 failed; 4704 filtered out",
        ],
    ),
    (
        "Runtime 10 Vampire W input real-backend gate",
        &[
            "dynamic_vampire_w_input_real_backend_gate_ignored_without_zr_vm_remaining_ui_input_pending",
            "requires backend-zr-vm and ZR_VM_RUST_BINDING_LIB_DIR",
            "vampire_project_session_w_key_moves_player_before_input_clear",
            "0 passed; 0 failed; 1 ignored",
        ],
    ),
];
