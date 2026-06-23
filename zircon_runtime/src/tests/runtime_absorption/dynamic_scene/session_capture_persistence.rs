use super::*;

#[test]
fn runtime_05_dynamic_scene_session_capture_persistence_anchors_stay_visible() {
    for anchor in [
        "runtime_session_archive_world_capture_commit_matches_preview_generated_slot",
        "preview_capture_world_slot(\" manual \", &source, metadata.clone())",
        "capture_world_slot(\" manual \", &source, metadata)",
        "committed_summary.metadata, preview.metadata",
        "committed_summary.entity_count, preview.entity_count",
        "committed_summary.resource_count, preview.resource_count",
    ] {
        assert!(
            CAPTURE_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 capture preview/commit parity behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_level_capture_preview_preserves_from_level_semantics",
        "RuntimeSessionSlot::from_level(\" level-preview \", &level)",
        "preview_capture_level_slot(\" level-preview \", &level)",
        "preview.metadata, expected_slot.metadata",
        "preview.entity_count, expected_slot.scene.entities.len()",
        "preview.resource_count, expected_slot.scene.resources.len()",
    ] {
        assert!(
            CAPTURE_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 level capture preview/from_level behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_capture_retention_reuses_shared_preview_report_projection",
        "preview_capture_world_slot(\" manual-mid \", &captured_world, metadata.clone())",
        "preview_capture_world_slot_with_tag_retention(",
        "retention_preview.capture, capture_preview",
        "retention_preview.prune.removed_slot_ids",
        "retention_preview.manifest.slot_ids().collect::<Vec<_>>()",
        "archive.slot_ids().collect::<Vec<_>>()",
    ] {
        assert!(
            CAPTURE_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 capture-retention shared preview behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_selected_capture_targets_resolved_slot_and_preserves_metadata",
        "preview_capture_world_selected_slot(",
        "capture_world_selected_slot(",
        "capture_world_selected_slot_preserving_metadata(",
        "RuntimeSessionSlotSelector::latest_updated_with_tag(\" manual \")",
        "RuntimeSessionSlotSelector::oldest_updated_with_tag(\"manual\")",
        "preserved.metadata.updated_at_unix_millis, Some(10)",
        "runtime_session_archive_selected_capture_to_path_previews_and_prunes_atomically",
        "preview_capture_world_selected_slot_with_tag_retention_to_path(",
        "capture_world_selected_slot_with_tag_retention_to_path_atomically(",
        "preview.prune.removed_slot_ids",
        "archive payload should remain after preview",
        "temporary_archive_leftovers(",
    ] {
        assert!(
            CAPTURE_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 selected capture behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_preview_save_to_path_reports_targets_without_writing_files",
        "preview_save_to_path(&missing_path)",
        "missing_target.will_replace_target",
        "missing_target.statistics.slot_count",
        "preview_save_to_path(&existing_path)",
        "existing_target.will_replace_target",
        "fs::read_to_string(&existing_path)",
        "preview_save_to_path(&directory_target)",
        "RuntimeSessionArchiveError::Io(error)",
        "temporary_archive_leftovers",
    ] {
        assert!(
            PERSISTENCE_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 full-save preview behavior source should keep anchor `{anchor}`"
        );
    }
    for anchor in [
        "runtime_session_archive_preview_save_to_path_rejects_parent_file_without_writes",
        "preview_save_to_path(&target_path)",
        "parent_file_target",
        "fs::read_to_string(&parent_file)",
        "parent_file.is_file()",
        "!target_path.exists()",
    ] {
        assert!(
            PERSISTENCE_BEHAVIOR_SOURCE.contains(anchor),
            "Runtime 05 target parent-file preview behavior source should keep anchor `{anchor}`"
        );
    }
}
