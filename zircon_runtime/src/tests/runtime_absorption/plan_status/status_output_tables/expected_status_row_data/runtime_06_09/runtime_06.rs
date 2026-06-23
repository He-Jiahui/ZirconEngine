use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 06 Plugin surface/lifecycle 镜像文档守卫",
        &[
            "runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts",
            "plugin_surface_lifecycle_boundary",
            "standalone plugin_surface_lifecycle 1/1",
            "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
        ],
    ),
    (
        "Runtime 06 native root re-export current mirror fix",
        &[
            "plugin_root_symbols.len()",
            "native root re-export 0/0",
            "last_refined = 2026-06-16",
            "standalone plugin_surface_lifecycle 1/1",
        ],
    ),
    (
        "Runtime 06 plugin surface/lifecycle Markdown renderer split",
        &[
            "plugin_surface_lifecycle_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "plugin_surface_lifecycle_markdown.py",
            "plugin_surface_lifecycle_boundary.py remains the 450-line audit/risk owner",
            "Markdown owner is 144 lines",
        ],
    ),
    (
        "Runtime 06 F8 RuntimePluginDescriptor builder scaffold",
        &[
            "runtime_plugin_descriptor_builder_scaffold_coremin_check_passed",
            "RuntimePluginDescriptorBuilder",
            "RuntimePluginDescriptor::builder(...).build()",
            "RuntimePluginDescriptor public-field convergence remains pending",
        ],
    ),
    (
        "Runtime 06 F8 first-party RuntimePluginDescriptor builder migration",
        &[
            "runtime_plugin_descriptor_first_party_builder_migration_coremin_check_passed",
            "first-party runtime plugin descriptor production files 16/16",
            "RuntimePluginDescriptor::builder(",
            "RuntimePluginDescriptor public-field convergence remains pending",
        ],
    ),
    (
        "Runtime 06 F8 RuntimePluginDescriptor test fixture builder migration",
        &[
            "runtime_plugin_descriptor_test_fixture_builder_migration_coremin_check_passed",
            "plugin extension RuntimePluginDescriptor test fixtures 14/14",
            "review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
            "RuntimePluginDescriptor public-field convergence remains pending",
        ],
    ),
    (
        "Runtime 06 F8 RuntimePluginDescriptor public-field convergence",
        &[
            "runtime_plugin_descriptor_public_field_convergence_coremin_check_passed",
            "RuntimePluginDescriptor private fields 15/15",
            "review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors",
            "RuntimePluginDescriptor public-field convergence complete",
        ],
    ),
    (
        "Runtime 06 F8 RuntimePluginDescriptor public constructor retired",
        &[
            "runtime_plugin_descriptor_public_constructor_retired_coremin_check_passed",
            "RuntimePluginDescriptor::new retired",
            "descriptor/builder/construction.rs retired",
            "review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
        ],
    ),
    (
        "Runtime 06 plugin::native hard-cutover",
        &[
            "plugin::native",
            "root_reexport_count = 0",
            "native_namespace_reexport_count = 60",
            "M4 gate `classified-and-clear`",
        ],
    ),
    (
        "Runtime 06 fallback lifecycle failure tests",
        &[
            "runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed",
            "fallback lifecycle failure tests 4/4",
            "vm_lifecycle_fallback_missing_optional_export_returns_none_not_error",
            "real-backend Cargo remains pending",
        ],
    ),
    (
        "Runtime 06 fallback lifecycle Cargo 验证",
        &[
            "fallback_cargo_passed_real_backend_pending",
            "vm_lifecycle_fallback --no-default-features --features core-min",
            "5/5",
            "real-backend Cargo",
        ],
    ),
    (
        "Runtime 06 shader artifact cache real-backend unblock",
        &[
            "asset_cache_fixed_vampire_session_pending",
            "ArtifactCacheShaderImportRedirectAsset",
            "project_manager_imports_compound_zshader_package_with_subassets",
            "vampire_project_session_starts_paused_until_start_button_click",
        ],
    ),
    (
        "Runtime 06 Vampire real-backend menu/retry focused validation",
        &[
            "vampire_real_backend_focused_passed_full_gate_pending",
            "vampire_project_session_game_over_menu_retries_to_playing",
            "gameplay.script_number_at_most",
            "vampire.spawn_grace",
        ],
    ),
    (
        "Runtime 06 Vampire HUD real-backend capture validation",
        &[
            "vampire_hud_real_backend_focused_passed_full_gate_pending",
            "particle-render",
            "vampire_project_session_capture_frame_draws_world_hud_bars",
            "particle_pipeline_keeps_world_hud_billboards_transparent_and_depth_read_only",
        ],
    ),
    (
        "Runtime 06 native loader test namespace migration",
        &[
            "runtime_06_native_loader_tests_use_isolated_plugin_native_namespace",
            "native loader test files 3/3",
            "native test namespace import files 2/2",
            "native test root import leaks 0/0",
        ],
    ),
    (
        "Runtime 06 V1/V2 ABI hard-cutover",
        &[
            "V3-only native plugin ABI",
            "unknown ABI rejection",
            "native_loader_v1_v2_file_count = 0",
            "plugin_v1_v2_usage_files = 0",
        ],
    ),
    (
        "Runtime 06 hot reload failure injection",
        &[
            "hot_reload_missing_symbol_after_reload_rolls_back_to_previous_instance",
            "hot_reload_state_restore_failure_rolls_back_and_reports",
            "hot reload failure injection",
            "Cargo timeout",
        ],
    ),
];
