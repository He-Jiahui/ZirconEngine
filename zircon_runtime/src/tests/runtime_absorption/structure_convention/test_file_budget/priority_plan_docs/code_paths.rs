use super::*;

#[test]
fn runtime_15_priority_plan_docs_code_paths_stay_current() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_code_paths_exist(label, path, &source);
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}

fn assert_priority_plan_doc_code_paths_exist(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let mut checked_paths = 0usize;
    let mut checked_sections = std::collections::BTreeSet::new();
    let mut active_section: Option<&str> = None;

    for line in frontmatter {
        if !line.starts_with(' ') {
            active_section = line
                .split_once(':')
                .map(|(section, _)| section)
                .filter(|section| matches!(*section, "related_code" | "implementation_files"));
            continue;
        }

        let Some(section) = active_section else {
            continue;
        };
        let Some(item) = line.strip_prefix("  - ") else {
            continue;
        };
        let candidate = item.split("::").next().unwrap_or(item).trim();
        if candidate.is_empty()
            || candidate.starts_with("user:")
            || candidate.starts_with("http://")
            || candidate.starts_with("https://")
        {
            continue;
        }

        assert!(
            repo_path(candidate).exists(),
            "{label} frontmatter `{section}` entry should resolve to an existing repository path: {candidate}"
        );
        checked_paths += 1;
        checked_sections.insert(section);
    }

    assert!(
        checked_sections.contains("related_code"),
        "{label} priority plan doc `{path}` should expose at least one related_code path"
    );
    assert!(
        checked_paths > 0,
        "{label} priority plan doc `{path}` should expose code-facing frontmatter paths"
    );
}
