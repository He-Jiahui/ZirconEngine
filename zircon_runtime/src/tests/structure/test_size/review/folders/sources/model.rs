//! 为审查守卫汇总组织专属断言与资料清单；调用关系保留在子职责中，转发入口自身不提供额外的运行验收。
#[derive(Debug)]
pub(in super::super) struct CodeReviewFindingsSources {
    pub(in super::super) parent: String,
    pub(in super::super) f8_api_convergence: String,
    pub(in super::super) f8_texture_import_settings: String,
    pub(in super::super) f8_descriptor_builder: String,
    pub(in super::super) f8_descriptor_builder_scaffold: String,
    pub(in super::super) f8_descriptor_builder_first_party: String,
    pub(in super::super) f8_descriptor_builder_test_fixtures: String,
    pub(in super::super) f8_descriptor_privacy: String,
    pub(in super::super) f8_descriptor_privacy_private_fields: String,
    pub(in super::super) f8_descriptor_privacy_constructor_retirement: String,
    pub(in super::super) f8_descriptor_privacy_status_mirrors: String,
    pub(in super::super) late_api_cleanup: String,
    pub(in super::super) p0_robustness: String,
    pub(in super::super) p0_native_host_callbacks: String,
    pub(in super::super) p0_lock_poison: String,
    pub(in super::super) p0_render_submit: String,
    pub(in super::super) p0_native_fixture: String,
    pub(in super::super) p0_native_fixture_sdk_macro: String,
    pub(in super::super) p0_native_fixture_importer: String,
    pub(in super::super) p0_priority_recommendation: String,
    pub(in super::super) render_structure: String,
    pub(in super::super) f12_dead_code: String,
}

impl CodeReviewFindingsSources {
    pub(in super::super) fn direct_review_guard_count(&self) -> usize {
        [
            self.f8_api_convergence.as_str(),
            self.f8_texture_import_settings.as_str(),
            self.f8_descriptor_builder.as_str(),
            self.f8_descriptor_builder_scaffold.as_str(),
            self.f8_descriptor_builder_first_party.as_str(),
            self.f8_descriptor_builder_test_fixtures.as_str(),
            self.f8_descriptor_privacy.as_str(),
            self.f8_descriptor_privacy_private_fields.as_str(),
            self.f8_descriptor_privacy_constructor_retirement.as_str(),
            self.f8_descriptor_privacy_status_mirrors.as_str(),
            self.p0_robustness.as_str(),
            self.p0_native_host_callbacks.as_str(),
            self.p0_lock_poison.as_str(),
            self.p0_render_submit.as_str(),
            self.p0_native_fixture.as_str(),
            self.p0_native_fixture_sdk_macro.as_str(),
            self.p0_native_fixture_importer.as_str(),
            self.p0_priority_recommendation.as_str(),
            self.render_structure.as_str(),
            self.f12_dead_code.as_str(),
        ]
        .into_iter()
        .map(|source| source.matches("#[test]").count())
        .sum()
    }
}

