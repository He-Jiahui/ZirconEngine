//! 为审查守卫汇总读取或聚合父子源码，保留路径与文本的配对关系供上层检查；聚合结果只描述被列入清单的文件。
use super::super::super::super::*;
use super::model::CodeReviewFindingsSources;

pub(super) fn code_review_findings_sources() -> CodeReviewFindingsSources {
    CodeReviewFindingsSources {
        parent: read_runtime_src("tests/contracts/review.rs"),
        f8_api_convergence: read_runtime_src("tests/contracts/review/f8_api_convergence.rs"),
        f8_texture_import_settings: read_runtime_src(
            "tests/review/f8_api_convergence/texture_import_settings.rs",
        ),
        f8_descriptor_builder: read_runtime_src(
            "tests/review/f8_api_convergence/descriptor_builder.rs",
        ),
        f8_descriptor_builder_scaffold: read_runtime_src(
            "tests/contracts/review/descriptor_builder/scaffold.rs",
        ),
        f8_descriptor_builder_first_party: read_runtime_src(
            "tests/review/descriptor_builder/first_party_descriptors.rs",
        ),
        f8_descriptor_builder_test_fixtures: read_runtime_src(
            "tests/contracts/review/descriptor_builder/test_fixtures.rs",
        ),
        f8_descriptor_privacy: read_runtime_src(
            "tests/review/f8_api_convergence/descriptor_privacy.rs",
        ),
        f8_descriptor_privacy_private_fields: read_runtime_src(
            "tests/contracts/review/descriptor_privacy/private_fields.rs",
        ),
        f8_descriptor_privacy_constructor_retirement: read_runtime_src(
            "tests/review/descriptor_privacy/constructor_retirement.rs",
        ),
        f8_descriptor_privacy_status_mirrors: read_runtime_src(
            "tests/contracts/review/descriptor_privacy/status_mirrors.rs",
        ),
        late_api_cleanup: read_runtime_src("tests/contracts/review/late_api_cleanup.rs"),
        p0_robustness: read_runtime_src("tests/contracts/review/p0_robustness.rs"),
        p0_native_host_callbacks: read_runtime_src(
            "tests/contracts/review/p0_robustness/native_host_callbacks.rs",
        ),
        p0_lock_poison: read_runtime_src("tests/contracts/review/p0_robustness/lock_poison.rs"),
        p0_render_submit: read_runtime_src("tests/contracts/review/p0_robustness/render_submit.rs"),
        p0_native_fixture: read_runtime_src(
            "tests/contracts/review/p0_robustness/native_fixture.rs",
        ),
        p0_native_fixture_sdk_macro: read_runtime_src(
            "tests/contracts/review/native_fixture/sdk_macro_manifest.rs",
        ),
        p0_native_fixture_importer: read_runtime_src(
            "tests/contracts/review/native_fixture/importer_manifest.rs",
        ),
        p0_priority_recommendation: read_runtime_src(
            "tests/review/p0_robustness/priority_recommendation.rs",
        ),
        render_structure: read_runtime_src("tests/contracts/review/render_structure.rs"),
        f12_dead_code: read_runtime_src("tests/contracts/review/f12_dead_code.rs"),
    }
}

