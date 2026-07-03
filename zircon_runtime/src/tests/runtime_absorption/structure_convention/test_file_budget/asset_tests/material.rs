use super::*;

#[test]
fn runtime_15_asset_material_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/assets/material.rs");
    let asset_serialization =
        read_runtime_src("asset/tests/assets/material/asset_serialization.rs");
    let owned_descriptor = read_runtime_src("asset/tests/assets/material/owned_descriptor.rs");
    let override_validation =
        read_runtime_src("asset/tests/assets/material/override_validation.rs");
    let shader_readiness = read_runtime_src("asset/tests/assets/material/shader_readiness.rs");
    let management_records = read_runtime_src("asset/tests/assets/material/management_records.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );

    assert_contains_all(
        "asset material parent test module mounts",
        &parent,
        &[
            "mod asset_serialization;",
            "mod owned_descriptor;",
            "mod override_validation;",
            "mod shader_readiness;",
            "mod management_records;",
            "fn shader_contract",
            "fn asset_reference",
        ],
    );
    for moved_test in [
        "fn material_asset_zmaterial_roundtrip_maps_pbr_fields_to_shader_overrides",
        "fn material_owned_lighting_model_drives_standard_descriptor_without_shader_override",
        "fn material_owned_receive_shadows_reports_non_bool_override",
        "fn material_asset_reports_shader_contract_diagnostics_without_blocking_import",
        "fn material_asset_management_record_set_sorts_and_summarizes_records",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/assets/material.rs should mount child test owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/assets/material.rs should not keep executable tests in the parent module"
    );
    let child_sources = [
        asset_serialization.as_str(),
        owned_descriptor.as_str(),
        override_validation.as_str(),
        shader_readiness.as_str(),
        management_records.as_str(),
    ];
    assert_eq!(
        child_sources
            .iter()
            .map(|source| source.matches("#[test]").count())
            .sum::<usize>(),
        23,
        "asset material child modules should preserve the original 23 tests"
    );

    assert_contains_all(
        "asset material serialization child owns TOML contracts",
        &asset_serialization,
        &[
            "use super::*;",
            "fn material_asset_zmaterial_roundtrip_maps_pbr_fields_to_shader_overrides",
            "fn material_asset_roundtrip_preserves_standard_texture_transforms",
            "fn material_asset_serialization_rewrites_stale_canonical_overrides",
        ],
    );
    assert_contains_all(
        "asset material descriptor child owns standard material contracts",
        &owned_descriptor,
        &[
            "use super::*;",
            "fn material_owned_lighting_model_drives_standard_descriptor_without_shader_override",
            "fn material_owned_render_queue_reports_blend_queue_alpha_conflict",
            "fn material_owned_taa_reactive_mask_strength_drives_standard_descriptor_without_shader_override",
        ],
    );
    assert_contains_all(
        "asset material validation child owns override error contracts",
        &override_validation,
        &[
            "use super::*;",
            "fn material_owned_receive_shadows_reports_non_bool_override",
            "fn material_owned_sort_fields_report_invalid_override_types",
            "fn material_asset_reports_invalid_lighting_model_as_material_validation_error",
        ],
    );
    assert_contains_all(
        "asset material shader readiness child owns contract diagnostics",
        &shader_readiness,
        &[
            "use super::*;",
            "fn material_asset_reports_shader_contract_diagnostics_without_blocking_import",
            "fn material_asset_reports_missing_required_shader_texture_slot",
            "fn shader_declared_texture_slot_overrides_standard_material_bridge",
        ],
    );
    assert_contains_all(
        "asset material management child owns management summary contracts",
        &management_records,
        &[
            "use super::*;",
            "fn material_asset_management_record_set_sorts_and_summarizes_records",
            "MaterialAssetManagementRecordSet::from_records",
        ],
    );

    for source in [
        parent.as_str(),
        asset_serialization.as_str(),
        owned_descriptor.as_str(),
        override_validation.as_str(),
        shader_readiness.as_str(),
        management_records.as_str(),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "asset material parent and child test owners should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset material test folder split",
                "runtime_15_asset_material_tests_folder_split_static_passed_cargo_lock_blocked",
                "asset/tests/assets/material.rs",
                "asset/tests/assets/material/owned_descriptor.rs",
                "runtime_15_asset_material_tests_are_folder_backed",
            ],
        );
    }
}
