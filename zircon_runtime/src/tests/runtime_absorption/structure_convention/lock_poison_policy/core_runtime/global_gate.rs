use super::*;

#[test]
fn runtime_15_production_sources_do_not_directly_unwrap_mutex_locks() {
    let runtime_src = runtime_src_path("");
    let mut runtime_sources = Vec::new();
    collect_runtime_rust_sources(&runtime_src, &runtime_src, &mut runtime_sources);

    assert!(
        !runtime_sources.is_empty(),
        "runtime lock-poison global gate should scan runtime production sources"
    );

    let mut violations = Vec::new();
    for source_path in runtime_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read runtime source: {error}"));
        let production = production_section(&source);
        for (line_index, line) in production.lines().enumerate() {
            if line.contains(LOCK_UNWRAP_CALL) {
                violations.push(format!(
                    "{}:{}: {}",
                    runtime_source_display_path(&runtime_src, &source_path),
                    line_index + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "runtime production code should use poison-safe lock helpers instead of direct {LOCK_UNWRAP_CALL}:\n{}",
        violations.join("\n")
    );

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 production direct lock unwrap global gate",
                "runtime_15_production_direct_lock_unwrap_global_gate_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy/core_runtime.rs",
                "runtime_15_production_sources_do_not_directly_unwrap_mutex_locks",
            ],
        );
    }
}

fn collect_runtime_rust_sources(
    runtime_src: &std::path::Path,
    root: &std::path::Path,
    sources: &mut Vec<std::path::PathBuf>,
) {
    for entry in std::fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read runtime source directory: {error}"))
    {
        let path = entry
            .unwrap_or_else(|error| panic!("failed to read runtime source entry: {error}"))
            .path();
        if path.is_dir() {
            collect_runtime_rust_sources(runtime_src, &path, sources);
        } else if is_runtime_production_source(runtime_src, &path) {
            sources.push(path);
        }
    }
}

fn is_runtime_production_source(runtime_src: &std::path::Path, path: &std::path::Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return false;
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if file_name == "tests.rs" || file_name.ends_with("_tests.rs") {
        return false;
    }

    let Ok(relative) = path.strip_prefix(runtime_src) else {
        return false;
    };

    !relative
        .components()
        .any(|component| component.as_os_str().to_string_lossy() == "tests")
}

fn runtime_source_display_path(runtime_src: &std::path::Path, path: &std::path::Path) -> String {
    path.strip_prefix(runtime_src)
        .map(|relative| format!("zircon_runtime/src/{}", relative.display()).replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string().replace('\\', "/"))
}
