use super::super::super::super::*;
use super::*;

fn folder_backed_summary_source_inventory_child_tree() -> String {
    [
        read_runtime_src(FOLDER_BACKED_SUMMARY_SOURCE_INVENTORY_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_SOURCE_MODEL_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_SOURCE_READS_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_SOURCE_BUDGETS_CHILD_OWNER),
        read_runtime_src(FOLDER_BACKED_SUMMARY_SOURCE_DELEGATION_CHILD_OWNER),
    ]
    .join("\n")
}

pub(super) fn assert_folder_backed_summary_source_inventory_is_current() {
    let parent = read_runtime_src(FOLDER_BACKED_SUMMARY_STRUCTURE_CHILD_OWNER);
    let child_tree = folder_backed_summary_source_inventory_child_tree();

    for source_inventory_guard in [
        "folder-backed summary source inventory child keeps source-path and count checks",
        "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/scaffold.rs",
        "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/sdk_macro_manifest.rs",
        "15 directly counted review guards",
    ] {
        assert!(
            !parent.contains(source_inventory_guard),
            "source inventory structure guard `{source_inventory_guard}` should stay in {FOLDER_BACKED_SUMMARY_STRUCTURE_SOURCE_INVENTORY_CHILD_OWNER}"
        );
    }
    assert_contains_all(
        "folder-backed summary source inventory child keeps source-path and count checks",
        &child_tree,
        &[
            "fn runtime_15_code_review_findings_source_inventory_is_child_owner",
            "struct CodeReviewFindingsSources",
            "pub(super) fn code_review_findings_sources",
            "pub(super) fn assert_code_review_findings_line_budgets",
            "fn direct_review_guard_count",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/scaffold.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/sdk_macro_manifest.rs",
            "tests/runtime_absorption/code_review_findings/render_structure.rs",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
            "15 directly counted review guards",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_structure_guard_folder_backed_summary_source_inventory_is_child_owned(
) {
    assert_folder_backed_summary_source_inventory_is_current();
}
