use super::*;

#[test]
fn runtime_15_priority_plan_docs_plan_sources_stay_cross_linked() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_plan_sources_are_cross_linked(label, path, &source);
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}

fn assert_priority_plan_doc_plan_sources_are_cross_linked(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let plan_sources = frontmatter_section_items(&frontmatter, "plan_sources");
    assert!(
        plan_sources.iter().any(|item| item.starts_with("user:")),
        "{label} priority plan doc `{path}` should keep a user-request plan source"
    );
    assert!(
        plan_sources.iter().any(|item| *item
            == "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"),
        "{label} priority plan doc `{path}` should cite Runtime 15 as its implementation-status source"
    );

    let required_companion = match path {
        "docs/plans/engine-code-structure-convention.md" => {
            "docs/plans/engine-code-review-findings-2026-06.md"
        }
        "docs/plans/engine-code-review-findings-2026-06.md" => {
            "docs/plans/engine-code-structure-convention.md"
        }
        _ => panic!("unexpected priority plan doc path `{path}`"),
    };
    assert!(
        plan_sources.iter().any(|item| *item == required_companion),
        "{label} priority plan doc `{path}` should cite companion priority plan source `{required_companion}`"
    );
}
