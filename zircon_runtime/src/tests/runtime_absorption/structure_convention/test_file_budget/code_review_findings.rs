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
    let typed_error_diagnostics = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/diagnostics.rs",
    );
    let typed_error_dynamic_api = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/dynamic_api.rs",
    );
    let typed_error_export_cli = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/export_cli.rs",
    );
    let typed_error_script_host = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host.rs",
    );
    let typed_error_scene_world = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
    );
    let typed_error_ui_asset_documents = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_asset_documents.rs",
    );
    let typed_error_ui_input = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
    );
    let typed_error_ui_template_resource = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_template_resource.rs",
    );
    let f8_api_convergence =
        read_runtime_src("tests/runtime_absorption/code_review_findings/f8_api_convergence.rs");
    let late_api_cleanup =
        read_runtime_src("tests/runtime_absorption/code_review_findings/late_api_cleanup.rs");
    let p0_robustness =
        read_runtime_src("tests/runtime_absorption/code_review_findings/p0_robustness.rs");
    let plugin_importer_dx =
        read_runtime_src("tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs");
    let plugin_importer_dx_d10 = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
    );
    let plugin_importer_dx_d1 = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source.rs",
    );
    let plugin_importer_dx_d11 = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d11_test_runtime_fixture.rs",
    );
    let plugin_importer_dx_d12 = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d12_runtime_exports.rs",
    );
    let plugin_importer_dx_d13 = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
    );
    let plugin_importer_dx_d6 = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs",
    );
    let render_structure =
        read_runtime_src("tests/runtime_absorption/code_review_findings/render_structure.rs");
    let f12_dead_code =
        read_runtime_src("tests/runtime_absorption/code_review_findings/f12_dead_code.rs");

    assert_contains_all(
        "code review findings parent mounts folder-backed children",
        &parent,
        &[
            "mod f12_dead_code;",
            "mod f8_api_convergence;",
            "mod late_api_cleanup;",
            "mod p0_robustness;",
            "mod plugin_importer_dx;",
            "mod render_structure;",
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
        "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
        "review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror",
        "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
        "review_d12_runtime_helper_exports_use_sdk_macro",
        "review_d5_editor_authoring_plugins_use_sdk_macro",
        "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
        "review_d13_importer_runtime_exports_use_sdk_macro",
        "review_d13_importer_runtime_manifests_use_sdk_builder",
        "review_d11_animation_physics_tests_use_sdk_test_runtime_fixture",
        "review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
        "review_f16_compiled_scene_render_path_uses_split_owners",
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
            "mod diagnostics;",
            "mod dynamic_api;",
            "mod export_cli;",
            "mod script_host;",
            "mod scene_world;",
            "mod ui_asset_documents;",
            "mod ui_input;",
            "mod ui_template_resource;",
        ],
    );
    let typed_error_children = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        typed_error_animation_resource,
        typed_error_asset_loaders,
        typed_error_asset_records,
        typed_error_diagnostics,
        typed_error_dynamic_api,
        typed_error_export_cli,
        typed_error_script_host,
        typed_error_scene_world,
        typed_error_ui_asset_documents,
        typed_error_ui_input,
        typed_error_ui_template_resource
    );
    assert_contains_all(
        "typed-error convergence child owners preserve F5-F7 review guards",
        &typed_error_children,
        &[
            "fn review_f5_world_spawn_bundle_surface_uses_scene_error",
            "fn review_f5_dynamic_component_errors_preserve_scene_error_sources",
            "fn review_f5_sound_asset_uses_typed_error",
            "fn review_f5_animation_asset_binary_uses_typed_errors",
            "fn review_f5_profile_export_uses_typed_error",
            "fn review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary",
            "fn review_f5_script_scene_hook_uses_typed_errors_before_core_boundary",
            "fn review_f5_vm_plugin_management_policy_uses_typed_validation_errors",
            "fn review_f5_ui_asset_documents_use_typed_errors_before_import_boundary",
            "fn review_f5_ui_input_surrounding_text_error_implements_std_error",
            "fn review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary",
            "fn review_f5_ui_template_resource_resolver_uses_typed_lookup_errors_before_diagnostics_boundary",
            "fn review_f5_export_cli_uses_typed_errors_before_cli_boundary",
            "fn review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary",
            "fn review_f5_dynamic_api_session_uses_typed_errors_before_abi_status_boundary",
            "fn review_f6_core_resource_registry_rename_uses_core_error",
            "fn review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    );
    assert_contains_all(
        "P0 robustness child owns FFI panic, lock-poison, and render submit review guards",
        &p0_robustness,
        &[
            "fn review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
            "fn review_f2_scene_eventbus_locks_recover_after_poison",
            "fn review_f4_render_submit_capability_gaps_return_typed_errors",
        ],
    );
    assert_contains_all(
        "plugin importer DX child owns D8 registration builder and mounts importer review guard children",
        &plugin_importer_dx,
        &[
            "mod d10_bridge_call;",
            "mod d1_capability_single_source;",
            "mod d11_test_runtime_fixture;",
            "mod d13_importer_sdk;",
            "mod d6_runtime_plugin_id;",
            "fn review_d10_animation_physics_tests_use_sdk_bridge_call",
            "d10_animation_physics_bridge_call_static_passed_cargo_deferred",
            "WeakBridge<dyn PhysicsQueryInterface>",
            "fn review_d6_runtime_plugin_id_accepts_external_string_keys",
            "d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred",
            "fn review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
            "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred",
            "RuntimePluginRegistrationBuilder",
            "RuntimePluginModuleRegistration::event",
            "mod d12_runtime_exports;",
            "fn review_d5_editor_authoring_plugins_use_sdk_macro",
            "d5_editor_authoring_macro_consumers_static_passed_cargo_deferred",
            "zircon_plugin_sdk::authoring_plugin!",
            "fn review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
            "d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred",
            "EditorPluginDeclaration::mirrors_runtime_manifest",
            "d9_editor_runtime_mirror_gate_status",
        ],
    );
    assert_contains_all(
        "plugin importer DX D10 child owns animation/physics bridge-call migration review guard",
        &plugin_importer_dx_d10,
        &[
            "fn review_d10_animation_physics_tests_use_sdk_bridge_call",
            "PhysicsQueryInterface",
            "WeakBridge<dyn PhysicsQueryInterface>",
            "physics.query.v1",
            "d10_animation_physics_bridge_call_static_passed_cargo_deferred",
            "zircon_plugins/physics/runtime/src/plugin.rs",
            "zircon_plugins/plugin_sdk/src/registration.rs",
        ],
    );
    assert_contains_all(
        "plugin importer DX D1 child owns capability single-source and SDK builder mirror review guard",
        &plugin_importer_dx_d1,
        &[
            "fn review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror",
            "D1_RUNTIME_CAPABILITY_ROOTS",
            "plugins_12_runtime_capability_single_source_guard_passed",
            "plugins_12_capability_single_source_conformance",
            "m4_runtime_capability_gate_status = runtime-capability-single-source-clean",
            "m4_t2_builder_mirror_gate_status = sdk-builder-mirror-clean",
            "PluginFeatureBundleBuilder",
            "d1_capability_single_source_review_synced_static_passed_cargo_deferred",
            "15 个 trait-backed first-party runtime roots",
        ],
    );
    assert_contains_all(
        "plugin importer DX D11 child owns animation/physics TestRuntime fixture migration review guard",
        &plugin_importer_dx_d11,
        &[
            "fn review_d11_animation_physics_tests_use_sdk_test_runtime_fixture",
            "runtime_physics_animation_tick_contract.rs",
            "runtime_physics_animation_tick_contract/animation_assets.rs",
            "runtime_physics_animation_tick_contract/runtime_helpers.rs",
            "TestRuntime::builder()",
            "d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "plugin importer DX D6 child owns RuntimePluginId open string-newtype review guard",
        &plugin_importer_dx_d6,
        &[
            "fn review_d6_runtime_plugin_id_accepts_external_string_keys",
            "RuntimePluginId",
            "enum RuntimePluginId",
            "runtime_plugin_id_accepts_external_keys_without_core_variant",
            "d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred",
            "第三方合法 key 不需 core enum 分支",
        ],
    );
    assert_contains_all(
        "plugin importer DX D12 child owns runtime export macro review guard",
        &plugin_importer_dx_d12,
        &[
            "fn review_d12_runtime_helper_exports_use_sdk_macro",
            "D12_TRAIT_BACKED_RUNTIME_CRATES",
            "plugins_12_runtime_export_macro_rollout_check_passed",
            "zircon_plugin_sdk::runtime_plugin_exports!",
            "15 个 first-party trait-backed runtime roots",
        ],
    );
    assert_contains_all(
        "plugin importer DX D13 child owns importer SDK export, manifest builder, and parity review guards",
        &plugin_importer_dx_d13,
        &[
            "fn review_d13_importer_runtime_exports_use_sdk_macro",
            "d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred",
            "zircon_plugin_sdk::runtime_plugin_exports!",
            "fn review_d13_importer_runtime_manifests_use_sdk_builder",
            "d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred",
            "ImporterRuntimeManifestBuilder",
            "fn review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
            "d13_importer_manifest_parity_guard_static_passed_cargo_deferred",
            "d13_importer_top_row_closed_status_static_passed_cargo_deferred",
            "importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity",
            "NATIVE_ABI_VERSION_V3",
            "NATIVE_DESCRIPTOR_SYMBOL_V3",
        ],
    );
    assert_contains_all(
        "render structure child owns F16 render_compiled_scene review guard",
        &render_structure,
        &[
            "fn review_f16_compiled_scene_render_path_uses_split_owners",
            "bind_compiled_scene_graph_resources.rs",
            "execute_compiled_scene_graph_stages.rs",
            "submit_compiled_scene_frame.rs",
            "compiled_scene_render_split_review_guard_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "F12 dead-code child owns production suppression review guard",
        &f12_dead_code,
        &[
            "fn review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
            "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
            "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
            "Runtime production `allow(dead_code)` sweep is globally gated",
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
            "fn review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending",
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
        typed_error_diagnostics.as_str(),
        typed_error_dynamic_api.as_str(),
        typed_error_export_cli.as_str(),
        typed_error_script_host.as_str(),
        typed_error_scene_world.as_str(),
        typed_error_ui_asset_documents.as_str(),
        typed_error_ui_input.as_str(),
        typed_error_ui_template_resource.as_str(),
        f8_api_convergence.as_str(),
        late_api_cleanup.as_str(),
        p0_robustness.as_str(),
        plugin_importer_dx.as_str(),
        plugin_importer_dx_d10.as_str(),
        plugin_importer_dx_d1.as_str(),
        plugin_importer_dx_d11.as_str(),
        plugin_importer_dx_d12.as_str(),
        plugin_importer_dx_d13.as_str(),
        plugin_importer_dx_d6.as_str(),
        render_structure.as_str(),
        f12_dead_code.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 58,
        "code review findings children should preserve all 58 review guards"
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
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/diagnostics.rs",
            typed_error_diagnostics.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/dynamic_api.rs",
            typed_error_dynamic_api.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/export_cli.rs",
            typed_error_export_cli.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host.rs",
            typed_error_script_host.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
            typed_error_scene_world.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_asset_documents.rs",
            typed_error_ui_asset_documents.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
            typed_error_ui_input.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_template_resource.rs",
            typed_error_ui_template_resource.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
            f8_api_convergence.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/late_api_cleanup.rs",
            late_api_cleanup.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/p0_robustness.rs",
            p0_robustness.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
            plugin_importer_dx.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
            plugin_importer_dx_d10.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source.rs",
            plugin_importer_dx_d1.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d11_test_runtime_fixture.rs",
            plugin_importer_dx_d11.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d12_runtime_exports.rs",
            plugin_importer_dx_d12.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
            plugin_importer_dx_d13.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs",
            plugin_importer_dx_d6.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/render_structure.rs",
            render_structure.as_str(),
        ),
        (
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
            f12_dead_code.as_str(),
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
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
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/diagnostics.rs",
                "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
                "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d11_test_runtime_fixture.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d12_runtime_exports.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
                "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs",
                "review_d10_animation_physics_tests_use_sdk_bridge_call",
                "review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror",
                "review_d11_animation_physics_tests_use_sdk_test_runtime_fixture",
                "review_d6_runtime_plugin_id_accepts_external_string_keys",
                "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
                "review_d12_runtime_helper_exports_use_sdk_macro",
                "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
                "review_d13_importer_runtime_exports_use_sdk_macro",
                "review_d13_importer_runtime_manifests_use_sdk_builder",
                "review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
                "d13_importer_manifest_parity_guard_static_passed_cargo_deferred",
                "d13_importer_top_row_closed_status_static_passed_cargo_deferred",
                "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
                "p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred",
                "d7_core_workspace_dependency_top_row_closed_status_static_passed_cargo_deferred",
                "f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred",
                "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred",
                "f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred",
                "review_priority_recommendation_d13_parity_sync_static_passed_cargo_deferred",
                "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
                "runtime_15_code_review_findings_tests_are_folder_backed",
            ],
        );
    }
}
