use super::sources::*;

#[test]
fn runtime_05_dynamic_scene_session_retention_mutation_merge_anchors_stay_visible() {
    for anchor in [
        "runtime_session_archive_preview_capture_retention_projects_without_mutating_archive",
        "preview_capture_world_slot_with_tag_retention(",
        "preview.prune.removed_slot_ids",
        "preview.manifest.slot_ids().collect::<Vec<_>>()",
        "!archive.contains_slot(\"autosave-new\")",
        "runtime_session_archive_capture_retention_protects_captured_slot_before_pruning",
        "capture_world_slot_with_retention(",
        "RuntimeSessionArchiveRetentionPolicy::keep_latest(0)",
        "report.prune.removed_slot_ids",
        "archive.slot_ids().collect::<Vec<_>>()",
    ] {
        assert!(
            RETENTION_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 capture-retention transaction behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_selected_retention_protects_latest_tagged_slot",
        "preview_prune_slots_with_selected_protection(",
        "RuntimeSessionSlotSelector::latest_updated_with_tag(\" manual \")",
        "prune_slots_with_selected_protection(",
        "runtime_session_archive_tag_selected_retention_ignores_protection_outside_bucket",
        "preview_prune_slots_with_tag_and_selected_protection(",
        "RuntimeSessionSlotSelector::slot_id(\" manual-protected \")",
        "prune_slots_with_tag_and_selected_protection(",
        "runtime_session_archive_path_selected_retention_preview_does_not_write_archive",
        "preview_prune_slots_with_selected_protection_from_path(",
        "RuntimeSessionSlotSelector::oldest_updated_with_tag(\" manual \")",
        "temporary_archive_leftovers",
    ] {
        assert!(
            RETENTION_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 selected retention behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_named_mutation_commits_preserve_preview_boundaries",
        "preview_rename_slot(\"manual\", \" manual-renamed \")",
        "rename_slot(\"manual\", \" manual-renamed \")",
        "RuntimeSessionArchiveError::DuplicateSlotId",
        "runtime_session_archive_selected_mutations_resolve_targets_before_committing",
        "preview_rename_selected_slot(",
        "RuntimeSessionSlotSelector::latest_updated_with_tag(\" manual \")",
        "preview_remove_selected_slot(",
        "remove_selected_slot(",
        "runtime_session_archive_selected_path_mutations_preview_and_commit_atomically",
        "preview_rename_selected_slot_from_path(",
        "rename_selected_slot_at_path_atomically(",
        "preview_remove_selected_slot_from_path(",
        "remove_selected_slot_at_path_atomically(",
        "temporary_archive_leftovers",
    ] {
        assert!(
            MUTATION_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 mutation behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_merge_preview_and_keep_existing_commit_are_side_effect_free",
        "preview_merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)",
        "merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)",
        "preview.inserted_slot_ids",
        "incoming.slot_ids().collect::<Vec<_>>()",
        "runtime_session_archive_path_merge_preview_commit_and_same_path_guard_are_atomic",
        "preview_merge_archive_at_path(",
        "preview_merge_archive_from_path_at_path(",
        "merge_archive_from_path_at_path_atomically(",
        "RuntimeSessionArchiveMergePolicy::ReplaceExisting",
        "assert_same_path_merge_rejected",
        "target archive path must differ from source archive path",
        "temporary_archive_leftovers(",
    ] {
        assert!(
            MERGE_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 merge behavior source should keep anchor `{anchor}`"
        );
    }
}
