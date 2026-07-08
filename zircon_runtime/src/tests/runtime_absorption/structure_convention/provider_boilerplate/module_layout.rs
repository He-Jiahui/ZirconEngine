use super::*;

#[test]
fn runtime_15_provider_boilerplate_guard_child_owner_split() {
    let structure_parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let provider_parent =
        read_runtime_src("tests/runtime_absorption/structure_convention/provider_boilerplate.rs");
    let prepare_input = read_runtime_src(
        "tests/runtime_absorption/structure_convention/provider_boilerplate/prepare_input.rs",
    );
    let registration = read_runtime_src(
        "tests/runtime_absorption/structure_convention/provider_boilerplate/registration.rs",
    );
    let update = read_runtime_src(
        "tests/runtime_absorption/structure_convention/provider_boilerplate/update.rs",
    );
    let feedback = read_runtime_src(
        "tests/runtime_absorption/structure_convention/provider_boilerplate/feedback.rs",
    );
    let full_audit = read_runtime_src(
        "tests/runtime_absorption/structure_convention/provider_boilerplate/full_audit.rs",
    );
    let module_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/provider_boilerplate/module_layout.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );
    let status_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/dead_code_guard_maps.rs",
        ),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/dead_code_guard_maps.rs",
        ),
    ]
    .join("\n");

    assert_contains_all(
        "structure convention provider child module mount",
        &structure_parent,
        &[
            "#[path = \"structure_convention/provider_boilerplate.rs\"]",
            "mod provider_boilerplate;",
        ],
    );
    assert_contains_all(
        "provider boilerplate parent mounts child guard owners",
        &provider_parent,
        &[
            "#[path = \"provider_boilerplate/prepare_input.rs\"]",
            "mod prepare_input;",
            "#[path = \"provider_boilerplate/registration.rs\"]",
            "mod registration;",
            "#[path = \"provider_boilerplate/update.rs\"]",
            "mod update;",
            "#[path = \"provider_boilerplate/feedback.rs\"]",
            "mod feedback;",
            "#[path = \"provider_boilerplate/full_audit.rs\"]",
            "mod full_audit;",
            "#[path = \"provider_boilerplate/module_layout.rs\"]",
            "mod module_layout;",
        ],
    );

    for moved_guard in [
        "fn runtime_15_provider_prepare_input_uses_shared_extract_generation_owner",
        "fn runtime_15_provider_registration_uses_shared_owner",
        "fn runtime_15_provider_update_uses_shared_stats_owner",
        "fn runtime_15_provider_feedback_uses_shared_payload_owner",
        "fn runtime_15_no_duplicated_provider_boilerplate",
    ] {
        assert!(
            !structure_parent.contains(moved_guard),
            "provider boilerplate guard `{moved_guard}` should not live in structure_convention.rs"
        );
        assert!(
            !provider_parent.contains(moved_guard),
            "provider_boilerplate.rs should mount child guard owners instead of defining `{moved_guard}`"
        );
    }
    assert_contains_all(
        "provider prepare-input child owns prepare guard",
        &prepare_input,
        &["fn runtime_15_provider_prepare_input_uses_shared_extract_generation_owner"],
    );
    assert_contains_all(
        "provider registration child owns registration guard",
        &registration,
        &["fn runtime_15_provider_registration_uses_shared_owner"],
    );
    assert_contains_all(
        "provider update child owns update guard",
        &update,
        &["fn runtime_15_provider_update_uses_shared_stats_owner"],
    );
    assert_contains_all(
        "provider feedback child owns feedback guard",
        &feedback,
        &["fn runtime_15_provider_feedback_uses_shared_payload_owner"],
    );
    assert_contains_all(
        "provider full-audit child owns aggregate guard",
        &full_audit,
        &["fn runtime_15_no_duplicated_provider_boilerplate"],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/structure_convention.rs",
            structure_parent.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/provider_boilerplate.rs",
            provider_parent.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/provider_boilerplate/prepare_input.rs",
            prepare_input.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/provider_boilerplate/registration.rs",
            registration.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/provider_boilerplate/update.rs",
            update.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/provider_boilerplate/feedback.rs",
            feedback.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/provider_boilerplate/full_audit.rs",
            full_audit.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/provider_boilerplate/module_layout.rs",
            module_layout.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Runtime 15 status rows record provider guard child-owner split",
        &status_rows,
        &[
            "Runtime 15 M3 provider boilerplate guard child-owner split",
            "runtime_15_provider_boilerplate_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/provider_boilerplate.rs",
            "structure_convention/provider_boilerplate/module_layout.rs",
            "structure_convention/provider_boilerplate/full_audit.rs",
            "runtime_15_provider_boilerplate_guard_child_owner_split",
        ],
    );
    assert_contains_all(
        "Runtime 15 status/date maps record provider guard child-owner split",
        &format!("{status_map}\n{date_map}"),
        &[
            "Runtime 15 M3 provider boilerplate guard child-owner split",
            "runtime_15_provider_boilerplate_guard_child_owner_split_static_passed_cargo_deferred",
            "Some(\"2026-06-24\")",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 provider boilerplate guard child-owner split",
                "runtime_15_provider_boilerplate_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/provider_boilerplate.rs",
                "structure_convention/provider_boilerplate/module_layout.rs",
                "structure_convention/provider_boilerplate/full_audit.rs",
                "runtime_15_provider_boilerplate_guard_child_owner_split",
            ],
        );
    }
}
