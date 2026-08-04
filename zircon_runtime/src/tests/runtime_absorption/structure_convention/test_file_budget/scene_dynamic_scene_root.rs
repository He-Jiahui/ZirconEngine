use super::*;

#[test]
fn runtime_15_dynamic_scene_root_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/dynamic_scene.rs");
    let archive_core = read_runtime_src("scene/tests/dynamic_scene/archive_core.rs");
    let archive_manifest = read_runtime_src("scene/tests/dynamic_scene/archive_manifest.rs");
    let archive_mutation = read_runtime_src("scene/tests/dynamic_scene/archive_mutation.rs");
    let level_apply = read_runtime_src("scene/tests/dynamic_scene/level_apply.rs");
    let scene_patch_document =
        read_runtime_src("scene/tests/dynamic_scene/scene_patch_document.rs");

    assert_contains_all(
        "dynamic-scene root parent mounts folder-backed children",
        &parent,
        &[
            "mod archive_core;",
            "mod archive_manifest;",
            "mod archive_mutation;",
            "mod level_apply;",
            "mod scene_patch_document;",
            "fn cloud_layer_descriptor()",
            "fn register_frame_counter_resource(",
            "fn frame_counter_adapter()",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/dynamic_scene.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "dynamic_scene_roundtrips_reflected_components_with_entity_remap",
        "runtime_session_archive_roundtrips_slots_and_restores_world",
        "runtime_session_archive_merges_archives_with_explicit_conflict_policy",
        "runtime_session_archive_manifest_summarizes_sorted_slots",
        "runtime_session_archive_restores_slot_into_level_and_resets_runtime_state",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved dynamic-scene test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "scene-patch/document child owns scene patch and JSON tests",
        &scene_patch_document,
        &[
            "fn dynamic_scene_roundtrips_reflected_components_with_entity_remap",
            "fn scene_patch_applies_reflected_resources",
            "fn scene_patch_preview_reports_remaps_without_mutating_target_world",
            "fn dynamic_scene_world_mutation_preserves_scene_error_source",
            "fn versioned_json_migrates_legacy_world_project_documents",
        ],
    );
    assert_contains_all(
        "archive-core child owns archive validation and serialization tests",
        &archive_core,
        &[
            "fn runtime_session_archive_roundtrips_slots_and_restores_world",
            "fn runtime_session_archive_rejects_duplicate_slots",
            "fn runtime_session_archive_normalizes_noncanonical_inner_scene_versions",
            "fn runtime_session_archive_normalizes_noncanonical_inner_versions_on_push_and_upsert",
            "fn runtime_session_archive_rejects_non_canonical_slot_ids",
            "fn runtime_session_archive_serializes_manual_slots_in_canonical_order",
            "fn runtime_session_archive_normalizes_metadata_tags_for_manifest_and_json",
        ],
    );
    assert_contains_all(
        "archive-mutation child owns mutation, merge, retention, diff, and surface tests",
        &archive_mutation,
        &[
            "fn runtime_session_archive_renames_slots_and_updates_metadata",
            "fn runtime_session_archive_copies_slots_with_metadata_override",
            "fn runtime_session_archive_merges_archives_with_explicit_conflict_policy",
            "fn runtime_session_archive_prunes_old_slots_with_retention_policy",
            "fn runtime_session_archive_touches_slot_update_time_without_replacing_metadata",
            "fn runtime_session_archive_diffs_slots_against_worlds",
            "fn runtime_session_archive_keeps_slot_mutation_surface_guarded",
        ],
    );
    assert_contains_all(
        "archive-manifest child owns manifest and projection tests",
        &archive_manifest,
        &[
            "fn runtime_session_archive_statistics_summarizes_slots_without_restoring_worlds",
            "fn runtime_session_archive_selects_latest_and_oldest_updated_slots_without_restoring_worlds",
            "fn runtime_session_archive_manifest_summarizes_sorted_slots",
            "fn runtime_session_archive_manifest_filters_slots_without_restoring_worlds",
            "fn runtime_session_archive_selects_latest_and_oldest_updated_slots_by_tag",
            "fn runtime_session_archive_upsert_replaces_slot_summary",
        ],
    );
    assert_contains_all(
        "level-apply child owns level restore/apply tests",
        &level_apply,
        &[
            "fn runtime_session_archive_restores_slot_into_level_and_resets_runtime_state",
            "fn runtime_session_archive_applies_slot_to_live_level_with_entity_remap",
        ],
    );

    let child_test_total = [
        archive_core.as_str(),
        archive_manifest.as_str(),
        archive_mutation.as_str(),
        level_apply.as_str(),
        scene_patch_document.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 28,
        "dynamic_scene children should preserve all 28 tests"
    );

    for (path, source) in [
        ("scene/tests/dynamic_scene.rs", parent.as_str()),
        (
            "scene/tests/dynamic_scene/archive_core.rs",
            archive_core.as_str(),
        ),
        (
            "scene/tests/dynamic_scene/archive_manifest.rs",
            archive_manifest.as_str(),
        ),
        (
            "scene/tests/dynamic_scene/archive_mutation.rs",
            archive_mutation.as_str(),
        ),
        (
            "scene/tests/dynamic_scene/level_apply.rs",
            level_apply.as_str(),
        ),
        (
            "scene/tests/dynamic_scene/scene_patch_document.rs",
            scene_patch_document.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let dynamic_scene_doc = read_repo("docs/zircon_runtime/scene/dynamic_scene.md");
}
