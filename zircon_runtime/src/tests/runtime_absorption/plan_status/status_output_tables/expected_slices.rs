pub(super) fn expected_status_for_slice(slice: &str) -> &'static str {
    if slice == "Runtime 14 Cargo 验证窗口探测" {
        "cargo_deferred_active_lane"
    } else if slice == "Runtime 14 animation Cargo gate 尝试" {
        "cargo_blocked_external_compile_drift"
    } else if slice == "Runtime 14 animation Cargo gate 修复与复验阻塞" {
        "cargo_recheck_blocked_external_ui_compile_drift"
    } else if slice == "Runtime 14 animation runtime-status focused recheck timeout" {
        "cargo_recheck_timeout_no_result"
    } else if slice == "Runtime 05 plan-status Cargo attempt 状态审计" {
        "cargo_attempt_status_static_passed_cargo_pending"
    } else if slice == "Runtime 05 plan-status Cargo timeout 状态审计" {
        "cargo_attempt_timeout_status_static_passed_cargo_pending"
    } else if slice == "Runtime 05 full scene closeout failed evidence" {
        "cargo_recheck_failed_full_scene_gate"
    } else if slice == "Runtime 05 full scene closeout no-result recheck" {
        "cargo_recheck_no_result_external_editor_lane"
    } else if slice == "Runtime 05 scene:: failure support-first triage" {
        "support_first_triage_static_passed_cargo_pending"
    } else if slice == "Runtime 05 serialization source folder-split guard sync" {
        "source_guard_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 03 world bootstrap fixed-loop stage guard sync" {
        "guard_sync_static_passed_cargo_pending"
    } else if slice == "Runtime 07 scene asset split-drift repair" {
        "split_drift_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 07 scene asset folder-split public-surface guard" {
        "folder_split_guard_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 07 scene asset guard 纳入 performance_hotpath_boundary" {
        "boundary_guard_anchor_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 07 project_io folder split"
        || slice == "Runtime 10 Dynamic Session Event Split"
        || slice == "Runtime 10 Dynamic Session Test Owner Split"
    {
        "folder_split_static_passed_cargo_deferred_active_lanes"
    } else if slice == "Runtime 13 Gameplay Host Owner Split" {
        "folder_split_static_passed_script_vm_cargo_broader_gate_pending"
    } else if slice == "Runtime 02 generated template count 审计同步" {
        "structure_audit_static_passed_cargo_pending"
    } else if matches!(
        slice,
        "Runtime 05 recent-static Runtime 02/07 status metadata guard"
            | "Runtime 05 status-output recent-static metadata row"
            | "Runtime 05 status-output Runtime 12 gamepad event-owner row"
            | "Runtime 05 status-output Runtime 12 behavior-test row"
            | "Runtime 05 status-output Runtime 04 behavior-test row"
            | "Runtime 05 status-output Runtime 08 behavior-test row"
            | "Runtime 05 status-output Runtime 10 behavior-test row"
            | "Runtime 05 plan-status output-anchor module split"
            | "Runtime 05 plan-status output-anchor budget guard"
            | "Runtime 05 status-output status/date helper split"
            | "Runtime 05 status-output expected anchor split"
            | "Runtime 05 plan-status root module split"
            | "Runtime 05 plan-status support inventory split"
            | "Runtime 05 plan-status anchor inventory split"
            | "Runtime 05 plan-status markdown renderer split"
            | "Runtime 05 plan-status source helper split"
            | "Runtime 05 status-output expected row data split"
            | "Runtime 05 cargo-gates early Runtime 03 split"
            | "Runtime 05 cargo-gates early Runtime 01 split"
            | "Runtime 05 cargo-gates early Runtime 02 split"
            | "Runtime 05 cargo-gates early Runtime 04 split"
            | "Runtime 05 cargo-gates early Runtime 06 split"
            | "Runtime 05 cargo-gates early Runtime 08 split"
            | "Runtime 05 cargo-gates early Runtime 07 split"
            | "Runtime 05 cargo-gates late Runtime 10 split"
            | "Runtime 05 cargo-gates late Runtime 11 split"
            | "Runtime 05 cargo-gates late Runtime 12 split"
            | "Runtime 05 cargo-gates late Runtime 13 split"
            | "Runtime 05 cargo-gates late Runtime 14 split"
            | "Runtime 05 plan-status 输出表守卫"
            | "Runtime 05 plan-status 审计元数据守卫"
            | "Runtime 05 status-output Runtime 07 scene asset rows"
            | "Runtime 05 Runtime 07 scene status 审计元数据"
            | "Runtime 05 status-output Runtime 02 generated template row"
            | "Runtime 05 Runtime 02 generated status 审计元数据"
            | "Runtime 05 status-output Runtime 07 owner-budget row"
            | "Runtime 05 Runtime 07 owner-budget status 审计元数据"
            | "Runtime 05 status-output Runtime 03 module-doc row"
            | "Runtime 05 status-output Runtime 03 behavior-test row"
            | "Runtime 05 status-output all-index-row coverage guard"
            | "Runtime 05 status-output non-network server allowlist row"
    ) {
        "status_table_static_passed_cargo_pending"
    } else if slice == "Runtime 05 M0 absorption guard coverage sync" {
        "static_docs_passed_cargo_pending"
    } else if slice == "Runtime 05 non-network server UI sortingMode allowlist" {
        "static_audit_passed_cargo_pending"
    } else if slice == "Runtime 05 naming_boundary non-network server Rust guard" {
        "naming_guard_static_passed_cargo_pending"
    } else if slice == "Runtime 05 status-output row-data group split" {
        "status_table_static_passed_cargo_pending"
    } else if slice == "Runtime 10 dynamic_api_session 吸收守卫拆分" {
        "focused_cargo_passed_broader_gates_pending"
    } else if slice == "Runtime 08 First-stage event update guard" {
        "mirror_docs_static_passed_cargo_pending"
    } else if slice == "Runtime 12 gamepad event-owner 漂移同步" {
        "input_boundary_static_passed_cargo_pending"
    } else {
        "mirror_docs_static_passed_cargo_pending"
    }
}

pub(super) fn expected_date_for_slice(slice: &str) -> &'static str {
    if matches!(
        slice,
        "Runtime 07 owner-budget 38-hotspot 回漂同步"
            | "Runtime 07 owner-budget 39-hotspot 漂移同步"
            | "Runtime 07 owner-budget 37-hotspot 漂移同步"
            | "Runtime 07 owner-budget 37-hotspot 再同步"
            | "Runtime 05 status-output Runtime 07 owner-budget row"
    ) {
        "2026-06-15"
    } else if slice == "Runtime 14 animation runtime-status focused recheck timeout" {
        "2026-06-15"
    } else if slice == "Runtime 14 Module family guard anchors 审计同步" {
        "2026-06-15"
    } else if slice == "Runtime 05 plan-status Cargo timeout 状态审计" {
        "2026-06-15"
    } else if slice == "Runtime 12 gamepad event-owner 漂移同步"
        || slice == "Runtime 01 Tech-stack 行为测试锚审计同步"
        || slice == "Runtime 02 core/root/generated 镜像文档守卫"
        || slice == "Runtime 02 guard-test anchors 审计同步"
        || slice == "Runtime 10 Dynamic API 行为测试锚审计同步"
        || slice == "Runtime 10 dynamic_api_session 吸收守卫拆分"
        || slice == "Runtime 12 Input stack 行为测试锚审计同步"
        || slice == "Runtime 04 Asset pipeline 行为测试锚审计同步"
        || slice == "Runtime 08 ECS 行为测试锚审计同步"
        || slice == "Runtime 05 status-output Runtime 08 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 12 gamepad event-owner row"
        || slice == "Runtime 05 status-output Runtime 12 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 04 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 03 module-doc row"
        || slice == "Runtime 05 status-output Runtime 03 behavior-test row"
        || slice == "Runtime 05 status-output Runtime 10 behavior-test row"
        || slice == "Runtime 05 plan-status output-anchor module split"
        || slice == "Runtime 05 plan-status output-anchor budget guard"
        || slice == "Runtime 05 status-output status/date helper split"
        || slice == "Runtime 05 status-output expected anchor split"
        || slice == "Runtime 05 status-output row-data group split"
        || slice == "Runtime 05 plan-status root module split"
        || slice == "Runtime 05 plan-status support inventory split"
        || slice == "Runtime 05 plan-status anchor inventory split"
        || slice == "Runtime 05 plan-status markdown renderer split"
        || slice == "Runtime 05 plan-status source helper split"
        || slice == "Runtime 05 status-output expected row data split"
        || slice == "Runtime 05 full scene closeout failed evidence"
        || slice == "Runtime 05 full scene closeout no-result recheck"
        || slice == "Runtime 05 scene:: failure support-first triage"
        || slice == "Runtime 05 serialization source folder-split guard sync"
        || slice == "Runtime 03 world bootstrap fixed-loop stage guard sync"
        || slice == "Runtime 05 cargo-gates early Runtime 03 split"
        || slice == "Runtime 05 cargo-gates early Runtime 01 split"
        || slice == "Runtime 05 cargo-gates early Runtime 02 split"
        || slice == "Runtime 05 cargo-gates early Runtime 04 split"
        || slice == "Runtime 05 cargo-gates early Runtime 06 split"
        || slice == "Runtime 05 cargo-gates early Runtime 08 split"
        || slice == "Runtime 05 cargo-gates early Runtime 07 split"
        || slice == "Runtime 05 cargo-gates late Runtime 10 split"
        || slice == "Runtime 05 cargo-gates late Runtime 11 split"
        || slice == "Runtime 05 cargo-gates late Runtime 12 split"
        || slice == "Runtime 05 cargo-gates late Runtime 13 split"
        || slice == "Runtime 05 cargo-gates late Runtime 14 split"
        || slice == "Runtime 05 status-output all-index-row coverage guard"
        || slice == "Runtime 03 Schedule/frame-loop 行为测试锚审计同步"
        || slice == "Runtime 08 First-stage event update guard"
        || slice == "Runtime 11 JobSystem 行为测试锚审计同步"
    {
        "2026-06-15"
    } else {
        "2026-06-14"
    }
}
