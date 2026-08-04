use super::*;

#[test]
fn runtime_15_dynamic_scene_absorption_guard_is_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/dynamic_scene.rs");
    let patch_preview_api =
        read_runtime_src("tests/runtime_absorption/dynamic_scene/patch_preview_api.rs");
    let patch_preview_status_docs =
        read_runtime_src("tests/runtime_absorption/dynamic_scene/patch_preview_status_docs.rs");
    let patch_preview_behavior =
        read_runtime_src("tests/runtime_absorption/dynamic_scene/patch_preview_behavior.rs");
    let session_capture_persistence =
        read_runtime_src("tests/runtime_absorption/dynamic_scene/session_capture_persistence.rs");
    let session_retention_mutation_merge = read_runtime_src(
        "tests/runtime_absorption/dynamic_scene/session_retention_mutation_merge.rs",
    );
    let session_load_query_path =
        read_runtime_src("tests/runtime_absorption/dynamic_scene/session_load_query_path.rs");
    let asset_reload_selection_status =
        read_runtime_src("tests/runtime_absorption/dynamic_scene/asset_reload_selection_status.rs");
    let sources = read_runtime_src("tests/runtime_absorption/dynamic_scene/sources.rs");

    assert_contains_all(
        "dynamic-scene absorption parent mounts folder-backed children and shared sources",
        &parent,
        &[
            "mod asset_reload_selection_status;",
            "mod patch_preview_api;",
            "mod patch_preview_behavior;",
            "mod patch_preview_status_docs;",
            "mod session_capture_persistence;",
            "mod session_load_query_path;",
            "mod session_retention_mutation_merge;",
            "mod sources;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "tests/runtime_absorption/dynamic_scene.rs should only mount child owners and shared include_str sources"
    );
    assert_contains_all(
        "dynamic-scene absorption sources child owns shared include_str constants",
        &sources,
        &[
            "pub(super) const PATCH_SOURCE",
            "pub(super) const RUNTIME_05_PLAN",
            "pub(super) const DYNAMIC_SCENE_DOC",
        ],
    );
    assert!(
        !parent.contains("fn runtime_05_dynamic_scene_patch_preview_api_stays_read_only"),
        "the Runtime 05 dynamic-scene absorption guard should live in a child owner"
    );

    assert_contains_all(
        "patch-preview API child owns the original read-only preview guard",
        &patch_preview_api,
        &[
            "fn runtime_05_dynamic_scene_patch_preview_api_stays_read_only",
            "preview_scene_spawn_into must not call mutating apply helper",
            "ReflectError::MissingResource",
        ],
    );
    assert_contains_all(
        "patch-preview status child owns plan/index/doc anchors",
        &patch_preview_status_docs,
        &[
            "fn runtime_05_dynamic_scene_patch_preview_status_docs_stay_synced",
            "dynamic_scene_patch_preview_status_guard_static_passed_cargo_pending",
            "ScenePatchPreviewResource",
        ],
    );
    assert_contains_all(
        "patch-preview behavior child owns focused behavior anchors",
        &patch_preview_behavior,
        &[
            "fn runtime_05_dynamic_scene_patch_preview_behavior_anchors_stay_visible",
            "scene_patch_preview_reports_remaps_without_mutating_target_world",
            "preview.has_new_component_types()",
        ],
    );
    assert_contains_all(
        "session capture/persistence child owns preview and save anchors",
        &session_capture_persistence,
        &[
            "fn runtime_05_dynamic_scene_session_capture_persistence_anchors_stay_visible",
            "runtime_session_archive_world_capture_commit_matches_preview_generated_slot",
            "runtime_session_archive_preview_save_to_path_reports_targets_without_writing_files",
        ],
    );
    assert_contains_all(
        "session retention/mutation/merge child owns transactional anchors",
        &session_retention_mutation_merge,
        &[
            "fn runtime_05_dynamic_scene_session_retention_mutation_merge_anchors_stay_visible",
            "runtime_session_archive_preview_capture_retention_prunes_clone_without_mutating_archive",
            "runtime_session_archive_path_merge_preview_commit_and_same_path_guard_are_atomic",
        ],
    );
    assert_contains_all(
        "session load/query/path child owns path helper anchors",
        &session_load_query_path,
        &[
            "fn runtime_05_dynamic_scene_session_load_query_path_anchors_stay_visible",
            "runtime_session_archive_restores_slot_from_path_to_empty_world",
            "runtime_session_archive_previews_merge_from_path_without_mutating_archives",
        ],
    );
    assert_contains_all(
        "asset reload and status child owns reload, selection, and status anchors",
        &asset_reload_selection_status,
        &[
            "fn runtime_05_dynamic_scene_asset_reload_selection_and_status_anchors_stay_visible",
            "dynamic_scene_asset_reload_supersedes_older_pending_scene_revision",
            "runtime_session_archive_selected_transfer_helpers_use_resolved_slots",
        ],
    );

    let child_test_total = [
        patch_preview_api.as_str(),
        patch_preview_status_docs.as_str(),
        patch_preview_behavior.as_str(),
        session_capture_persistence.as_str(),
        session_retention_mutation_merge.as_str(),
        session_load_query_path.as_str(),
        asset_reload_selection_status.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 7,
        "dynamic-scene absorption children should decompose the single oversized guard into seven focused guards"
    );

    for (path, source) in [
        ("tests/runtime_absorption/dynamic_scene.rs", parent.as_str()),
        (
            "tests/runtime_absorption/dynamic_scene/patch_preview_api.rs",
            patch_preview_api.as_str(),
        ),
        (
            "tests/runtime_absorption/dynamic_scene/patch_preview_status_docs.rs",
            patch_preview_status_docs.as_str(),
        ),
        (
            "tests/runtime_absorption/dynamic_scene/patch_preview_behavior.rs",
            patch_preview_behavior.as_str(),
        ),
        (
            "tests/runtime_absorption/dynamic_scene/session_capture_persistence.rs",
            session_capture_persistence.as_str(),
        ),
        (
            "tests/runtime_absorption/dynamic_scene/session_retention_mutation_merge.rs",
            session_retention_mutation_merge.as_str(),
        ),
        (
            "tests/runtime_absorption/dynamic_scene/session_load_query_path.rs",
            session_load_query_path.as_str(),
        ),
        (
            "tests/runtime_absorption/dynamic_scene/asset_reload_selection_status.rs",
            asset_reload_selection_status.as_str(),
        ),
        (
            "tests/runtime_absorption/dynamic_scene/sources.rs",
            sources.as_str(),
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
    let dynamic_scene_doc = read_repo("docs/zircon_runtime/scene/dynamic_scene.md");
}
