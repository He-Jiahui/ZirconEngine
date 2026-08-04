use super::*;

#[test]
fn runtime_15_priority_plan_docs_guard_tests_stay_listed() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_tests_list_guard_functions(label, path, &source);
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}

fn assert_priority_plan_doc_tests_list_guard_functions(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let test_entries = frontmatter_section_items(&frontmatter, "tests");

    for guard in PRIORITY_PLAN_DOC_GUARDS {
        let expected_prefix = "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/";
        assert!(
            test_entries
                .iter()
                .any(|entry| entry.starts_with(expected_prefix) && entry.contains(guard)),
            "{label} priority plan doc `{path}` should list priority-plan-doc guard `{guard}` in frontmatter tests"
        );
    }
}
