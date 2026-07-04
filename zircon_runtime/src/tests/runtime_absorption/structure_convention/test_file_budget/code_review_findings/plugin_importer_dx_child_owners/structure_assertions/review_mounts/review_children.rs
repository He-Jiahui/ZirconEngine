use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_dx_review_children_are_mounted(
    sources: &PluginImporterDxReviewMountSources,
) {
    assert_contains_all(
        "plugin importer DX D10 child owns animation/physics bridge-call migration review guard",
        &sources.plugin_importer_dx_d10,
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
        &sources.plugin_importer_dx_d1,
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
        &sources.plugin_importer_dx_d11,
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
        &sources.plugin_importer_dx_d6,
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
        "plugin importer DX D8 child owns runtime registration builder review guard",
        &sources.plugin_importer_dx_d8,
        &[
            "fn review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
            "D8_RUNTIME_REGISTRATION_CRATES",
            "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred",
            "RuntimePluginRegistrationBuilder",
            "RuntimePluginModuleRegistration::event",
        ],
    );
    assert_contains_all(
        "plugin importer DX D5 child owns editor authoring macro review guard",
        &sources.plugin_importer_dx_d5,
        &[
            "fn review_d5_editor_authoring_plugins_use_sdk_macro",
            "D5_EDITOR_AUTHORING_MACRO_CRATES",
            "d5_editor_authoring_macro_consumers_static_passed_cargo_deferred",
            "zircon_plugin_sdk::authoring_plugin!",
            "plugin.declaration().registration_report(&plugin)",
        ],
    );
    assert_contains_all(
        "plugin importer DX D9 child owns editor/runtime mirror review guard",
        &sources.plugin_importer_dx_d9,
        &[
            "fn review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
            "D9_EDITOR_RUNTIME_MIRROR_CRATES",
            "d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred",
            "EditorPluginDeclaration::mirrors_runtime_manifest",
            "d9_editor_runtime_mirror_gate_status",
        ],
    );
    assert_contains_all(
        "plugin importer DX D12 child owns runtime export macro review guard",
        &sources.plugin_importer_dx_d12,
        &[
            "fn review_d12_runtime_helper_exports_use_sdk_macro",
            "D12_TRAIT_BACKED_RUNTIME_CRATES",
            "plugins_12_runtime_export_macro_rollout_check_passed",
            "zircon_plugin_sdk::runtime_plugin_exports!",
            "15 个 first-party trait-backed runtime roots",
        ],
    );
}
