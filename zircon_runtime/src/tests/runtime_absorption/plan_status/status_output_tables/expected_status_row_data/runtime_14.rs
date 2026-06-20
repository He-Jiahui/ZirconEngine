use super::ExpectedStatusOutputSlice;

#[path = "runtime_14/audit_sync.rs"]
mod audit_sync;
#[path = "runtime_14/cargo_gates.rs"]
mod cargo_gates;
#[path = "runtime_14/guard_anchors.rs"]
mod guard_anchors;

pub(super) const RUNTIME_14_AUDIT_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    audit_sync::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_14_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    guard_anchors::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_14_CARGO_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    cargo_gates::EXPECTED_STATUS_OUTPUT_SLICES;
