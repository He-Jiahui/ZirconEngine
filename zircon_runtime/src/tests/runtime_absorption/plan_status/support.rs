pub(super) fn frontmatter_status(source: &str) -> Option<&str> {
    frontmatter_value(source, "status:")
}

pub(super) fn frontmatter_last_refined(source: &str) -> Option<&str> {
    frontmatter_value(source, "last_refined:")
}

fn frontmatter_value<'a>(source: &'a str, prefix: &str) -> Option<&'a str> {
    let mut lines = source.lines();
    if lines.next() != Some("---") {
        return None;
    }

    for line in lines {
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix(prefix) {
            return Some(value.trim());
        }
    }

    None
}

fn runtime_plan_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest should live under the repository root")
        .join("docs")
        .join("plans")
        .join("zircon_runtime")
        .join("runtime")
}

pub(super) fn runtime_subplan_sources() -> Vec<(String, String)> {
    let plan_dir = runtime_plan_dir();
    let mut sources = Vec::new();

    for entry in std::fs::read_dir(&plan_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", plan_dir.display()))
    {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read runtime plan entry: {error}"));
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("md")
            || path.file_name().and_then(|name| name.to_str()) == Some("index.md")
        {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| panic!("runtime plan path should be utf-8: {}", path.display()))
            .to_owned();
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        sources.push((filename, source));
    }

    sources.sort_by(|left, right| left.0.cmp(&right.0));
    sources
}

pub(super) fn max_iso_date(source: &str) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut max_date = None;

    for start in 0..bytes.len().saturating_sub(9) {
        let candidate_bytes = &bytes[start..start + 10];
        let is_iso_date = candidate_bytes[0..4].iter().all(u8::is_ascii_digit)
            && candidate_bytes[4] == b'-'
            && candidate_bytes[5..7].iter().all(u8::is_ascii_digit)
            && candidate_bytes[7] == b'-'
            && candidate_bytes[8..10].iter().all(u8::is_ascii_digit);
        if is_iso_date {
            let candidate = std::str::from_utf8(candidate_bytes)
                .expect("ASCII date candidate should be valid utf-8");
            if match max_date {
                Some(date) => candidate > date,
                None => true,
            } {
                max_date = Some(candidate);
            }
        }
    }

    max_date
}

pub(super) fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should keep runtime plan-status anchor `{anchor}`"
        );
    }
}

pub(super) fn runtime_index_row_for<'a>(index_source: &'a str, filename: &str) -> &'a str {
    let filename_cell = format!("| `{filename}` |");
    index_source
        .lines()
        .find(|line| line.contains(&filename_cell))
        .unwrap_or_else(|| panic!("runtime index should include subplan row for `{filename}`"))
}

pub(super) fn index_section_between<'a>(
    source: &'a str,
    start_anchor: &str,
    end_anchor: &str,
) -> &'a str {
    let start = source
        .find(start_anchor)
        .unwrap_or_else(|| panic!("runtime index should include section `{start_anchor}`"));
    let section = &source[start..];
    section
        .find(end_anchor)
        .map(|end| &section[..end])
        .unwrap_or(section)
}

pub(super) fn first_backtick_value(source: &str) -> Option<&str> {
    let (_, tail) = source.split_once('`')?;
    let (value, _) = tail.split_once('`')?;
    Some(value)
}

pub(super) fn leading_plan_id(source: &str) -> Option<&str> {
    let bytes = source.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_digit() && bytes[1].is_ascii_digit() {
        Some(&source[..2])
    } else {
        None
    }
}

pub(super) fn referenced_plan_ids(source: &str) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut ids = Vec::new();

    for start in 0..bytes.len().saturating_sub(1) {
        let candidate = &bytes[start..start + 2];
        if candidate.iter().all(u8::is_ascii_digit) {
            let id = std::str::from_utf8(candidate)
                .expect("ASCII plan id candidate should be valid utf-8");
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }

    ids
}

pub(super) fn runtime_absorption_guard_modules() -> Vec<&'static str> {
    include_str!("../mod.rs")
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("mod ")
                .and_then(|module| module.strip_suffix(';'))
        })
        .collect()
}

pub(super) fn runtime_absorption_plan_status_support_files() -> Vec<String> {
    let tests_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests");
    let plan_status_dir = tests_root.join("runtime_absorption").join("plan_status");
    let mut files = Vec::new();

    collect_rust_files_relative_to(&plan_status_dir, &tests_root, &mut files);
    files.sort();
    files
}

fn collect_rust_files_relative_to(
    directory: &std::path::Path,
    relative_root: &std::path::Path,
    files: &mut Vec<String>,
) {
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .map(|entry| entry.unwrap_or_else(|error| panic!("failed to read source entry: {error}")))
        .collect();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files_relative_to(&path, relative_root, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let relative_path = path.strip_prefix(relative_root).unwrap_or_else(|error| {
                panic!(
                    "failed to make {} relative to {}: {error}",
                    path.display(),
                    relative_root.display()
                )
            });
            let anchor = relative_path
                .components()
                .map(|component| {
                    component.as_os_str().to_str().unwrap_or_else(|| {
                        panic!("source path should be utf-8: {}", path.display())
                    })
                })
                .collect::<Vec<_>>()
                .join("/");
            files.push(anchor);
        }
    }
}

pub(super) fn markdown_frontmatter_and_body(source: &str) -> (&str, &str) {
    let source = source
        .strip_prefix("---")
        .expect("markdown document should start with YAML frontmatter");
    let frontmatter_end = source
        .find("\n---")
        .expect("markdown document should close YAML frontmatter");
    let frontmatter = &source[..frontmatter_end];
    let body = &source[frontmatter_end + "\n---".len()..];
    (frontmatter, body)
}

pub(super) fn markdown_table_cells(row: &str) -> Vec<&str> {
    row.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}
