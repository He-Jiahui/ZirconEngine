use super::super::assert_contains_all;
use super::super::support::assert_contains_all_exact;
use super::{read_repo, read_runtime_src, runtime_source_path, DEAD_CODE_ALLOW_ATTRIBUTE};

#[test]
fn runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed() {
    let root =
        read_runtime_src("tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs");
    let child_sources = runtime_dead_code_child_sources();
    let runtime_15_plan_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert!(
        !child_sources.contains(DEAD_CODE_ALLOW_ATTRIBUTE),
        "runtime dead-code child guards should not embed the forbidden attribute as a source literal"
    );
    assert_contains_all(
        "runtime dead-code guard constant-backed forbidden attribute",
        &root,
        &[
            "const DEAD_CODE_ALLOW_ATTRIBUTE: &str = concat!(\"#[allow(\", \"dead_code\", \")]\");",
            "const DEAD_CODE_ALLOW_CALL_PREFIX: &str = concat!(\"allow(\", \"dead_code\");",
        ],
    );
    assert_contains_all(
        "runtime dead-code child guards use the shared constant",
        &child_sources,
        &[
            "!ui_mod.contains(DEAD_CODE_ALLOW_ATTRIBUTE)",
            "!builtin_host_modules.contains(DEAD_CODE_ALLOW_ATTRIBUTE)",
            "runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
        ],
    );

    for (label, source) in [
        (
            "Runtime 15 archived output",
            runtime_15_plan_output.as_str(),
        ),
        (
            "runtime index archived output",
            runtime_index_output.as_str(),
        ),
        (
            "review findings archived output",
            review_findings_output.as_str(),
        ),
        (
            "structure convention archived output",
            structure_convention_output.as_str(),
        ),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all_exact(
            label,
            source,
            &[
                "Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup",
                "runtime_15_runtime_dead_code_guard_literal_cleanup_static_passed_cargo_deferred",
                "runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
            ],
        );
    }
}

#[test]
fn runtime_15_runtime_dead_code_guard_is_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let root =
        read_runtime_src("tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs");
    let child_sources = runtime_dead_code_child_sources();
    let runtime_15_plan_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "structure convention parent runtime dead-code mount",
        &parent,
        &[
            "#[path = \"structure_convention/runtime_dead_code/mod.rs\"]",
            "mod runtime_dead_code;",
        ],
    );
    assert!(
        !runtime_source_path("tests/runtime_absorption/structure_convention/runtime_dead_code.rs")
            .exists(),
        "old flat runtime_dead_code.rs guard owner should be deleted after folder-backed cutover"
    );

    for moved_guard in [
        "fn runtime_15_runtime_ui_dead_code_surface_is_test_support",
        "fn runtime_15_runtime_owned_dead_code_suppression_cleanup",
        "fn runtime_15_ui_text_edit_state_dead_code_suppression_cleanup",
        "fn runtime_15_script_host_value_descriptors_do_not_suppress_dead_code",
        "fn runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code",
        "fn runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
        "fn runtime_15_production_sources_do_not_allow_dead_code_suppression",
    ] {
        assert!(
            !parent.contains(moved_guard) && !root.contains(moved_guard),
            "runtime dead-code root wiring should mount child guards instead of defining {moved_guard}"
        );
        assert!(
            child_sources.contains(moved_guard),
            "runtime dead-code child owners should preserve moved guard {moved_guard}"
        );
    }

    let parent_lines = parent.lines().count();
    assert!(
        parent_lines < 180,
        "structure_convention.rs should remain a thin aggregator after runtime dead-code split; got {parent_lines} lines"
    );
    let root_lines = root.lines().count();
    assert!(
        root_lines < 120,
        "runtime_dead_code/mod.rs should stay a thin support owner; got {root_lines} lines"
    );
    for (path, source) in runtime_dead_code_child_source_list() {
        let line_count = source.lines().count();
        assert!(
            line_count < 260,
            "{path} should stay below the Runtime 15 child guard budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        (
            "Runtime 15 archived output",
            runtime_15_plan_output.as_str(),
        ),
        (
            "runtime index archived output",
            runtime_index_output.as_str(),
        ),
        (
            "review findings archived output",
            review_findings_output.as_str(),
        ),
        (
            "structure convention archived output",
            structure_convention_output.as_str(),
        ),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all_exact(
            label,
            source,
            &[
                "Runtime 15 M3 runtime dead-code guard module split",
                "runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked",
                "runtime_15_runtime_dead_code_guard_is_folder_backed",
                "runtime_15_runtime_ui_dead_code_surface_is_test_support",
            ],
        );
    }
}

