use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_source_inventory_is_child_owner() {
    let parent = read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD);
    let source_inventory_parent = read_runtime_src(SOURCE_INVENTORY_CHILD);
    let child_sources = source_inventory_child_source_blob();
    let sources = code_review_findings_sources();

    assert_contains_all(
        "folder-backed summary parent delegates source inventory to child owner",
        &parent,
        &[
            "#[path = \"folder_backed_summary/source_inventory.rs\"]",
            "mod source_inventory;",
            "source_inventory::code_review_findings_sources",
            "source_inventory::assert_code_review_findings_line_budgets",
            "direct_review_guard_count",
        ],
    );
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
        "folder-backed source inventory parent mounts focused children",
        &source_inventory_parent,
        &[
            "#[path = \"source_inventory/model.rs\"]",
            "mod model;",
            "#[path = \"source_inventory/reads.rs\"]",
            "mod reads;",
            "#[path = \"source_inventory/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"source_inventory/delegation.rs\"]",
            "mod delegation;",
            SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_NAME,
            SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_ID,
        ],
    );
    for (_, child_path, anchor) in SOURCE_INVENTORY_CHILDREN {
        assert!(
            source_inventory_parent.contains(child_path),
            "source inventory parent should inventory child path {child_path}"
        );
        assert!(
            child_sources.contains(anchor),
            "source inventory child {child_path} should own anchor {anchor}"
        );
    }
    for moved_anchor in [
        "parent: read_runtime_src",
        "sources.parent.as_str()",
        "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/scaffold.rs",
        "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/sdk_macro_manifest.rs",
    ] {
        assert!(
            !source_inventory_parent.contains(moved_anchor),
            "source inventory implementation anchor `{moved_anchor}` should stay in focused children"
        );
        assert!(
            child_sources.contains(moved_anchor),
            "source inventory children should own implementation anchor `{moved_anchor}`"
        );
    }
    assert_eq!(
        sources.direct_review_guard_count(),
        15,
        "folder-backed source inventory should preserve the 15 directly counted review guards"
    );
    assert_code_review_findings_line_budgets(&sources);

    let mut budget_sources = vec![(FOLDER_BACKED_SUMMARY_CHILD, parent)];
    budget_sources.extend(source_inventory_child_sources());
    budget_sources.push((SOURCE_INVENTORY_CHILD, source_inventory_parent));
    for (path, source) in budget_sources {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
