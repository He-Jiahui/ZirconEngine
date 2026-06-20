use super::ExpectedStatusOutputSlice;

#[path = "runtime_10/dynamic_api.rs"]
mod dynamic_api;
#[path = "runtime_10/session.rs"]
mod session;
#[path = "runtime_10/ui_contract.rs"]
mod ui_contract;

pub(super) const RUNTIME_10_DYNAMIC_API_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = dynamic_api::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_10_SESSION_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    session::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_10_UI_CONTRACT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = ui_contract::EXPECTED_STATUS_OUTPUT_SLICES;
