use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

const DIRECT_REVIEW_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
const P0_DIRECT_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0.rs";
const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) fn assert_p0_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    assert_contains_all(
        "P0 robustness parent only mounts focused child review guard owners",
        &sources.p0_robustness,
        &[
            "#[path = \"p0_robustness/native_host_callbacks.rs\"]",
            "mod native_host_callbacks;",
            "#[path = \"p0_robustness/lock_poison.rs\"]",
            "mod lock_poison;",
            "#[path = \"p0_robustness/render_submit.rs\"]",
            "mod render_submit;",
            "#[path = \"p0_robustness/native_fixture.rs\"]",
            "mod native_fixture;",
            "#[path = \"p0_robustness/priority_recommendation.rs\"]",
            "mod priority_recommendation;",
        ],
    );
    assert_eq!(
        sources.p0_robustness.matches("#[test]").count(),
        0,
        "p0_robustness.rs should only mount child review guard owners"
    );
    for moved_test in [
        "fn review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
        "fn review_f2_scene_eventbus_locks_recover_after_poison",
        "fn review_f4_render_submit_capability_gaps_return_typed_errors",
        "fn review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
        "fn review_d13_native_fixture_importer_is_manifest_described",
        "fn review_priority_recommendation_tracks_current_remaining_work",
    ] {
        assert!(
            !sources.p0_robustness.contains(moved_test),
            "P0 robustness parent should not keep child-owned review guard `{moved_test}`"
        );
    }
    assert_contains_all(
        "P0 native host callback child owns F1 panic-boundary review guard",
        &sources.p0_native_host_callbacks,
        &[
            "fn review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
            "catch_native_host_api_panic",
            "ZIRCON_NATIVE_PLUGIN_STATUS_PANIC",
            "p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "P0 lock poison child owns F2 scene/EventBus review guard",
        &sources.p0_lock_poison,
        &[
            "fn review_f2_scene_eventbus_locks_recover_after_poison",
            "runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending",
            "level_system_accessors_recover_poisoned_state_locks",
            "scene/EventBus poison-safe lock recovery complete",
        ],
    );
    assert_contains_all(
        "P0 render submit child owns F4 viewport/provider typed-error review guard",
        &sources.p0_render_submit,
        &[
            "fn review_f4_render_submit_capability_gaps_return_typed_errors",
            "RenderFrameworkError::UnsupportedCapability",
            "viewport_record_mut_after_generation_check",
            "review_f4_render_submit_capability_gaps_return_typed_errors",
        ],
    );
    assert_contains_all(
        "P0 native fixture child owns D-S8/D3/D13 fixture review guards",
        &sources.p0_native_fixture,
        &[
            "#[path = \"native_fixture/importer_manifest.rs\"]",
            "mod importer_manifest;",
            "#[path = \"native_fixture/sdk_macro_manifest.rs\"]",
            "mod sdk_macro_manifest;",
        ],
    );
    assert_eq!(
        sources.p0_native_fixture.matches("#[test]").count(),
        0,
        "p0 native_fixture.rs should only mount D-S8/D3/D13 review guard leaf owners"
    );
    assert_contains_all(
        "P0 native fixture leaf owners keep D-S8/D3/D13 fixture review guards",
        &[
            sources.p0_native_fixture_sdk_macro.as_str(),
            sources.p0_native_fixture_importer.as_str(),
        ]
        .join("\n"),
        &[
            "fn review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
            "fn review_d13_native_fixture_importer_is_manifest_described",
            "zircon_plugin_sdk::native_dist_plugin_v3!",
            "native_dynamic_fixture_importer_manifest_self_description_static_passed_cargo_deferred",
            "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "P0 priority recommendation child owns current remaining-work review guard",
        &sources.p0_priority_recommendation,
        &[
            "fn review_priority_recommendation_tracks_current_remaining_work",
            "ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred",
            "d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred",
            "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred",
            "review_priority_recommendation_d13_parity_sync_static_passed_cargo_deferred",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_p0_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(P0_DIRECT_ASSERTIONS_CHILD);
    let sources = super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "direct-review assertion child delegates P0 assertions to child owner",
        &parent,
        &[
            "#[path = \"direct_review_assertions/p0.rs\"]",
            "mod p0;",
            "p0::assert_p0_direct_sources_are_folder_backed",
        ],
    );
    for moved_guard in [
        concat!(
            "P0 robustness parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "P0 native host callback child owns F1 panic-boundary ",
            "review guard"
        ),
        concat!(
            "P0 native fixture leaf owners keep D-S8/D3/D13 fixture ",
            "review guards"
        ),
        concat!(
            "review_priority_recommendation_",
            "tracks_current_remaining_work"
        ),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "P0 direct assertion `{moved_guard}` should stay in {P0_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "P0 direct assertion child owns P0 source checks",
        &child,
        &[
            "pub(super) fn assert_p0_direct_sources_are_folder_backed",
            "P0 robustness parent only mounts focused child review guard owners",
            "P0 native host callback child owns F1 panic-boundary review guard",
            "P0 lock poison child owns F2 scene/EventBus review guard",
            "P0 render submit child owns F4 viewport/provider typed-error review guard",
            "P0 native fixture child owns D-S8/D3/D13 fixture review guards",
            "P0 native fixture leaf owners keep D-S8/D3/D13 fixture review guards",
            "P0 priority recommendation child owns current remaining-work review guard",
            "review_priority_recommendation_tracks_current_remaining_work",
        ],
    );

    assert_p0_direct_sources_are_folder_backed(&sources);

    for (path, source) in [
        (DIRECT_REVIEW_ASSERTIONS_CHILD, parent.as_str()),
        (P0_DIRECT_ASSERTIONS_CHILD, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
