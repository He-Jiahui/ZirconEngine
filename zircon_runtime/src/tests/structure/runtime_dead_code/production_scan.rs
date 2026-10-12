use super::{collect_production_rust_sources, dead_code_suppression_lines, runtime_source_path};

#[test]
fn runtime_15_production_sources_do_not_allow_dead_code_suppression() {
    let src_root = runtime_source_path("");
    let mut production_sources = Vec::new();
    collect_production_rust_sources(&src_root, &src_root, &mut production_sources);
    production_sources.sort();

    assert!(
        production_sources.len() > 100,
        "production dead-code scan should cover the runtime source tree; got {} files",
        production_sources.len()
    );

    let mut violations = Vec::new();
    for path in &production_sources {
        let source = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read production source `{path:?}`: {error}"));
        let suppression_lines = dead_code_suppression_lines(&source);
        if !suppression_lines.is_empty() {
            let relative = path
                .strip_prefix(&src_root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            violations.push(format!("{relative}: {suppression_lines:?}"));
        }
    }

    assert!(
        violations.is_empty(),
        "production runtime sources should not use dead-code suppression: {violations:?}"
    );
}
