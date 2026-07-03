use super::*;

pub(super) fn assert_p0_robustness_child_owners_are_folder_backed() {
    let sources = read_p0_robustness_sources();

    assert_contains_all(
        "P0 robustness parent mounts focused child owners",
        &sources.parent,
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
        sources.parent.matches("#[test]").count(),
        0,
        "p0_robustness.rs should only mount child review guard owners"
    );
    for child_owned_test in REVIEW_GUARDS {
        assert!(
            !sources.parent.contains(&format!("fn {child_owned_test}")),
            "child-owned P0 review guard `{child_owned_test}` should not return to p0_robustness.rs"
        );
    }
    assert_contains_all(
        "native host callback child owns F1 panic-boundary review guard",
        &sources.native_host_callbacks,
        &[
            REVIEW_GUARDS[0],
            "catch_native_host_api_panic",
            "ZIRCON_NATIVE_PLUGIN_STATUS_PANIC",
            "p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "lock poison child owns F2 scene/EventBus review guard",
        &sources.lock_poison,
        &[
            REVIEW_GUARDS[1],
            "runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending",
            "level_system_accessors_recover_poisoned_state_locks",
            "scene/EventBus poison-safe lock recovery complete",
        ],
    );
    assert_contains_all(
        "render submit child owns F4 viewport/provider typed-error review guard",
        &sources.render_submit,
        &[
            REVIEW_GUARDS[2],
            "RenderFrameworkError::UnsupportedCapability",
            "viewport_record_mut_after_generation_check",
            "review_f4_render_submit_capability_gaps_return_typed_errors",
        ],
    );
    assert_contains_all(
        "native fixture child owns fixture review sync guards",
        &sources.native_fixture,
        &[
            "#[path = \"native_fixture/importer_manifest.rs\"]",
            "mod importer_manifest;",
            "#[path = \"native_fixture/sdk_macro_manifest.rs\"]",
            "mod sdk_macro_manifest;",
        ],
    );
    assert_eq!(
        sources.native_fixture.matches("#[test]").count(),
        0,
        "native_fixture.rs should only mount native fixture review guard leaf owners"
    );
    assert_contains_all(
        "native fixture SDK macro leaf owns D-S8/D3 review sync guard",
        &sources.native_fixture_sdk_macro,
        &[
            REVIEW_GUARDS[3],
            "zircon_plugin_sdk::native_dist_plugin_v3!",
            "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "native fixture importer leaf owns D13 manifest self-description guard",
        &sources.native_fixture_importer,
        &[
            REVIEW_GUARDS[4],
            "native_dynamic_fixture_importer_manifest_self_description_static_passed_cargo_deferred",
            "runtime.asset.importer.native_dynamic_fixture.data_json",
        ],
    );
    assert_contains_all(
        "priority recommendation child owns cross-review priority sync",
        &sources.priority_recommendation,
        &[
            REVIEW_GUARDS[5],
            "review_priority_recommendation_d13_parity_sync_static_passed_cargo_deferred",
            "d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred",
            "d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred",
        ],
    );
}

#[test]
fn runtime_15_p0_robustness_review_guards_are_child_owners() {
    assert_p0_robustness_child_owners_are_folder_backed();
}
