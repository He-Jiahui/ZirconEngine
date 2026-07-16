const DEAD_CODE_ALLOW_CALL_PREFIX: &str = concat!("allow(", "dead_code");

#[test]
fn review_f12_runtime_production_dead_code_suppression_is_globally_gated() {
    let runtime_src_root = repo_root().join("zircon_runtime/src");
    let mut production_sources = Vec::new();
    collect_production_rust_sources(
        &runtime_src_root,
        &runtime_src_root,
        &mut production_sources,
    );
    production_sources.sort();

    assert!(
        production_sources.len() > 100,
        "F12 production suppression scan should cover the runtime source tree; got {} files",
        production_sources.len()
    );

    let mut violations = Vec::new();
    for path in &production_sources {
        let source = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read production source `{path:?}`: {error}"));
        let suppression_lines = dead_code_suppression_lines(&source);
        if !suppression_lines.is_empty() {
            let relative = path
                .strip_prefix(&runtime_src_root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            violations.push(format!("{relative}: {suppression_lines:?}"));
        }
    }

    assert!(
        violations.is_empty(),
        "F12 runtime production sources should stay free of dead-code suppression: {violations:?}"
    );

    let review_findings = concat!(
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let f12_row = markdown_table_row(review_findings, "| F12 |");
    assert_contains_all(
        "F12 review row",
        f12_row,
        &[
            "production `allow(dead_code)` suppression",
            "Runtime 15 + Editor UI 10",
        ],
    );
    assert!(
        f12_row.ends_with("| Runtime 15 + Editor UI 10 |"),
        "F12 overview row should keep only the finding and delegated owners"
    );
    assert_contains_all(
        "F12 numbered output record",
        review_findings,
        &[
            "Runtime production `allow(dead_code)` sweep is globally gated",
            "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
            "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
            "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
        ],
    );
    assert!(
        !f12_row.contains("全 crate 200 命中"),
        "F12 top review row should not keep the stale full-crate 200-hit evidence as current runtime status"
    );

    assert_contains_all(
        "F12 numbered output record",
        review_findings,
        &[
            "Runtime 15 F12 dead-code review status sync",
            "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
            "Runtime 15 F12 dead-code runtime/editor boundary status guard",
            "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
            "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
        ],
    );
}

fn markdown_table_row<'a>(source: &'a str, prefix: &str) -> &'a str {
    source
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("missing markdown table row starting with `{prefix}`"))
}

fn dead_code_suppression_lines(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let compact: String = line.chars().filter(|ch| !ch.is_whitespace()).collect();
            if compact.contains(DEAD_CODE_ALLOW_CALL_PREFIX)
                && (compact.contains("#[") || compact.contains("#!["))
            {
                Some((index + 1, line.trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn collect_production_rust_sources(
    src_root: &std::path::Path,
    current_dir: &std::path::Path,
    sources: &mut Vec<std::path::PathBuf>,
) {
    for entry in std::fs::read_dir(current_dir)
        .unwrap_or_else(|error| panic!("failed to read directory `{current_dir:?}`: {error}"))
    {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read directory entry under `{current_dir:?}`: {error}")
        });
        let path = entry.path();
        if path.is_dir() {
            collect_production_rust_sources(src_root, &path, sources);
        } else if is_production_rust_source(src_root, &path) {
            sources.push(path);
        }
    }
}

fn is_production_rust_source(root: &std::path::Path, path: &std::path::Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return false;
    }

    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "tests.rs" || file_name.ends_with("_tests.rs") {
        return false;
    }

    let relative = path.strip_prefix(root).unwrap_or(path);
    !relative.components().any(|component| match component {
        std::path::Component::Normal(name) => name == std::ffi::OsStr::new("tests"),
        _ => false,
    })
}

fn repo_root() -> std::path::PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|error| panic!("failed to read cwd: {error}"));
    for candidate in cwd.ancestors() {
        if candidate.join("zircon_runtime/src").is_dir() {
            return candidate.to_path_buf();
        }
    }

    let source_path = cwd.join(file!());
    for candidate in source_path.ancestors() {
        if candidate.join("zircon_runtime/src").is_dir() {
            return candidate.to_path_buf();
        }
    }

    panic!("failed to locate repository root from cwd `{cwd:?}` and source `{source_path:?}`")
}

fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    let missing: Vec<&str> = anchors
        .iter()
        .copied()
        .filter(|anchor| !source.contains(anchor))
        .collect();
    assert!(missing.is_empty(), "{label} missing anchors: {missing:?}");
}
