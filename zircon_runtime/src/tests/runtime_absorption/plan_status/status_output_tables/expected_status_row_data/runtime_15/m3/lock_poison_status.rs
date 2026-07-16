use super::ExpectedStatusOutputSlice;

#[path = "lock_poison_status/core_runtime_recovery.rs"]
mod core_runtime_recovery;
#[path = "lock_poison_status/policy_guards.rs"]
mod policy_guards;
#[path = "lock_poison_status/resource_render_input_recovery.rs"]
mod resource_render_input_recovery;
#[path = "lock_poison_status/row_data_owner.rs"]
mod row_data_owner;
#[path = "lock_poison_status/runtime_services_recovery.rs"]
mod runtime_services_recovery;
#[path = "lock_poison_status/script_vm_recovery.rs"]
mod script_vm_recovery;
#[path = "lock_poison_status/status_rows.rs"]
mod status_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    status_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const POLICY_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    policy_guards::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const CORE_RUNTIME_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = core_runtime_recovery::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_SERVICES_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = runtime_services_recovery::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RESOURCE_RENDER_INPUT_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = resource_render_input_recovery::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const SCRIPT_VM_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    script_vm_recovery::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
