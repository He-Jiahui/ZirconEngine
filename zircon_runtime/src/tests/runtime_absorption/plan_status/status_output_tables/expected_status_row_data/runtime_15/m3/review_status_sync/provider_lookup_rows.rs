type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync",
        &[
            "f13_f14_provider_diagnostics_top_row_closed_status_static_passed_cargo_deferred",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "tests/runtime_absorption/structure_convention/provider_boilerplate/full_audit.rs",
            "tests/runtime_absorption/structure_convention/diagnostics_surface.rs",
            "runtime_15_no_duplicated_provider_boilerplate",
            "runtime_15_diagnostics_use_frame_trait_without_world_wrapper",
            "| convention + Runtime 15 / review closed |",
            "| Runtime 15 / review closed |",
            "runtime_15_provider_boilerplate_full_audit_coremin_check_passed",
            "runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 F17/F18 lookup/manager top-row closed status sync",
        &[
            "f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "tests/runtime_absorption/code_review_findings/late_api_cleanup.rs",
            "review_f17_entity_path_option_lookup_uses_get_verb",
            "review_f18_asset_manager_resolution_returns_registered_handle",
            "| convention + Runtime 08 / review closed |",
            "| Runtime 10 / review closed |",
            "runtime_08_entity_path_lookup_getter_rename_coremin_check_passed",
            "runtime_10_asset_manager_resolution_handle_shape_coremin_check_passed",
            "get_entity_by_path",
            "Result<Arc<AssetManagerHandle>, CoreError>",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 F19 scene renderer construction top-row closed status sync",
        &[
            "f19_scene_renderer_construction_top_row_closed_status_static_passed_cargo_deferred",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "tests/runtime_absorption/code_review_findings/late_api_cleanup.rs",
            "review_f19_scene_renderer_construction_modules_use_construct_names",
            "| convention + render index / review closed |",
            "render_scene_renderer_construct_modules_coremin_passed",
            "scene_renderer_core_construct",
            "scene_renderer_construct",
            "Cargo gate deferred",
        ],
    ),
];
