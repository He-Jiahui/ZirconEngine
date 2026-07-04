use super::*;

#[test]
fn runtime_05_dynamic_scene_session_load_query_path_anchors_stay_visible() {
    for anchor in [
        "runtime_session_archive_restores_slot_from_path_to_empty_world",
        "restore_slot_from_path_to_empty_world(&path, \"manual\")",
        "runtime_session_archive_restores_slot_from_path_into_level_and_applies_metadata",
        "restore_slot_from_path_into_level(&path, \"level\", &level)",
        "stale_entity",
        "runtime_session_archive_applies_slot_from_path_to_live_world_and_level",
        "apply_slot_from_path_to_world(&path, \"prefab\", &mut world)",
        "apply_slot_from_path_to_level(&path, \"prefab\", &level)",
        "mapped_world_entity",
        "mapped_level_entity",
        "runtime_session_archive_path_load_helpers_report_missing_slot",
        "RuntimeSessionArchiveError::MissingSlot",
    ] {
        assert!(
            LOAD_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 load behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_loads_statistics_from_path",
        "statistics_from_path(&path)",
        "runtime_session_archive_reads_slot_summaries_directly_from_path",
        "slot_ids_from_path(&path)",
        "contains_slot_from_path(&path, \"manual\")",
        "slot_summary_from_path(&path, \"manual\")",
        "runtime_session_archive_diffs_slot_from_path_without_mutating_target",
        "diff_slot_from_path_with_world(&path, \"manual\", &target)",
        "diff_slot_from_path_with_level(&path, \"manual\", &level)",
        "runtime_session_archive_previews_path_retention_without_saving",
        "preview_prune_slots_from_path(",
        "preview_prune_slots_with_tag_from_path(",
        "runtime_session_archive_selects_updated_slots_directly_from_path",
        "latest_updated_slot_id_from_path(&path)",
        "oldest_updated_slot_id_from_path(&path)",
        "latest_updated_slot_id_with_tag_from_path(&path, \" manual \")",
        "oldest_updated_slot_id_with_tag_from_path(&path, \"manual\")",
        "runtime_session_archive_filters_manifest_summaries_directly_from_path",
        "slots_with_tag_from_path(&path, \" manual \")",
        "slots_matching_display_name_from_path(&path, \"Two\")",
        "archive payload should remain readable after query",
        "archive payload should remain readable after preview",
        "archive payload should remain readable after selection",
        "archive payload should remain readable after filter",
        "temporary_archive_leftovers(",
    ] {
        assert!(
            QUERIES_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 query behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_renames_slot_at_path_atomically",
        "rename_slot_at_path_atomically(&path, \"manual-old\", \" manual-new \")",
        "runtime_session_archive_updates_slot_metadata_at_path_atomically",
        "update_slot_metadata_at_path_atomically(",
        "runtime_session_archive_touches_slot_at_path_atomically",
        "touch_slot_at_path_atomically(&path, \"manual\", 90)",
        "runtime_session_archive_removes_slot_at_path_atomically",
        "remove_slot_at_path_atomically(&path, \"autosave\")",
        "runtime_session_archive_previews_slot_mutations_without_mutating_archive",
        "preview_rename_slot(\"manual\", \" manual-renamed \")",
        "RuntimeSessionArchiveError::DuplicateSlotId",
        "runtime_session_archive_previews_slot_mutations_from_path_without_mutating_archive",
        "preview_update_slot_metadata_from_path(",
        "preview_remove_slot_from_path(&path, \"autosave\")",
        "runtime_session_archive_copies_slot_at_path_atomically",
        "copy_slot_with_metadata_at_path_atomically(",
        "runtime_session_archive_previews_slot_copy_without_mutating_archive",
        "preview_copy_slot_with_metadata(",
        "runtime_session_archive_previews_slot_copy_from_path_without_mutating_archive",
        "preview_copy_slot_with_metadata_from_path(",
        "runtime_session_archive_imports_single_slot_at_path_atomically",
        "import_slot_from_archive_with_metadata_at_path_atomically(",
        "runtime_session_archive_imports_single_slot_from_path_at_path_atomically",
        "import_slot_from_archive_path_with_metadata_at_path_atomically(",
        "runtime_session_archive_previews_single_slot_import_without_mutating_archives",
        "preview_import_slot_from_archive_with_metadata(",
        "runtime_session_archive_previews_single_slot_import_from_path_without_mutating_archives",
        "preview_import_slot_from_archive_path_with_metadata_at_path(",
        "runtime_session_archive_saves_single_slot_archive_from_path_atomically",
        "save_single_slot_archive_from_path_atomically(",
        "runtime_session_archive_saves_single_slot_archive_from_memory_atomically",
        "save_single_slot_archive_to_path_atomically(",
        "runtime_session_archive_merges_archive_at_path_atomically",
        "merge_archive_at_path_atomically(",
        "runtime_session_archive_merges_archive_from_path_at_path_atomically",
        "merge_archive_from_path_at_path_atomically(",
        "runtime_session_archive_previews_merge_without_mutating_archives",
        "preview_merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)",
        "runtime_session_archive_previews_merge_from_path_without_mutating_archives",
        "preview_merge_archive_from_path_at_path(",
        "temporary_archive_leftovers(",
    ] {
        let path_management_sources = [
            PATH_MANAGEMENT_BEHAVIOR_SOURCE,
            PATH_MANAGEMENT_ARCHIVE_MERGE_SOURCE,
            PATH_MANAGEMENT_MUTATION_PREVIEWS_SOURCE,
            PATH_MANAGEMENT_SINGLE_SLOT_IMPORT_SOURCE,
            PATH_MANAGEMENT_SINGLE_SLOT_SAVE_SOURCE,
            PATH_MANAGEMENT_SLOT_COPY_SOURCE,
            PATH_MANAGEMENT_SLOT_MUTATIONS_SOURCE,
        ];
        assert!(
            path_management_sources
                .iter()
                .any(|source| source.contains(anchor)),
            "Runtime 05 path-management behavior source should keep anchor `{anchor}`"
        );
    }
}
