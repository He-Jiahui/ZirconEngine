type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync",
        &[
            "f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
            "review_f6_core_resource_registry_rename_uses_core_error",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
            "| Runtime 08 + Runtime 15 / review closed |",
            "| Runtime 02 / review closed |",
            "| Runtime 04 / review closed |",
            "world_typed_mutation_errors_coremin_check_passed_partial",
            "core_resource_registry_typed_errors_coremin_check_passed",
            "asset_artifact_importer_typed_errors_coremin_passed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 F8/F9/F10 runtime surface top-row closed status sync",
        &[
            "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
            "tests/runtime_absorption/structure_convention/facade_surface.rs",
            "tests/runtime_absorption/structure_convention/runtime_dead_code.rs",
            "review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending",
            "runtime_15_prelude_covers_required_types",
            "runtime_15_runtime_ui_dead_code_surface_is_test_support",
            "| convention + Runtime 04 + Runtime 06 + Runtime 15 / review closed |",
            "| Runtime 15 / review closed |",
            "| Runtime 09 + Runtime 15 / review closed |",
            "runtime_15_runtime_plugin_descriptor_status_mirror_cleanup_static_passed_cargo_deferred",
            "runtime_15_prelude_required_types_coremin_check_passed",
            "runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed",
            "Cargo gate deferred",
        ],
    ),
];
