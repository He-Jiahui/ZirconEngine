use super::*;

#[test]
fn runtime_15_m3_child_group_moved_lock_poison_rows_are_child_owned() {
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_ROWS_PATH);
    let lock_poison_status = read_runtime_src(LOCK_POISON_STATUS_ROWS_PATH);
    let lock_poison_child_rows = [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/status_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/policy_guards.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/core_runtime_recovery.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/runtime_services_recovery.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/resource_render_input_recovery.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/script_vm_recovery.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status/row_data_owner.rs",
    ]
    .map(read_runtime_src)
    .join("\n");

    assert!(
        !lock_poison_status.contains("( \"Runtime 15 M3"),
        "lock_poison_status.rs should only route child row owners and must not retain a raw-string row mirror"
    );

    for moved_lock_poison_row in [
        "Runtime 15 M3 lock poison policy guard folder split",
        "Runtime 15 M3 core runtime lock poison guard child-owner split",
        "Runtime 15 M3 F2 lock poison recovery guard",
        "Runtime 15 M3 production direct lock unwrap global gate",
        "Runtime 15 M3 config store lock poison recovery",
        "Runtime 15 M3 core runtime devtools lock poison recovery",
        "Runtime 15 M3 core handle diagnostics lock poison recovery",
        "Runtime 15 M3 core handle time lock poison recovery",
        "Runtime 15 M3 core handle states lock poison recovery",
        "Runtime 15 M3 core runtime task lock poison recovery",
        "Runtime 15 M3 core runtime profiling lock poison recovery",
        "Runtime 15 M3 core handle registry lock poison recovery",
        "Runtime 15 M3 plugin bridge table lock poison recovery",
        "Runtime 15 M3 native live-host bridge methods lock poison recovery",
        "Runtime 15 M3 navigation lock poison recovery",
        "Runtime 15 M3 dynamic API session lock poison recovery",
        "Runtime 15 M3 dynamic scene spawn task lock poison recovery",
        "Runtime 15 M3 scene ECS parallel executor lock poison recovery",
        "Runtime 15 M3 core resource manager lock poison recovery",
        "Runtime 15 M3 asset project manager lock poison recovery",
        "Runtime 15 M3 asset worker pool lock poison recovery",
        "Runtime 15 M3 WGPU render framework lock poison recovery",
        "Runtime 15 M3 RHI WGPU render device lock poison recovery",
        "Runtime 15 M3 animation manager lock poison recovery",
        "Runtime 15 M3 input runtime manager lock poison recovery",
        "Runtime 15 M3 script VM registry lock poison recovery",
        "Runtime 15 M3 VM plugin manager selected-backend lock poison recovery",
    ] {
        assert!(
            !foundation_guards.contains(moved_lock_poison_row),
            "foundation_guards.rs should delegate lock-poison status rows to lock_poison_status.rs instead of keeping {moved_lock_poison_row}"
        );
        assert!(
            lock_poison_child_rows.contains(moved_lock_poison_row),
            "lock_poison_status child modules should own moved lock-poison status row {moved_lock_poison_row}"
        );
    }
}
