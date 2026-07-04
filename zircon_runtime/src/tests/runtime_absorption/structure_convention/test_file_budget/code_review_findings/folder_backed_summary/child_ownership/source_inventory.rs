use super::super::super::super::*;
use super::*;

pub(super) fn assert_folder_backed_source_inventory_child_is_current(parent: &str) {
    let source_inventory_child_sources = source_inventory::source_inventory_child_source_blob();

    for source_inventory_guard in [
        concat!("let ", "f8_api_convergence ="),
        concat!("let ", "p0_robustness ="),
        concat!(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/",
            "descriptor_builder/scaffold.rs"
        ),
        concat!(
            "tests/runtime_absorption/code_review_findings/p0_robustness/",
            "native_fixture/sdk_macro_manifest.rs"
        ),
    ] {
        assert!(
            !parent.contains(source_inventory_guard),
            "source inventory guard `{source_inventory_guard}` should stay in {SOURCE_INVENTORY_CHILD}"
        );
    }
    assert_contains_all(
        "folder-backed source inventory child owns source reads and helper counts",
        &source_inventory_child_sources,
        &[
            "fn runtime_15_code_review_findings_source_inventory_is_child_owner",
            "struct CodeReviewFindingsSources",
            "pub(super) fn code_review_findings_sources",
            "pub(super) fn assert_code_review_findings_line_budgets",
            "fn direct_review_guard_count",
            concat!(
                "tests/runtime_absorption/code_review_findings/f8_api_convergence/",
                "descriptor_builder/scaffold.rs"
            ),
            concat!(
                "tests/runtime_absorption/code_review_findings/p0_robustness/",
                "native_fixture/sdk_macro_manifest.rs"
            ),
            "tests/runtime_absorption/code_review_findings/render_structure.rs",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
        ],
    );
}
