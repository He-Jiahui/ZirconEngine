use super::*;

#[test]
fn runtime_15_priority_plan_docs_test_paths_stay_current() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_test_paths_exist(label, path, &source);
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}

fn assert_priority_plan_doc_test_paths_exist(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let mut checked_paths = 0usize;
    let mut active_section_is_tests = false;

    for line in frontmatter {
        if !line.starts_with(' ') {
            active_section_is_tests = line
                .split_once(':')
                .map(|(section, _)| section == "tests")
                .unwrap_or(false);
            continue;
        }

        if !active_section_is_tests {
            continue;
        }

        let Some(item) = line.strip_prefix("  - ") else {
            continue;
        };
        let candidate = item
            .split("::")
            .next()
            .unwrap_or(item)
            .split(": ")
            .next()
            .unwrap_or(item)
            .trim();
        if !is_repo_path_like_test_entry(candidate) {
            continue;
        }

        assert!(
            repo_path(candidate).exists(),
            "{label} frontmatter `tests` entry should resolve to an existing repository path: {candidate}"
        );
        checked_paths += 1;
    }

    assert!(
        checked_paths > 0,
        "{label} priority plan doc `{path}` should expose at least one path-like tests entry"
    );
}

fn is_repo_path_like_test_entry(candidate: &str) -> bool {
    candidate.starts_with("zircon_")
        || candidate.starts_with("tools/")
        || candidate.starts_with("tools\\")
        || candidate.starts_with("docs/")
        || candidate.starts_with("docs\\")
        || candidate.starts_with(".github/")
        || candidate.starts_with(".github\\")
        || candidate.starts_with(".codex/")
        || candidate.starts_with(".codex\\")
}