#[test]
fn runtime_15_runtime_dead_code_guard_children_are_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let root =
        read_runtime_src("tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs");
    let child_sources = runtime_dead_code_child_sources();
    let runtime_15_plan_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention_output = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "runtime dead-code root mounts child owners",
        &root,
        &[
            "mod guard_layout;",
            "mod production_scan;",
            "mod runtime_owned;",
            "mod runtime_ui;",
            "mod script_host;",
            "mod status_anchor_cleanup;",
            "mod ui_text;",
        ],
    );
    assert!(
        !parent.contains("structure_convention/runtime_dead_code.rs"),
        "structure convention parent should not point at the retired flat runtime_dead_code.rs path"
    );
    for moved_guard in [
        "runtime_15_runtime_ui_dead_code_surface_is_test_support",
        "runtime_15_runtime_owned_dead_code_suppression_cleanup",
        "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup",
        "runtime_15_script_host_value_descriptors_do_not_suppress_dead_code",
        "runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code",
        "runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
        "runtime_15_production_sources_do_not_allow_dead_code_suppression",
    ] {
        assert!(
            child_sources.contains(moved_guard),
            "runtime dead-code child owners should preserve {moved_guard}"
        );
    }

    for (label, source) in [
        (
            "Runtime 15 archived output",
            runtime_15_plan_output.as_str(),
        ),
        (
            "runtime index archived output",
            runtime_index_output.as_str(),
        ),
        (
            "review findings archived output",
            review_findings_output.as_str(),
        ),
        (
            "structure convention archived output",
            structure_convention_output.as_str(),
        ),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all_exact(
            label,
            source,
            &[
                "Runtime 15 M3 runtime dead-code guard child-owner split",
                "runtime_15_runtime_dead_code_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/runtime_dead_code/mod.rs",
                "structure_convention/runtime_dead_code/runtime_ui.rs",
                "structure_convention/runtime_dead_code/production_scan.rs",
                "runtime_15_runtime_dead_code_guard_children_are_folder_backed",
            ],
        );
    }
}

fn runtime_dead_code_child_sources() -> String {
    runtime_dead_code_child_source_list()
        .into_iter()
        .map(|(_, source)| source)
        .collect::<Vec<_>>()
        .join("\n")
}

fn runtime_dead_code_child_source_list() -> Vec<(&'static str, String)> {
    [
        "tests/runtime_absorption/structure_convention/runtime_dead_code/guard_layout.rs",
        "tests/runtime_absorption/structure_convention/runtime_dead_code/production_scan.rs",
        "tests/runtime_absorption/structure_convention/runtime_dead_code/runtime_owned.rs",
        "tests/runtime_absorption/structure_convention/runtime_dead_code/runtime_ui.rs",
        "tests/runtime_absorption/structure_convention/runtime_dead_code/script_host.rs",
        "tests/runtime_absorption/structure_convention/runtime_dead_code/status_anchor_cleanup.rs",
        "tests/runtime_absorption/structure_convention/runtime_dead_code/status_anchor_cleanup/documentation.rs",
        "tests/runtime_absorption/structure_convention/runtime_dead_code/status_anchor_cleanup/f12_current_state.rs",
        "tests/runtime_absorption/structure_convention/runtime_dead_code/status_anchor_cleanup/gate_wording.rs",
        "tests/runtime_absorption/structure_convention/runtime_dead_code/ui_text.rs",
    ]
    .into_iter()
    .map(|path| (path, read_runtime_src(path)))
    .collect()
}
