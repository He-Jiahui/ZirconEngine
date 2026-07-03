use super::*;

#[test]
fn runtime_15_asset_budget_row_data_owner_is_child_backed() {
    let asset_budget = read_runtime_src(ASSET_BUDGET_ROWS_PATH);
    let runtime_rhi = read_runtime_src(ASSET_BUDGET_RUNTIME_RHI_PATH);
    let asset_tests = read_runtime_src(ASSET_BUDGET_ASSET_TESTS_PATH);
    let budget_render_ui = read_runtime_src(ASSET_BUDGET_BUDGET_RENDER_UI_PATH);
    let naming_core_asset_dynamic = read_runtime_src(ASSET_BUDGET_NAMING_CORE_ASSET_DYNAMIC_PATH);
    let naming_graphics_misc = read_runtime_src(ASSET_BUDGET_NAMING_GRAPHICS_MISC_PATH);
    let row_data_owner = read_runtime_src(ASSET_BUDGET_ROW_DATA_OWNER_PATH);
    let row_children = [
        runtime_rhi.as_str(),
        asset_tests.as_str(),
        budget_render_ui.as_str(),
        naming_core_asset_dynamic.as_str(),
        naming_graphics_misc.as_str(),
        row_data_owner.as_str(),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 asset-budget row-data parent mounts child owners",
        &asset_budget,
        &[
            "#[path = \"asset_budget_tests/runtime_rhi.rs\"]",
            "#[path = \"asset_budget_tests/asset_tests.rs\"]",
            "#[path = \"asset_budget_tests/budget_render_ui.rs\"]",
            "#[path = \"asset_budget_tests/naming_core_asset_dynamic.rs\"]",
            "#[path = \"asset_budget_tests/naming_graphics_misc.rs\"]",
            "#[path = \"asset_budget_tests/row_data_owner.rs\"]",
            "runtime_rhi::EXPECTED_STATUS_OUTPUT_SLICES",
            "asset_tests::EXPECTED_STATUS_OUTPUT_SLICES",
            "budget_render_ui::EXPECTED_STATUS_OUTPUT_SLICES",
            "naming_core_asset_dynamic::EXPECTED_STATUS_OUTPUT_SLICES",
            "naming_graphics_misc::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !asset_budget.contains(
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &["
        ),
        "asset_budget_tests.rs should route child row-data owners instead of owning row tuples directly"
    );
    assert_contains_all(
        "Runtime 15 asset-budget row-data children own representative rows",
        &row_children,
        &[
            "Runtime 15 M3 runtime diagnostics test folder split",
            "Runtime 15 M3 asset pack test folder split",
            "Runtime 15 M3 no oversized test files global gate",
            "Runtime 15 M3 core-framework naming camera-controller guard child-owner split",
            "Runtime 15 M3 graphics render-framework receiver guard child-owner split",
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
        ],
    );
}