pub(super) fn assert_review_graph_part(identities: &mut Vec<String>) {
    super::assert_review_owner_contract(
        "tests/contracts/review.rs",
        "code_review_findings",
        &[
            ("f12_dead_code", "#[path = \"review/f12_dead_code.rs\"]"),
            (
                "f8_api_convergence",
                "#[path = \"review/f8_api_convergence.rs\"]",
            ),
            (
                "late_api_cleanup",
                "#[path = \"review/late_api_cleanup.rs\"]",
            ),
            ("p0_robustness", "#[path = \"review/p0_robustness.rs\"]"),
            (
                "plugin_importer_dx",
                "#[path = \"review/plugin_importer_dx.rs\"]",
            ),
            (
                "render_structure",
                "#[path = \"review/render_structure.rs\"]",
            ),
            (
                "status_placeholder_guard",
                "#[path = \"review/status_placeholder_guard.rs\"]",
            ),
            (
                "typed_error_convergence",
                "#[path = \"review/errors/mod.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/f12_dead_code.rs",
        "code_review_findings::f12_dead_code",
        &[],
        &["review_f12_runtime_production_dead_code_suppression_is_globally_gated"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/f8_api_convergence.rs",
        "code_review_findings::f8_api_convergence",
        &[
            (
                "descriptor_builder",
                "#[path = \"../../review/f8_api_convergence/descriptor_builder.rs\"]",
            ),
            (
                "descriptor_privacy",
                "#[path = \"../../review/f8_api_convergence/descriptor_privacy.rs\"]",
            ),
            (
                "texture_import_settings",
                "#[path = \"../../review/f8_api_convergence/texture_import_settings.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/f8_api_convergence/descriptor_builder.rs",
        "code_review_findings::f8_api_convergence::descriptor_builder",
        &[
            (
                "first_party_descriptors",
                "#[path = \"../descriptor_builder/first_party_descriptors.rs\"]",
            ),
            (
                "scaffold",
                "#[path = \"../../contracts/review/descriptor_builder/scaffold.rs\"]",
            ),
            (
                "test_fixtures",
                "#[path = \"../../contracts/review/descriptor_builder/test_fixtures.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/descriptor_builder/first_party_descriptors.rs",
        "code_review_findings::f8_api_convergence::descriptor_builder::first_party_descriptors",
        &[],
        &["review_f8_first_party_runtime_plugin_descriptors_use_builder"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/descriptor_builder/scaffold.rs",
        "code_review_findings::f8_api_convergence::descriptor_builder::scaffold",
        &[],
        &["review_f8_runtime_plugin_descriptor_exposes_builder_scaffold"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/descriptor_builder/test_fixtures.rs",
        "code_review_findings::f8_api_convergence::descriptor_builder::test_fixtures",
        &[],
        &["review_f8_runtime_plugin_descriptor_test_fixtures_use_builder"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/f8_api_convergence/descriptor_privacy.rs",
        "code_review_findings::f8_api_convergence::descriptor_privacy",
        &[
            (
                "constructor_retirement",
                "#[path = \"../descriptor_privacy/constructor_retirement.rs\"]",
            ),
            (
                "private_fields",
                "#[path = \"../../contracts/review/descriptor_privacy/private_fields.rs\"]",
            ),
            (
                "status_mirrors",
                "#[path = \"../../contracts/review/descriptor_privacy/status_mirrors.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/descriptor_privacy/constructor_retirement.rs",
        "code_review_findings::f8_api_convergence::descriptor_privacy::constructor_retirement",
        &[],
        &["review_f8_runtime_plugin_descriptor_public_constructor_is_retired"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/descriptor_privacy/private_fields.rs",
        "code_review_findings::f8_api_convergence::descriptor_privacy::private_fields",
        &[],
        &["review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/descriptor_privacy/status_mirrors.rs",
        "code_review_findings::f8_api_convergence::descriptor_privacy::status_mirrors",
        &[],
        &["review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/f8_api_convergence/texture_import_settings.rs",
        "code_review_findings::f8_api_convergence::texture_import_settings",
        &[],
        &["review_f8_texture_import_settings_use_fallible_apply_not_with"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/late_api_cleanup.rs",
        "code_review_findings::late_api_cleanup",
        &[
            (
                "f11_shading_model_registry",
                "#[path = \"../../review/late_api_cleanup/f11_shading_model_registry.rs\"]",
            ),
            (
                "f15_editor_pane_data_conversion",
                "#[path = \"../../late_api_cleanup/f15_editor_pane_data_conversion.rs\"]",
            ),
            (
                "f17_entity_path_lookup",
                "#[path = \"../../review/late_api_cleanup/f17_entity_path_lookup.rs\"]",
            ),
            (
                "f18_asset_manager_resolution",
                "#[path = \"../../review/late_api_cleanup/f18_asset_manager_resolution.rs\"]",
            ),
            (
                "f19_scene_renderer_construction",
                "#[path = \"../../review/late_api_cleanup/f19_scene_renderer_construct.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/late_api_cleanup/f11_shading_model_registry.rs",
        "code_review_findings::late_api_cleanup::f11_shading_model_registry",
        &[],
        &["review_f11_shading_model_registry_has_no_dead_plugin_registration_surface"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/late_api_cleanup/f15_editor_pane_data_conversion.rs",
        "code_review_findings::late_api_cleanup::f15_editor_pane_data_conversion",
        &[],
        &["review_f15_editor_pane_data_conversion_top_row_uses_projection_owners"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/late_api_cleanup/f17_entity_path_lookup.rs",
        "code_review_findings::late_api_cleanup::f17_entity_path_lookup",
        &[],
        &["review_f17_entity_path_option_lookup_uses_get_verb"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/late_api_cleanup/f18_asset_manager_resolution.rs",
        "code_review_findings::late_api_cleanup::f18_asset_manager_resolution",
        &[],
        &["review_f18_asset_manager_resolution_returns_registered_handle"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/late_api_cleanup/f19_scene_renderer_construct.rs",
        "code_review_findings::late_api_cleanup::f19_scene_renderer_construction",
        &[],
        &["review_f19_scene_renderer_construction_modules_use_construct_names"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/p0_robustness.rs",
        "code_review_findings::p0_robustness",
        &[
            (
                "gpu_pass_timer",
                "#[path = \"p0_robustness/gpu_pass_timer.rs\"]",
            ),
            ("lock_poison", "#[path = \"p0_robustness/lock_poison.rs\"]"),
            (
                "native_fixture",
                "#[path = \"p0_robustness/native_fixture.rs\"]",
            ),
            (
                "native_host_callbacks",
                "#[path = \"p0_robustness/native_host_callbacks.rs\"]",
            ),
            (
                "priority_recommendation",
                "#[path = \"../../review/p0_robustness/priority_recommendation.rs\"]",
            ),
            (
                "render_submit",
                "#[path = \"p0_robustness/render_submit.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/p0_robustness/gpu_pass_timer.rs",
        "code_review_findings::p0_robustness::gpu_pass_timer",
        &[],
        &["gpu_timer_readback_uses_the_shared_queue_without_a_private_map_lifecycle"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/p0_robustness/lock_poison.rs",
        "code_review_findings::p0_robustness::lock_poison",
        &[],
        &["review_f2_scene_eventbus_locks_recover_after_poison"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/p0_robustness/native_fixture.rs",
        "code_review_findings::p0_robustness::native_fixture",
        &[
            (
                "importer_manifest",
                "#[path = \"../native_fixture/importer_manifest.rs\"]",
            ),
            (
                "sdk_macro_manifest",
                "#[path = \"../native_fixture/sdk_macro_manifest.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/native_fixture/importer_manifest.rs",
        "code_review_findings::p0_robustness::native_fixture::importer_manifest",
        &[],
        &["review_d13_native_fixture_importer_is_manifest_described"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/native_fixture/sdk_macro_manifest.rs",
        "code_review_findings::p0_robustness::native_fixture::sdk_macro_manifest",
        &[],
        &["review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/p0_robustness/native_host_callbacks.rs",
        "code_review_findings::p0_robustness::native_host_callbacks",
        &[],
        &["review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/p0_robustness/priority_recommendation.rs",
        "code_review_findings::p0_robustness::priority_recommendation",
        &[],
        &["review_priority_recommendation_tracks_current_remaining_work"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/p0_robustness/render_submit.rs",
        "code_review_findings::p0_robustness::render_submit",
        &[],
        &["review_f4_render_submit_capability_gaps_return_typed_errors"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/plugin_importer_dx.rs",
        "code_review_findings::plugin_importer_dx",
        &[
            (
                "d10_bridge_call",
                "#[path = \"plugin_importer_dx/d10_bridge_call.rs\"]",
            ),
            (
                "d11_test_runtime_fixture",
                "#[path = \"../../review/plugin_importer_dx/d11_test_runtime_fixture.rs\"]",
            ),
            (
                "d12_runtime_exports",
                "#[path = \"../../review/plugin_importer_dx/d12_runtime_exports.rs\"]",
            ),
            (
                "d13_importer_sdk",
                "#[path = \"plugin_importer_dx/d13_importer_sdk.rs\"]",
            ),
            (
                "d1_capability_single_source",
                "#[path = \"../../plugin_importer_dx/d1_capability_single_source.rs\"]",
            ),
            (
                "d5_editor_authoring",
                "#[path = \"../../review/plugin_importer_dx/d5_editor_authoring.rs\"]",
            ),
            (
                "d6_runtime_plugin_id",
                "#[path = \"../../review/plugin_importer_dx/d6_runtime_plugin_id.rs\"]",
            ),
            (
                "d8_registration_builder",
                "#[path = \"../../review/plugin_importer_dx/d8_registration_builder.rs\"]",
            ),
            (
                "d9_editor_runtime_mirror",
                "#[path = \"../../review/plugin_importer_dx/d9_editor_runtime_mirror.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/plugin_importer_dx/d10_bridge_call.rs",
        "code_review_findings::plugin_importer_dx::d10_bridge_call",
        &[],
        &["review_d10_animation_physics_tests_use_sdk_bridge_call"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/plugin_importer_dx/d11_test_runtime_fixture.rs",
        "code_review_findings::plugin_importer_dx::d11_test_runtime_fixture",
        &[],
        &["review_d11_animation_physics_tests_use_sdk_test_runtime_fixture"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/plugin_importer_dx/d12_runtime_exports.rs",
        "code_review_findings::plugin_importer_dx::d12_runtime_exports",
        &[],
        &["review_d12_runtime_helper_exports_use_sdk_macro"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/plugin_importer_dx/d13_importer_sdk.rs",
        "code_review_findings::plugin_importer_dx::d13_importer_sdk",
        &[
            (
                "manifest_parity",
                "#[path = \"../d13_importer_sdk/manifest_parity.rs\"]",
            ),
            (
                "runtime_crates",
                "#[path = \"../d13_importer_sdk/runtime_crates.rs\"]",
            ),
            (
                "runtime_exports",
                "#[path = \"../d13_importer_sdk/runtime_exports.rs\"]",
            ),
            (
                "runtime_manifests",
                "#[path = \"../d13_importer_sdk/runtime_manifests.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/d13_importer_sdk/manifest_parity.rs",
        "code_review_findings::plugin_importer_dx::d13_importer_sdk::manifest_parity",
        &[],
        &["review_d13_importer_manifest_parity_guard_lives_in_sdk_builder"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/d13_importer_sdk/runtime_crates.rs",
        "code_review_findings::plugin_importer_dx::d13_importer_sdk::runtime_crates",
        &[],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/d13_importer_sdk/runtime_exports.rs",
        "code_review_findings::plugin_importer_dx::d13_importer_sdk::runtime_exports",
        &[],
        &["review_d13_importer_runtime_exports_use_sdk_macro"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/d13_importer_sdk/runtime_manifests.rs",
        "code_review_findings::plugin_importer_dx::d13_importer_sdk::runtime_manifests",
        &[],
        &["review_d13_importer_runtime_manifests_use_sdk_builder"],
        identities,
    );
}
