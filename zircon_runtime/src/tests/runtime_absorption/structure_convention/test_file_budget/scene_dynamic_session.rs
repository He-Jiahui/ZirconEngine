use super::*;

#[test]
fn runtime_15_dynamic_scene_session_path_management_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/dynamic_scene_session/path_management.rs");
    let archive_merge =
        read_runtime_src("scene/tests/dynamic_scene_session/path_management/archive_merge.rs");
    let mutation_previews =
        read_runtime_src("scene/tests/dynamic_scene_session/path_management/mutation_previews.rs");
    let single_slot_import =
        read_runtime_src("scene/tests/dynamic_scene_session/path_management/single_slot_import.rs");
    let single_slot_save =
        read_runtime_src("scene/tests/dynamic_scene_session/path_management/single_slot_save.rs");
    let slot_copy =
        read_runtime_src("scene/tests/dynamic_scene_session/path_management/slot_copy.rs");
    let slot_mutations =
        read_runtime_src("scene/tests/dynamic_scene_session/path_management/slot_mutations.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let dynamic_scene_doc = read_repo("docs/zircon_runtime/scene/dynamic_scene.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "dynamic scene session path-management parent test module mounts",
        &parent,
        &[
            "mod archive_merge;",
            "mod mutation_previews;",
            "mod single_slot_import;",
            "mod single_slot_save;",
            "mod slot_copy;",
            "mod slot_mutations;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/dynamic_scene_session/path_management.rs should mount child owners instead of keeping executable tests"
    );

    for moved_test in [
        "fn runtime_session_archive_renames_slot_at_path_atomically",
        "fn runtime_session_archive_previews_slot_mutations_without_mutating_archive",
        "fn runtime_session_archive_copies_slot_at_path_atomically",
        "fn runtime_session_archive_imports_single_slot_at_path_atomically",
        "fn runtime_session_archive_saves_single_slot_archive_from_path_atomically",
        "fn runtime_session_archive_merges_archive_at_path_atomically",
    ] {
        assert!(
            !parent.contains(moved_test),
            "dynamic_scene_session/path_management.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "slot mutations child owns path rename metadata touch and remove commits",
        &slot_mutations,
        &[
            "use super::*;",
            "fn runtime_session_archive_renames_slot_at_path_atomically",
            "fn runtime_session_archive_updates_slot_metadata_at_path_atomically",
            "fn runtime_session_archive_touches_slot_at_path_atomically",
            "fn runtime_session_archive_removes_slot_at_path_atomically",
        ],
    );
    assert_contains_all(
        "mutation previews child owns no-write path mutation previews",
        &mutation_previews,
        &[
            "use super::*;",
            "fn runtime_session_archive_previews_slot_mutations_without_mutating_archive",
            "fn runtime_session_archive_previews_slot_mutations_from_path_without_mutating_archive",
        ],
    );
    assert_contains_all(
        "slot copy child owns path copy commit and preview contracts",
        &slot_copy,
        &[
            "use super::*;",
            "fn runtime_session_archive_copies_slot_at_path_atomically",
            "fn runtime_session_archive_previews_slot_copy_without_mutating_archive",
            "fn runtime_session_archive_previews_slot_copy_from_path_without_mutating_archive",
        ],
    );
    assert_contains_all(
        "single-slot import child owns loaded and source-path import contracts",
        &single_slot_import,
        &[
            "use super::*;",
            "fn runtime_session_archive_imports_single_slot_at_path_atomically",
            "fn runtime_session_archive_imports_single_slot_from_path_at_path_atomically",
            "fn runtime_session_archive_previews_single_slot_import_without_mutating_archives",
            "fn runtime_session_archive_previews_single_slot_import_from_path_without_mutating_archives",
        ],
    );
    assert_contains_all(
        "single-slot save child owns standalone archive save contracts",
        &single_slot_save,
        &[
            "use super::*;",
            "fn runtime_session_archive_saves_single_slot_archive_from_path_atomically",
            "fn runtime_session_archive_saves_single_slot_archive_from_memory_atomically",
        ],
    );
    assert_contains_all(
        "archive merge child owns path merge commit and preview contracts",
        &archive_merge,
        &[
            "use super::*;",
            "fn runtime_session_archive_merges_archive_at_path_atomically",
            "fn runtime_session_archive_merges_archive_from_path_at_path_atomically",
            "fn runtime_session_archive_previews_merge_without_mutating_archives",
            "fn runtime_session_archive_previews_merge_from_path_without_mutating_archives",
        ],
    );

    let migrated_test_count = [
        slot_mutations.as_str(),
        mutation_previews.as_str(),
        slot_copy.as_str(),
        single_slot_import.as_str(),
        single_slot_save.as_str(),
        archive_merge.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 19,
        "dynamic scene session path-management child owners should preserve all 19 tests moved out of the parent"
    );

    for (path, source) in [
        (
            "scene/tests/dynamic_scene_session/path_management.rs",
            parent.as_str(),
        ),
        (
            "scene/tests/dynamic_scene_session/path_management/archive_merge.rs",
            archive_merge.as_str(),
        ),
        (
            "scene/tests/dynamic_scene_session/path_management/mutation_previews.rs",
            mutation_previews.as_str(),
        ),
        (
            "scene/tests/dynamic_scene_session/path_management/single_slot_import.rs",
            single_slot_import.as_str(),
        ),
        (
            "scene/tests/dynamic_scene_session/path_management/single_slot_save.rs",
            single_slot_save.as_str(),
        ),
        (
            "scene/tests/dynamic_scene_session/path_management/slot_copy.rs",
            slot_copy.as_str(),
        ),
        (
            "scene/tests/dynamic_scene_session/path_management/slot_mutations.rs",
            slot_mutations.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("dynamic scene doc", dynamic_scene_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 dynamic scene session path-management test folder split",
                "runtime_15_dynamic_scene_session_path_management_tests_folder_split_static_passed_cargo_deferred",
                "scene/tests/dynamic_scene_session/path_management.rs",
                "scene/tests/dynamic_scene_session/path_management/single_slot_import.rs",
                "scene/tests/dynamic_scene_session/path_management/archive_merge.rs",
                "runtime_15_dynamic_scene_session_path_management_tests_are_folder_backed",
            ],
        );
    }
}
