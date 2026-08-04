use super::*;

#[test]
fn runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_frontmatter_entries_are_unique(label, path, &source);
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}

fn assert_priority_plan_doc_frontmatter_entries_are_unique(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);

    for section in [
        "related_code",
        "implementation_files",
        "plan_sources",
        "tests",
    ] {
        let entries = frontmatter_section_items(&frontmatter, section);
        let mut seen = std::collections::BTreeSet::new();
        let mut duplicates = Vec::new();

        for entry in entries {
            if !seen.insert(entry) {
                duplicates.push(entry);
            }
        }

        assert!(
            duplicates.is_empty(),
            "{label} priority plan doc `{path}` should keep `{section}:` duplicate-free; duplicates: {:?}",
            duplicates
        );
    }
}
