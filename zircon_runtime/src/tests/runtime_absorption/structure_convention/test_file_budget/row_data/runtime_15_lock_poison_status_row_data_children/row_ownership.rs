use super::*;

#[test]
fn runtime_15_lock_poison_status_row_data_owner_is_child_backed() {
    let lock_poison_status = read_runtime_src(LOCK_POISON_STATUS_ROWS_PATH);
    let status_rows = read_runtime_src(LOCK_POISON_STATUS_STATUS_ROWS_PATH);
    let policy_guards = read_runtime_src(LOCK_POISON_STATUS_POLICY_GUARDS_PATH);
    let core_runtime_recovery = read_runtime_src(LOCK_POISON_STATUS_CORE_RUNTIME_RECOVERY_PATH);
    let runtime_services_recovery =
        read_runtime_src(LOCK_POISON_STATUS_RUNTIME_SERVICES_RECOVERY_PATH);
    let resource_render_input_recovery =
        read_runtime_src(LOCK_POISON_STATUS_RESOURCE_RENDER_INPUT_RECOVERY_PATH);
    let script_vm_recovery = read_runtime_src(LOCK_POISON_STATUS_SCRIPT_VM_RECOVERY_PATH);
    let row_data_owner = read_runtime_src(LOCK_POISON_STATUS_ROW_DATA_OWNER_PATH);
    let row_children = [
        status_rows.as_str(),
        policy_guards.as_str(),
        core_runtime_recovery.as_str(),
        runtime_services_recovery.as_str(),
        resource_render_input_recovery.as_str(),
        script_vm_recovery.as_str(),
        row_data_owner.as_str(),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 lock-poison status row-data parent mounts child owners",
        &lock_poison_status,
        &[
            "#[path = \"lock_poison_status/status_rows.rs\"]",
            "#[path = \"lock_poison_status/policy_guards.rs\"]",
            "#[path = \"lock_poison_status/core_runtime_recovery.rs\"]",
            "#[path = \"lock_poison_status/runtime_services_recovery.rs\"]",
            "#[path = \"lock_poison_status/resource_render_input_recovery.rs\"]",
            "#[path = \"lock_poison_status/script_vm_recovery.rs\"]",
            "#[path = \"lock_poison_status/row_data_owner.rs\"]",
            "status_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "policy_guards::EXPECTED_STATUS_OUTPUT_SLICES",
            "core_runtime_recovery::EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_services_recovery::EXPECTED_STATUS_OUTPUT_SLICES",
            "resource_render_input_recovery::EXPECTED_STATUS_OUTPUT_SLICES",
            "script_vm_recovery::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !lock_poison_status.contains(
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &["
        ),
        "lock_poison_status.rs should route child row-data owners instead of owning row tuples directly"
    );
    assert_contains_all(
        "Runtime 15 lock-poison status row-data children own representative rows",
        &row_children,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
            "Runtime 15 M3 lock-poison status row-data child-owner split",
            "Runtime 15 M3 runtime services lock-poison guard child-owner split",
            "Runtime 15 M3 core handle registry lock poison recovery",
            "Runtime 15 M3 dynamic scene spawn task lock poison recovery",
            "Runtime 15 M3 input runtime manager lock poison recovery",
            "Runtime 15 M3 VM plugin manager selected-backend lock poison recovery",
        ],
    );
}
