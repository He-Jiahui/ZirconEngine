use super::*;

pub(super) fn assert_module_convention_status_export_chain_is_current() {
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "Runtime 15 M3 exports module-convention status children",
        &runtime_15_m3,
        &[
            "MODULE_CONVENTION_STATUS_FRONTMATTER_AND_GATE_EXPECTED_STATUS_OUTPUT_SLICES",
            "MODULE_CONVENTION_STATUS_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "MODULE_CONVENTION_STATUS_AUDIT_EXPECTED_STATUS_OUTPUT_SLICES",
            "MODULE_CONVENTION_STATUS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 and top-level exports include module-convention status children",
        &[runtime_15.as_str(), top_level.as_str()].join("\n"),
        &[
            "RUNTIME_15_M3_MODULE_CONVENTION_STATUS_FRONTMATTER_AND_GATE_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_MODULE_CONVENTION_STATUS_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_MODULE_CONVENTION_STATUS_AUDIT_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_MODULE_CONVENTION_STATUS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
