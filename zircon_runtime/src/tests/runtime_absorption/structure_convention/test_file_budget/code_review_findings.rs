use super::*;

#[test]
fn runtime_15_code_review_findings_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/code_review_findings.rs");
    let typed_error_convergence = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
    );
    let typed_error_animation_resource =
        read_runtime_src("tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs");
    let typed_error_asset_loaders = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
    );
    let typed_error_asset_records = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
    );
    let typed_error_scene_world = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
    );
    let f8_api_convergence =
        read_runtime_src("tests/runtime_absorption/code_review_findings/f8_api_convergence.rs");
    let late_api_cleanup =
        read_runtime_src("tests/runtime_absorption/code_review_findings/late_api_cleanup.rs");

    assert_contains_all(
        "code review findings parent mounts folder-backed children",
        &parent,
        &[
            "mod f8_api_convergence;",
            "mod late_api_cleanup;",
            "mod typed_error_convergence;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "code_review_findings.rs should only mount child test owners"
    );
    for moved_test in [
        "review_f5_world_spawn_bundle_surface_uses_scene_error",
        "review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
        "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
        "review_f19_scene_renderer_construction_modules_use_construct_names",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved code review findings test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "typed-error convergence child owns F5-F7 review guards",
        &typed_error_convergence,
        &[
            "mod animation_resource;",
            "mod asset_loaders;",
            "mod asset_records;",
            "mod scene_world;",
        ],
    );
    let typed_error_children = format!(
        "{}\n{}\n{}\n{}",
        typed_error_animation_resource,
        typed_error_asset_loaders,
        typed_error_asset_records,
        typed_error_scene_world
    );
    assert_contains_all(
        "typed-error convergence child owners preserve F5-F7 review guards",
        &typed_error_children,
        &[
            "fn review_f5_world_spawn_bundle_surface_uses_scene_error",
            "fn review_f5_dynamic_component_errors_preserve_scene_error_sources",
            "fn review_f5_sound_asset_uses_typed_error",
            "fn review_f6_core_resource_registry_rename_uses_core_error",
            "fn review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    );
    assert_contains_all(
        "F8 API convergence child owns texture and descriptor review guards",
        &f8_api_convergence,
        &[
            "fn review_f8_texture_import_settings_use_fallible_apply_not_with",
            "fn review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
            "fn review_f8_first_party_runtime_plugin_descriptors_use_builder",
            "fn review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
            "fn review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors",
            "fn review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
        ],
    );
    assert_contains_all(
        "late API cleanup child owns F11/F17/F18/F19 review guards",
        &late_api_cleanup,
        &[
            "fn review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
            "fn review_f17_entity_path_option_lookup_uses_get_verb",
            "fn review_f18_asset_manager_resolution_returns_registered_handle",
            "fn review_f19_scene_renderer_construction_modules_use_construct_names",
        ],
    );

    let child_test_total = [
        typed_error_convergence.as_str(),
        typed_error_animation_resource.as_str(),
        typed_error_asset_loaders.as_str(),
        typed_error_asset_records.as_str(),
        typed_error_scene_world.as_str(),
        f8_api_convergence.as_str(),
        late_api_cleanup.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 25,
        "code review findings children should preserve all 25 review guards"
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/code_review_findings.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            typed_error_convergence.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs",
            typed_error_animation_resource.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
            typed_error_asset_loaders.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
            typed_error_asset_records.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
            typed_error_scene_world.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
            f8_api_convergence.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/late_api_cleanup.rs",
            late_api_cleanup.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
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
                "Runtime 15 M3 code review findings test folder split",
                "runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred",
                "tests/runtime_absorption/code_review_findings.rs",
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
                "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
                "runtime_15_code_review_findings_tests_are_folder_backed",
            ],
        );
    }
}
