use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 10 UI contract duplicate public types cleanup",
        [
            "runtime_10_ui_contract_types_have_single_definition_across_interface_and_runtime",
            "UiBindingCodec",
            "ui_contract_single_source_anchors = 7/7",
            "ui_contract_duplicate_public_types = 0",
        ],
    ),
    (
        "Runtime 10 UI v2 contract sync",
        [
            "runtime_10_ui_v2_contract_sync_matches_runtime_09_verdict_and_interface_owner",
            "ui_component_api_version_mismatch_is_rejected_with_parse_error",
            "ui_v2_contract_sync_anchors = 9/9",
            "UiComponentApiVersion",
        ],
    ),
];
