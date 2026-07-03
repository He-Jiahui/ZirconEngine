use super::*;

#[test]
fn runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_root.join("src/dynamic_api/session/tests");
    let retired_helpers = tests_dir.join("helpers.rs");
    let tests_mod = read_text(
        &tests_dir.join("mod.rs"),
        "dynamic API session tests module parent should be readable",
    );
    let vampire_runtime_support = read_text(
        &tests_dir.join("vampire_runtime_support.rs"),
        "dynamic API vampire runtime support owner should be readable",
    );
    let frame_diagnostics = read_text(
        &tests_dir.join("frame_diagnostics.rs"),
        "dynamic API frame diagnostics tests should be readable",
    );
    let vampire_gameplay = read_text(
        &tests_dir.join("vampire_gameplay.rs"),
        "dynamic API vampire gameplay tests should be readable",
    );
    let vampire_hud = read_text(
        &tests_dir.join("vampire_hud.rs"),
        "dynamic API vampire HUD tests should be readable",
    );
    let vampire_menu = read_text(
        &tests_dir.join("vampire_menu.rs"),
        "dynamic API vampire menu tests should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let dynamic_api_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/dynamic_api/session.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

    assert!(
        !retired_helpers.exists(),
        "dynamic API session tests should not keep banned-name module file {:?}",
        retired_helpers
    );
    assert_contains_all(
        "dynamic API session tests module parent",
        &tests_mod,
        &["mod vampire_runtime_support;"],
    );
    assert!(
        !tests_mod.contains("mod helpers;"),
        "dynamic_api/session/tests/mod.rs should not preserve the banned helpers module name"
    );
    assert_contains_all(
        "dynamic API vampire runtime support owner",
        &vampire_runtime_support,
        &[
            "fn vampire_project_config",
            "fn start_vampire_game",
            "fn count_hud_panel_pixels",
            "fn diagnostic_current",
            "fn small_headless_frame_request",
        ],
    );

    for (label, source) in [
        ("frame diagnostics tests", frame_diagnostics.as_str()),
        ("vampire gameplay tests", vampire_gameplay.as_str()),
        ("vampire HUD tests", vampire_hud.as_str()),
        ("vampire menu tests", vampire_menu.as_str()),
    ] {
        assert_contains_all(label, source, &["super::vampire_runtime_support::*"]);
        assert!(
            !source.contains("super::helpers::*"),
            "{label} should not import the retired helpers owner"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("dynamic API session doc", dynamic_api_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 dynamic API vampire runtime support module naming hard cutover",
                "runtime_15_dynamic_api_vampire_runtime_support_naming_hard_cutover_static_passed_cargo_deferred",
                "dynamic_api/session/tests/vampire_runtime_support.rs",
                "runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name",
            ],
        );
    }
}
