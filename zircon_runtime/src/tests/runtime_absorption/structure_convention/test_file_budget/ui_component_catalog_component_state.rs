use super::*;

#[test]
fn runtime_15_ui_component_catalog_component_state_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/component_catalog/component_state.rs");
    let collection_mutation =
        read_runtime_src("ui/tests/component_catalog/component_state/collection_mutation.rs");
    let interaction_numeric =
        read_runtime_src("ui/tests/component_catalog/component_state/interaction_numeric.rs");
    let reference_sources =
        read_runtime_src("ui/tests/component_catalog/component_state/reference_sources.rs");
    let retained_events =
        read_runtime_src("ui/tests/component_catalog/component_state/retained_events.rs");

    assert_contains_all(
        "UI component state parent mounts folder-backed children",
        &parent,
        &[
            "mod collection_mutation;",
            "mod interaction_numeric;",
            "mod reference_sources;",
            "mod retained_events;",
            "mod keyboard;",
            "mod tree_view;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/component_catalog/component_state.rs should only mount child test owners"
    );
    for moved_test in [
        "component_state_applies_retained_number_dropdown_collection_and_drop_events",
        "drag_payload_source_metadata_roundtrips_and_summarizes",
        "component_state_edits_and_reorders_array_elements",
        "component_state_renames_map_keys_and_rejects_duplicate_targets",
        "component_state_handles_reference_actions_and_drop_rejection_feedback",
        "component_state_retains_reference_drop_source_metadata",
        "component_state_applies_transient_interaction_flags",
        "component_state_clamps_range_slider_thumbs_against_each_other",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI component state test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI component state retained child owns retained event contracts",
        &retained_events,
        &[
            "fn component_state_applies_retained_number_dropdown_collection_and_drop_events",
            "fn drag_payload_source_metadata_roundtrips_and_summarizes",
            "fn component_state_rejects_disabled_selection_options_with_validation_reason",
            "fn component_state_opens_context_action_menu_at_pointer_anchor",
        ],
    );
    assert_contains_all(
        "UI component state collection child owns collection mutation contracts",
        &collection_mutation,
        &[
            "fn component_state_edits_and_reorders_array_elements",
            "fn component_state_renames_map_keys_and_rejects_duplicate_targets",
            "fn component_state_sets_collection_validation_on_row_errors",
        ],
    );
    assert_contains_all(
        "UI component state reference child owns reference source contracts",
        &reference_sources,
        &[
            "fn component_state_handles_reference_actions_and_drop_rejection_feedback",
            "fn component_state_retains_reference_drop_source_metadata",
            "fn component_state_serializes_reference_sources_compatibly",
            "fn component_state_sourced_drop_reference_survives_serde_roundtrip",
            "fn component_state_clears_reference_source_on_sourceless_accepted_drop",
            "fn component_state_preserves_reference_source_on_rejected_drop",
        ],
    );
    assert_contains_all(
        "UI component state interaction/numeric child owns transient and numeric contracts",
        &interaction_numeric,
        &[
            "fn component_state_applies_transient_interaction_flags",
            "fn component_state_clears_reference_source_on_non_drop_value_replacement",
            "fn component_state_updates_existing_map_entries_without_creating_keys",
            "fn component_state_applies_numeric_state_step_large_step_and_clamp_settings",
            "fn component_state_clamps_range_slider_thumbs_against_each_other",
        ],
    );

    let child_test_total = [
        collection_mutation.as_str(),
        interaction_numeric.as_str(),
        reference_sources.as_str(),
        retained_events.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 18,
        "UI component state children should preserve all 18 parent tests"
    );

    for (path, source) in [
        (
            "ui/tests/component_catalog/component_state.rs",
            parent.as_str(),
        ),
        (
            "ui/tests/component_catalog/component_state/collection_mutation.rs",
            collection_mutation.as_str(),
        ),
        (
            "ui/tests/component_catalog/component_state/interaction_numeric.rs",
            interaction_numeric.as_str(),
        ),
        (
            "ui/tests/component_catalog/component_state/reference_sources.rs",
            reference_sources.as_str(),
        ),
        (
            "ui/tests/component_catalog/component_state/retained_events.rs",
            retained_events.as_str(),
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
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI component state test folder split",
                "runtime_15_ui_component_catalog_component_state_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/component_catalog/component_state.rs",
                "ui/tests/component_catalog/component_state/reference_sources.rs",
                "ui/tests/component_catalog/component_state/interaction_numeric.rs",
                "runtime_15_ui_component_catalog_component_state_tests_are_folder_backed",
            ],
        );
    }
}
