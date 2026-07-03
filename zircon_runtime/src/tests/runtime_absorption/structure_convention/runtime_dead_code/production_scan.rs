use super::super::assert_contains_all;
use super::{
    collect_production_rust_sources, dead_code_suppression_lines, read_repo, read_runtime_src,
    runtime_source_path,
};

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

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M5 production dead-code suppression global gate",
                "runtime_15_production_dead_code_suppression_global_gate_static_passed_cargo_deferred",
                "runtime_15_production_sources_do_not_allow_dead_code_suppression",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status map",
        &status_map,
        &[
            "Runtime 15 M5 production dead-code suppression global gate",
            "runtime_15_production_dead_code_suppression_global_gate_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 date map",
        &date_map,
        &["Runtime 15 M5 production dead-code suppression global gate"],
    );
}
