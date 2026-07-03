type Slice = super::ExpectedStatusOutputSlice;

#[path = "priority_plan_docs/integrity_guards.rs"]
mod integrity_guards;
#[path = "priority_plan_docs/owner_guards.rs"]
mod owner_guards;
#[path = "priority_plan_docs/row_data_owner.rs"]
mod row_data_owner;
#[path = "priority_plan_docs/status_followups.rs"]
mod status_followups;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    integrity_guards::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const OWNER_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    owner_guards::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const OWNER_GUARDS_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    owner_guards::INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    status_followups::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
