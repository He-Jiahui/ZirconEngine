use super::super::repo_path;

pub(super) const AUDIT_ROOT: &str =
    ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts";

pub(super) const CORE_DOCS: &[&str] = &[
    "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    "docs/zircon_runtime/structure/module-convention.md",
];

pub(super) const CORE_DOCS_WITH_SESSION: &[&str] = &[
    "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    "docs/zircon_runtime/structure/module-convention.md",
];

pub(super) fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}

pub(super) fn assert_not_contains(label: &str, source: &str, forbidden: &[&str]) {
    let present: Vec<_> = forbidden
        .iter()
        .copied()
        .filter(|anchor| source.contains(anchor))
        .collect();
    assert!(
        present.is_empty(),
        "{label} contains forbidden stale anchors: {present:?}"
    );
}

pub(super) fn frontmatter_section_entries<'a>(source: &'a str, section: &str) -> Vec<&'a str> {
    let mut in_frontmatter = false;
    let mut in_section = false;
    let mut entries = Vec::new();

    for line in source.lines() {
        if line == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if !in_frontmatter {
            continue;
        }
        if let Some(section_name) = line.strip_suffix(':') {
            in_section = section_name == section;
            continue;
        }
        if in_section {
            if let Some(entry) = line.trim_start().strip_prefix("- ") {
                entries.push(entry.trim());
            }
        }
    }

    entries
}

pub(super) fn assert_frontmatter_section_has_unique_entries(
    label: &str,
    source: &str,
    section: &str,
) {
    let entries = frontmatter_section_entries(source, section);
    let mut duplicates = Vec::new();

    for (index, entry) in entries.iter().enumerate() {
        if entries[..index].contains(entry) {
            duplicates.push(*entry);
        }
    }

    assert!(
        duplicates.is_empty(),
        "{label} frontmatter section `{section}` contains duplicate entries: {duplicates:?}"
    );
}
