use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 F9 runtime prelude required type coverage",
        &[
            "runtime_15_prelude_required_types_coremin_check_passed",
            "asset/prelude.rs",
            "runtime_prelude_exports_asset_scene_ui_and_graphics_contracts",
            "runtime_15_prelude_covers_required_types",
        ],
    ),
    (
        "Runtime 15 runtime UI dead-code support split",
        &[
            "runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed",
            "ui/public_runtime_frame.rs",
            "ui/tests/runtime_ui_support",
            "runtime_15_runtime_ui_dead_code_surface_is_test_support",
        ],
    ),
    (
        "Runtime 15 M5 production dead-code suppression global gate",
        &[
            "runtime_15_production_dead_code_suppression_global_gate_static_passed_cargo_deferred",
            "structure_convention/runtime_dead_code/production_scan.rs",
            "DEAD_CODE_ALLOW_ATTRIBUTE",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
        ],
    ),
    (
        "Runtime 15 F12 dead-code review status sync",
        &[
            "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
            "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
        ],
    ),
    (
        "Runtime 15 F12 dead-code runtime/editor boundary status guard",
        &[
            "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
            "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "Runtime 15 + Editor UI 10 + convention",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
        ],
    ),
    (
        "Runtime 15 F12 production dead-code current-state wording cleanup",
        &[
            "runtime_15_f12_production_dead_code_current_state_wording_static_passed_cargo_deferred",
            "structure_convention/runtime_dead_code/status_anchor_cleanup.rs",
            "runtime_15_f12_production_dead_code_current_state_is_zero_hit",
            "runtime production `allow(dead_code)` 零命中",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
        ],
    ),
    (
        "Runtime 15 F12 UI text edit-state dead-code suppression cleanup",
        &[
            "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred",
            "ui/text/mod.rs",
            "ui/text/edit_state.rs",
            "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup",
        ],
    ),
    (
        "Runtime 15 UI boundary runtime-host forbidden attribute literal cleanup",
        &[
            "runtime_15_ui_boundary_runtime_host_literal_cleanup_static_passed_cargo_deferred",
            "tests/ui_boundary/runtime_host.rs",
            "DEAD_CODE_ALLOW_ATTRIBUTE",
            "runtime_ui_host_surface_splits_production_frame_from_test_support",
        ],
    ),
    (
        "Runtime 15 F1 native host callback panic guard",
        &[
            "runtime_15_native_host_callback_panic_guard_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/ffi_panic_guard.rs",
            "catch_native_host_api_panic",
            "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
        ],
    ),
    (
        "Runtime 15 graphics facade visibility note",
        &[
            "runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift",
            "graphics/mod.rs",
            "Public facade exports",
            "runtime_15_mixed_visibility_has_facade_note",
        ],
    ),
    (
        "Runtime 15 M1 graphics facade visibility review findings mirror",
        &[
            "runtime_15_graphics_facade_visibility_review_findings_mirror_static_passed_cargo_deferred",
            "runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "runtime_15_mixed_visibility_has_facade_note",
            "runtime_15_graphics_facade_visibility_review_findings_mirror_is_recorded",
        ],
    ),
    (
        "Runtime 15 F14 diagnostics normalization",
        &[
            "runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed",
            "FrameDiagnosticsStatus",
            "scene.ecs",
            "runtime_15_diagnostics_use_frame_trait_without_world_wrapper",
        ],
    ),
    (
        "Runtime 15 F13 provider registration shared owner",
        &[
            "runtime_15_provider_registration_shared_owner_coremin_check_passed",
            "graphics/runtime_provider/registration.rs",
            "RuntimeProviderRegistration<P: ?Sized>",
            "runtime_15_provider_registration_uses_shared_owner",
        ],
    ),
    (
        "Runtime 15 F13 provider update shared stats owner",
        &[
            "runtime_15_provider_update_shared_stats_owner_coremin_check_passed",
            "graphics/runtime_provider/update.rs",
            "RuntimeProviderUpdate<S>",
            "runtime_15_provider_update_uses_shared_stats_owner",
        ],
    ),
    (
        "Runtime 15 F13 provider feedback shared payload owner",
        &[
            "runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed",
            "graphics/runtime_provider/feedback.rs",
            "RuntimeProviderFeedback<G, V>",
            "runtime_15_provider_feedback_uses_shared_payload_owner",
        ],
    ),
    (
        "Runtime 15 F13 provider prepare input shared frame owner",
        &[
            "runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed",
            "graphics/runtime_provider/prepare_input.rs",
            "RuntimeProviderPrepareInput<'a, E>",
            "runtime_15_provider_prepare_input_uses_shared_extract_generation_owner",
        ],
    ),
    (
        "Runtime 15 F13 full provider boilerplate audit",
        &[
            "runtime_15_provider_boilerplate_full_audit_coremin_check_passed",
            "structure_convention/provider_boilerplate.rs",
            "RuntimeProviderRegistration<P: ?Sized>",
            "runtime_15_no_duplicated_provider_boilerplate",
        ],
    ),
    (
        "Runtime 15 F12 runtime-owned dead-code suppression cleanup",
        &[
            "runtime_15_runtime_owned_dead_code_suppression_cleanup_coremin_check_passed",
            "asset/pipeline/worker_pool.rs",
            "core/runtime/state/module_entry.rs",
            "runtime_15_runtime_owned_dead_code_suppression_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 script host value descriptor dead-code cleanup",
        &[
            "runtime_15_script_host_value_descriptors_coremin_check_passed",
            "script/vm/host/builtin_host_modules.rs",
            "docs/zircon_runtime/script/vm/host/function_ledger.md",
            "runtime_15_script_host_value_descriptors_do_not_suppress_dead_code",
        ],
    ),
    (
        "Runtime 15 F12 script reflection macro fixture dead-code cleanup",
        &[
            "runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred",
            "script/vm/tests/reflection_docs.rs",
            "docs/zircon_runtime/script/vm/zr_vm_host_reflection.md",
            "runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code",
        ],
    ),
    (
        "Runtime 15 M1 animation manager folder-backed cutover",
        &[
            "runtime_15_animation_manager_folder_backed_cutover_static_passed_cargo_deferred",
            "animation/manager/mod.rs",
            "animation/manager/graph.rs",
            "docs/zircon_runtime/animation/runtime.md",
            "runtime_15_animation_manager_is_folder_backed",
        ],
    ),
];
