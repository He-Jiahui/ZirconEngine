use super::*;

mod code_paths;
mod frontmatter_uniqueness;
mod guard_tests;
mod header_sections;
mod plan_sources;
mod test_paths;

pub(super) const PRIORITY_PLAN_DOCS: &[(&str, &str)] = &[
    (
        "engine-code-structure-convention",
        "docs/plans/engine-code-structure-convention.md",
    ),
    (
        "engine-code-review-findings",
        "docs/plans/engine-code-review-findings-2026-06.md",
    ),
];

pub(super) const PRIORITY_PLAN_DOC_GUARDS: &[&str] = &[
    "runtime_15_priority_plan_docs_code_paths_stay_current",
    "runtime_15_priority_plan_docs_test_paths_stay_current",
    "runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
    "runtime_15_priority_plan_docs_required_header_sections_stay_complete",
    "runtime_15_priority_plan_docs_plan_sources_stay_cross_linked",
    "runtime_15_priority_plan_docs_guard_tests_stay_listed",
    "runtime_15_priority_plan_docs_guard_children_are_folder_backed",
    "runtime_15_priority_plan_docs_guard_test_children_are_folder_backed",
    "runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory",
    "runtime_15_priority_plan_docs_moved_guard_paths_stay_current",
    "runtime_15_priority_plan_docs_moved_mirror_names_full_inventory",
    "runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources",
    "runtime_15_priority_plan_docs_listing_prose_names_full_inventory",
];

pub(super) fn frontmatter_scalar<'a>(frontmatter: &'a [&'a str], key: &str) -> Option<&'a str> {
    let prefix = format!("{key}:");
    frontmatter
        .iter()
        .find_map(|line| line.strip_prefix(&prefix).map(str::trim))
        .filter(|value| !value.is_empty())
}

pub(super) fn frontmatter_section_items<'a>(
    frontmatter: &'a [&'a str],
    section: &str,
) -> Vec<&'a str> {
    let mut active_section = false;
    let mut items = Vec::new();

    for line in frontmatter {
        if !line.starts_with(' ') {
            active_section = line
                .split_once(':')
                .map(|(candidate, _)| candidate == section)
                .unwrap_or(false);
            continue;
        }

        if active_section {
            let trimmed = line.trim_start();
            if let Some(item) = trimmed.strip_prefix("- ") {
                items.push(item.trim());
            }
        }
    }

    items
}

pub(super) fn frontmatter_lines<'a>(label: &str, path: &str, source: &'a str) -> Vec<&'a str> {
    let mut lines = source.lines();
    assert_eq!(
        lines.next(),
        Some("---"),
        "{label} priority plan doc `{path}` should start with YAML frontmatter"
    );

    let mut frontmatter = Vec::new();
    for line in lines {
        if line == "---" {
            return frontmatter;
        }
        frontmatter.push(line);
    }

    panic!("{label} priority plan doc `{path}` should close YAML frontmatter");
}

pub(super) fn runtime_index_with_output_archive_source() -> String {
    let mut source = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    source.push('\n');
    source.push_str(priority_plan_doc_current_owner_archive_source());
    source
}

pub(super) fn priority_plan_doc_current_owner_archive_source() -> &'static str {
    crate::tests::runtime_absorption::structure_convention::priority_plan_doc_current_owner_archive_source()
}
