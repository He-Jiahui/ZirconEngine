use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 10 Dynamic API 镜像文档守卫",
        [
            "runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts",
            "dynamic_runtime_api_boundary",
            "standalone rustc 4/4",
            "dynamic_api/app/UI gates pending",
        ],
    ),
    (
        "Runtime 10 Dynamic API 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 15",
            "missing_behavior_test_anchors = []",
            "standalone dynamic_api_session 5/5",
            "dynamic_api/app/UI gates pending",
        ],
    ),
    (
        "Runtime 10 dynamic_api_session 吸收守卫拆分",
        [
            "dynamic_api_session/{shared,headless_profiles,event_split,test_owner_split,ffi_panic_boundary,mirror_docs}.rs",
            "expected_source_file_count = 21",
            "cargo test -p zircon_runtime --lib dynamic_api_session",
            "5 passed / 4231 filtered out",
        ],
    ),
    (
        "Runtime 10 Dynamic Session Event Split",
        [
            "session/events.rs",
            "runtime_10_dynamic_session_event_split_keeps_abi_owner_and_event_router",
            "expected_source_file_count = 21",
            "active compile lanes deferred",
        ],
    ),
    (
        "Runtime 10 Dynamic Session Test Owner Split",
        [
            "session/tests/{mod,helpers,vampire_gameplay,vampire_menu,vampire_hud,frame_diagnostics,runtime_errors}.rs",
            "runtime_10_dynamic_session_test_owner_split_keeps_focused_modules",
            "session/tests/frame_diagnostics.rs",
            "standalone `dynamic_api_session.rs` 5/5",
        ],
    ),
    (
        "Runtime 11 JobSystem 镜像文档守卫",
        [
            "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
            "job_system_boundary",
            "standalone rustc 1/1",
            "tasks/ecs_schedule/worker_pool/rayon Cargo gates pending",
        ],
    ),
    (
        "Runtime 11 JobSystem 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 9",
            "missing_behavior_test_anchors = []",
            "standalone job_system 1/1",
            "standalone status-output 2/2",
        ],
    ),
    (
        "Runtime 12 Input stack 镜像文档守卫",
        [
            "runtime_12_input_stack_mirror_docs_match_structure_audit_counts",
            "input_stack_boundary",
            "standalone rustc 4/4",
            "Cargo input/action_map/gamepad/app gates pending",
        ],
    ),
    (
        "Runtime 12 gamepad event-owner 漂移同步",
        [
            "session/events.rs",
            "InputEvent::Gamepad*",
            "missing_gamepad_abi_anchors = []",
            "standalone `input_stack.rs` rustc 4/4 passed",
        ],
    ),
    (
        "Runtime 12 Input stack 行为测试锚审计同步",
        [
            "behavior_test_anchor_count = 6",
            "missing_behavior_test_anchors = []",
            "standalone input_stack 4/4",
            "input/action_map/gamepad/app Cargo gates pending",
        ],
    ),
    (
        "Runtime 13 Script binding 镜像文档守卫",
        [
            "runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
            "expected_source_file_count = 18",
            "standalone rustc 2/2",
            "script Cargo filters pending",
        ],
    ),
    (
        "Runtime 13 Gameplay Host Owner Split",
        [
            "gameplay_host/{combat,components,input,lifecycle,navigation,script_bindings,transform,values}.rs",
            "runtime_13_gameplay_host_owner_split_keeps_domain_files",
            "script::vm -- --nocapture` 47/47 passed",
            "script Cargo filters pending",
        ],
    ),
];
