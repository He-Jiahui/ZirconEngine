use super::*;

#[test]
fn runtime_15_priority_plan_docs_required_header_sections_stay_complete() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_required_header_sections(label, path, &source);
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}

fn assert_priority_plan_doc_required_header_sections(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let required_order = [
        "related_code",
        "implementation_files",
        "plan_sources",
        "tests",
        "doc_type",
        "status",
    ];
    let top_level_keys: Vec<&str> = frontmatter
        .iter()
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_once(':').map(|(key, _)| key))
        .collect();

    for key in required_order {
        assert!(
            top_level_keys.contains(&key),
            "{label} priority plan doc `{path}` should keep `{key}:` in frontmatter"
        );
    }

    let required_positions: Vec<usize> = required_order
        .iter()
        .map(|key| {
            top_level_keys
                .iter()
                .position(|candidate| candidate == key)
                .unwrap_or_else(|| {
                    panic!("{label} priority plan doc `{path}` missing required key `{key}:`")
                })
        })
        .collect();
    assert!(
        required_positions
            .windows(2)
            .all(|window| window[0] < window[1]),
        "{label} priority plan doc `{path}` should keep required frontmatter keys in docs lookup order: {:?}",
        required_order
    );

    for section in [
        "related_code",
        "implementation_files",
        "plan_sources",
        "tests",
    ] {
        let item_count = frontmatter_section_items(&frontmatter, section).len();
        assert!(
            item_count > 0,
            "{label} priority plan doc `{path}` should keep non-empty `{section}:` frontmatter"
        );
    }

    for scalar in ["doc_type", "status"] {
        let value = frontmatter_scalar(&frontmatter, scalar);
        assert!(
            value.is_some(),
            "{label} priority plan doc `{path}` should keep non-empty `{scalar}:` frontmatter"
        );
    }
}