pub(super) fn assert_review_graph_part(identities: &mut Vec<String>) {
    super::assert_review_owner_contract(
        "tests/plugin_importer_dx/d1_capability_single_source.rs",
        "code_review_findings::plugin_importer_dx::d1_capability_single_source",
        &[
            (
                "audit_surfaces",
                "#[path = \"../review/d1_capability_single_source/audit_surfaces.rs\"]",
            ),
            (
                "runtime_roots",
                "#[path = \"../review/d1_capability_single_source/runtime_roots.rs\"]",
            ),
            (
                "sdk_builder",
                "#[path = \"../review/d1_capability_single_source/sdk_builder.rs\"]",
            ),
            (
                "split_layout",
                "#[path = \"../review/d1_capability_single_source/split_layout.rs\"]",
            ),
            (
                "status_docs",
                "#[path = \"../review/d1_capability_single_source/status_docs.rs\"]",
            ),
            (
                "support",
                "#[path = \"../contracts/review/d1_capability_single_source/support.rs\"]",
            ),
        ],
        &["review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/d1_capability_single_source/audit_surfaces.rs",
        "code_review_findings::plugin_importer_dx::d1_capability_single_source::audit_surfaces",
        &[],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/d1_capability_single_source/runtime_roots.rs",
        "code_review_findings::plugin_importer_dx::d1_capability_single_source::runtime_roots",
        &[],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/d1_capability_single_source/sdk_builder.rs",
        "code_review_findings::plugin_importer_dx::d1_capability_single_source::sdk_builder",
        &[],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/d1_capability_single_source/split_layout.rs",
        "code_review_findings::plugin_importer_dx::d1_capability_single_source::split_layout",
        &[],
        &["runtime_15_plugin_importer_d1_capability_guard_is_folder_backed"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/d1_capability_single_source/status_docs.rs",
        "code_review_findings::plugin_importer_dx::d1_capability_single_source::status_docs",
        &[],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/d1_capability_single_source/support.rs",
        "code_review_findings::plugin_importer_dx::d1_capability_single_source::support",
        &[],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/plugin_importer_dx/d5_editor_authoring.rs",
        "code_review_findings::plugin_importer_dx::d5_editor_authoring",
        &[],
        &["review_d5_editor_authoring_plugins_use_sdk_macro"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/plugin_importer_dx/d6_runtime_plugin_id.rs",
        "code_review_findings::plugin_importer_dx::d6_runtime_plugin_id",
        &[],
        &[
            "review_d6_runtime_plugin_id_accepts_external_string_keys",
            "runtime_plugin_id_non_copy_contract_reaches_workspace_consumers",
        ],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/plugin_importer_dx/d8_registration_builder.rs",
        "code_review_findings::plugin_importer_dx::d8_registration_builder",
        &[],
        &["review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/plugin_importer_dx/d9_editor_runtime_mirror.rs",
        "code_review_findings::plugin_importer_dx::d9_editor_runtime_mirror",
        &[],
        &["review_d9_editor_runtime_mirror_consumers_use_sdk_declaration"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/render_structure.rs",
        "code_review_findings::render_structure",
        &[
            ("submission", "#[path = \"render_structure/submission.rs\"]"),
        ],
        &[
            "render01_compiled_pipeline_cached_sources_are_immutable_and_frame_flags_are_precomputed",
            "review_f16_compiled_scene_render_path_uses_split_owners",
        ],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/render_structure/submission.rs",
        "code_review_findings::render_structure::submission",
        &[],
        &["runtime09a_frame_submission_metrics_keep_backend_mapping_and_receipt_owners_split"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/status_placeholder_guard.rs",
        "code_review_findings::status_placeholder_guard",
        &[],
        &["review_numbered_output_uses_concrete_status_anchors_instead_of_placeholders"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/mod.rs",
        "code_review_findings::typed_error_convergence",
        &[
            ("animation_resource", "#[path = \"animation_resource.rs\"]"),
            ("asset_loaders", "#[path = \"asset_loaders.rs\"]"),
            ("asset_records", "#[path = \"asset_records.rs\"]"),
            ("diagnostics", "#[path = \"diagnostics.rs\"]"),
            ("dynamic_api", "#[path = \"dynamic_api.rs\"]"),
            ("export_cli", "#[path = \"export_cli.rs\"]"),
            ("native_plugin_loader", "#[path = \"loader.rs\"]"),
            ("scene_world", "#[path = \"scene_world.rs\"]"),
            ("script_host", "#[path = \"script_host.rs\"]"),
            ("shader_prewarm_cli", "#[path = \"shader_prewarm_cli.rs\"]"),
            ("ui_asset_documents", "#[path = \"ui_asset_documents.rs\"]"),
            ("ui_input", "#[path = \"ui_input.rs\"]"),
            (
                "ui_template_resource",
                "#[path = \"ui_template_resource.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/animation_resource.rs",
        "code_review_findings::typed_error_convergence::animation_resource",
        &[],
        &[
            "review_f5_animation_manager_uses_animation_error",
            "review_f6_core_resource_registry_rename_uses_resource_error",
        ],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_loaders.rs",
        "code_review_findings::typed_error_convergence::asset_loaders",
        &[
            (
                "animation_binary",
                "#[path = \"../../../review/errors/asset_loaders/animation_binary.rs\"]",
            ),
            (
                "artifact_importer",
                "#[path = \"../../../review/errors/asset_loaders/artifact_importer.rs\"]",
            ),
            ("mesh_obj", "#[path = \"asset_loaders/mesh_obj.rs\"]"),
            ("texture", "#[path = \"asset_loaders/texture.rs\"]"),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/errors/asset_loaders/animation_binary.rs",
        "code_review_findings::typed_error_convergence::asset_loaders::animation_binary",
        &[],
        &["review_f5_animation_asset_binary_uses_typed_errors"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/errors/asset_loaders/artifact_importer.rs",
        "code_review_findings::typed_error_convergence::asset_loaders::artifact_importer",
        &[],
        &["review_f7_asset_artifact_errors_use_asset_import_error_sources"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_loaders/mesh_obj.rs",
        "code_review_findings::typed_error_convergence::asset_loaders::mesh_obj",
        &[],
        &["review_f5_mesh_loader_and_obj_decoder_use_typed_errors"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_loaders/texture.rs",
        "code_review_findings::typed_error_convergence::asset_loaders::texture",
        &[],
        &["review_f5_texture_loader_uses_typed_error"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_records.rs",
        "code_review_findings::typed_error_convergence::asset_records",
        &[
            ("authoring", "#[path = \"asset_records/authoring.rs\"]"),
            ("font", "#[path = \"asset_records/font.rs\"]"),
            ("meta", "#[path = \"asset_records/meta.rs\"]"),
            ("navigation", "#[path = \"asset_records/navigation.rs\"]"),
            ("sound", "#[path = \"asset_records/sound.rs\"]"),
            ("zshader", "#[path = \"asset_records/zshader.rs\"]"),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_records/authoring.rs",
        "code_review_findings::typed_error_convergence::asset_records::authoring",
        &[],
        &["review_f5_asset_authoring_uses_typed_error"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_records/font.rs",
        "code_review_findings::typed_error_convergence::asset_records::font",
        &[],
        &["review_f5_font_asset_uses_typed_error_source"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_records/meta.rs",
        "code_review_findings::typed_error_convergence::asset_records::meta",
        &[],
        &["review_f5_asset_meta_uses_typed_error"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_records/navigation.rs",
        "code_review_findings::typed_error_convergence::asset_records::navigation",
        &[],
        &["review_f5_navigation_asset_uses_typed_error"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_records/sound.rs",
        "code_review_findings::typed_error_convergence::asset_records::sound",
        &[],
        &["review_f5_sound_asset_uses_typed_error"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/asset_records/zshader.rs",
        "code_review_findings::typed_error_convergence::asset_records::zshader",
        &[],
        &["review_f5_zshader_v2_replaces_user_shader_definitions"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/diagnostics.rs",
        "code_review_findings::typed_error_convergence::diagnostics",
        &[],
        &["review_f5_profile_export_uses_typed_error"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/dynamic_api.rs",
        "code_review_findings::typed_error_convergence::dynamic_api",
        &[],
        &["review_f5_dynamic_api_session_uses_typed_errors_before_abi_status_boundary"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/export_cli.rs",
        "code_review_findings::typed_error_convergence::export_cli",
        &[],
        &["review_f5_export_cli_uses_typed_errors_before_cli_boundary"],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/loader.rs",
        "code_review_findings::typed_error_convergence::native_plugin_loader",
        &[
            ("abi_surfaces", "#[path = \"loader/abi_surfaces.rs\"]"),
            (
                "bridge_lifecycle",
                "#[path = \"loader/bridge_lifecycle.rs\"]",
            ),
            ("diagnostics", "#[path = \"loader/diagnostics.rs\"]"),
            ("live_host", "#[path = \"loader/live_host.rs\"]"),
            (
                "manifest_sources",
                "#[path = \"loader/manifest_sources.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/contracts/review/errors/loader/abi_surfaces.rs",
        "code_review_findings::typed_error_convergence::native_plugin_loader::abi_surfaces",
        &[
            (
                "behavior_bridge",
                "#[path = \"../../../../review/errors/loader/abi_surfaces/behavior_bridge.rs\"]",
            ),
            (
                "host_adapter",
                "#[path = \"../../../../review/errors/loader/abi_surfaces/host_adapter.rs\"]",
            ),
            (
                "plugin_descriptor",
                "#[path = \"../../../../review/errors/loader/abi_surfaces/plugin_descriptor.rs\"]",
            ),
        ],
        &[],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/errors/loader/abi_surfaces/behavior_bridge.rs",
        "code_review_findings::typed_error_convergence::native_plugin_loader::abi_surfaces::behavior_bridge",
        &[
        ],
        &[
            "review_f5_native_plugin_behavior_abi_uses_typed_error",
            "review_f5_native_bridge_method_abi_uses_typed_error",
        ],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/errors/loader/abi_surfaces/host_adapter.rs",
        "code_review_findings::typed_error_convergence::native_plugin_loader::abi_surfaces::host_adapter",
        &[
        ],
        &[
            "review_f5_native_host_api_adapter_uses_typed_error",
        ],
        identities,
    );
    super::assert_review_owner_contract(
        "tests/review/errors/loader/abi_surfaces/plugin_descriptor.rs",
        "code_review_findings::typed_error_convergence::native_plugin_loader::abi_surfaces::plugin_descriptor",
        &[
            ("descriptor_abi", "#[path = \"../../../../errors/loader/plugin_descriptor/descriptor_abi.rs\"]"),
            ("entry_abi", "#[path = \"../plugin_descriptor/entry_abi.rs\"]"),
            ("string_helpers", "#[path = \"../../../../errors/loader/plugin_descriptor/string_helpers.rs\"]"),
        ],
        &[
        ],
        identities,
    );
}
