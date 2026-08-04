use std::fs;

use crate::scene::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector, World,
};

use super::{tagged_slot, temporary_archive_leftovers, unique_temp_root};

#[test]
fn runtime_session_archive_named_mutation_commits_preserve_preview_boundaries() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 20),
    ])
    .expect("archive should validate named mutation fixture slots");

    let rename_preview = archive
        .preview_rename_slot("manual", " manual-renamed ")
        .expect("rename preview should normalize destination");
    assert_eq!(rename_preview.source_slot_id, "manual");
    assert_eq!(
        rename_preview.destination_slot_id.as_deref(),
        Some("manual-renamed")
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );
    let revision_before_rename = archive.revision();

    archive
        .rename_slot("manual", " manual-renamed ")
        .expect("rename commit should update slot id");
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-renamed"]
    );
    assert_eq!(archive.revision(), revision_before_rename + 1);
    let revision_after_rename = archive.revision();

    let duplicate = archive.rename_slot("manual-renamed", " autosave ");
    assert!(matches!(
        duplicate,
        Err(RuntimeSessionArchiveError::DuplicateSlotId { slot_id }) if slot_id == "autosave"
    ));
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-renamed"]
    );
    assert_eq!(archive.revision(), revision_after_rename);

    let metadata_preview = archive
        .preview_update_slot_metadata(
            "manual-renamed",
            RuntimeSessionMetadata::default()
                .with_display_name("Renamed Manual")
                .with_tag(" manual ")
                .with_tag("reviewed")
                .with_tag("reviewed")
                .with_updated_at_unix_millis(70),
        )
        .expect("metadata preview should normalize replacement metadata");
    assert_eq!(metadata_preview.source_slot_id, "manual-renamed");
    assert_eq!(metadata_preview.destination_slot_id, None);
    assert_eq!(metadata_preview.metadata.tags, vec!["manual", "reviewed"]);
    assert_eq!(
        archive
            .slot("manual-renamed")
            .expect("renamed slot should remain before metadata commit")
            .metadata
            .tags,
        vec!["manual"]
    );

    archive
        .update_slot_metadata(
            "manual-renamed",
            RuntimeSessionMetadata::default()
                .with_display_name("Renamed Manual")
                .with_tag(" manual ")
                .with_tag("reviewed")
                .with_updated_at_unix_millis(70),
        )
        .expect("metadata commit should replace slot metadata");
    assert_eq!(
        archive
            .slot("manual-renamed")
            .expect("renamed slot should remain after metadata commit")
            .metadata
            .display_name
            .as_deref(),
        Some("Renamed Manual")
    );

    let touch_preview = archive
        .preview_touch_slot("manual-renamed", 90)
        .expect("touch preview should update only reported metadata");
    assert_eq!(touch_preview.metadata.updated_at_unix_millis, Some(90));
    assert_eq!(
        archive
            .slot("manual-renamed")
            .expect("renamed slot should remain before touch commit")
            .metadata
            .updated_at_unix_millis,
        Some(70)
    );

    archive
        .touch_slot("manual-renamed", 90)
        .expect("touch commit should update timestamp");
    assert_eq!(
        archive
            .slot("manual-renamed")
            .expect("renamed slot should remain after touch commit")
            .metadata
            .updated_at_unix_millis,
        Some(90)
    );

    let remove_preview = archive
        .preview_remove_slot("autosave")
        .expect("remove preview should report slot before commit");
    assert_eq!(remove_preview.source_slot_id, "autosave");
    assert!(archive.contains_slot("autosave"));

    let removed = archive
        .remove_slot("autosave")
        .expect("remove commit should return removed slot");
    assert_eq!(removed.slot_id, "autosave");
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["manual-renamed"]
    );
}

#[test]
fn runtime_session_archive_selected_mutations_resolve_targets_before_committing() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-new", "manual", 50),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate selected mutation fixture slots");

    let rename_preview = archive
        .preview_rename_selected_slot(
            RuntimeSessionSlotSelector::latest_updated_with_tag(" manual "),
            " manual-selected ",
        )
        .expect("selected rename preview should resolve latest manual slot");
    assert_eq!(rename_preview.source_slot_id, "manual-new");
    assert_eq!(
        rename_preview.destination_slot_id.as_deref(),
        Some("manual-selected")
    );
    assert!(archive.contains_slot("manual-new"));
    assert!(!archive.contains_slot("manual-selected"));

    archive
        .rename_selected_slot(
            RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
            "manual-selected",
        )
        .expect("selected rename commit should mutate resolved slot");
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-old", "manual-selected"]
    );

    let metadata_preview = archive
        .preview_update_selected_slot_metadata(
            RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
            RuntimeSessionMetadata::default()
                .with_display_name("Archived Manual")
                .with_tag(" archived ")
                .with_updated_at_unix_millis(70),
        )
        .expect("selected metadata preview should resolve oldest manual slot");
    assert_eq!(metadata_preview.source_slot_id, "manual-old");
    assert_eq!(metadata_preview.metadata.tags, vec!["archived"]);
    assert_eq!(
        archive
            .slot("manual-old")
            .expect("oldest manual slot should remain before metadata commit")
            .metadata
            .tags,
        vec!["manual"]
    );

    archive
        .update_selected_slot_metadata(
            RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
            RuntimeSessionMetadata::default()
                .with_display_name("Archived Manual")
                .with_tag(" archived ")
                .with_updated_at_unix_millis(70),
        )
        .expect("selected metadata commit should mutate resolved oldest slot");
    assert_eq!(
        archive
            .slot("manual-old")
            .expect("oldest manual slot should remain after metadata commit")
            .metadata
            .tags,
        vec!["archived"]
    );

    let touch_preview = archive
        .preview_touch_selected_slot(
            RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
            95,
        )
        .expect("selected touch preview should resolve renamed latest manual slot");
    assert_eq!(touch_preview.source_slot_id, "manual-selected");
    assert_eq!(touch_preview.metadata.updated_at_unix_millis, Some(95));
    assert_eq!(
        archive
            .slot("manual-selected")
            .expect("renamed selected slot should remain before touch commit")
            .metadata
            .updated_at_unix_millis,
        Some(50)
    );

    archive
        .touch_selected_slot(
            RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
            95,
        )
        .expect("selected touch commit should mutate resolved latest slot");
    assert_eq!(
        archive
            .slot("manual-selected")
            .expect("renamed selected slot should remain after touch commit")
            .metadata
            .updated_at_unix_millis,
        Some(95)
    );

    let remove_preview = archive
        .preview_remove_selected_slot(RuntimeSessionSlotSelector::latest_updated_with_tag(
            "autosave",
        ))
        .expect("selected remove preview should resolve autosave slot");
    assert_eq!(remove_preview.source_slot_id, "autosave");
    assert!(archive.contains_slot("autosave"));

    let removed = archive
        .remove_selected_slot(RuntimeSessionSlotSelector::latest_updated_with_tag(
            "autosave",
        ))
        .expect("selected remove commit should return resolved slot");
    assert_eq!(removed.slot_id, "autosave");
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["manual-old", "manual-selected"]
    );
}

#[test]
fn runtime_session_archive_selected_path_mutations_preview_and_commit_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-new", "manual", 50),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate selected path mutation fixture slots");
    let root = unique_temp_root("runtime_session_selected_path_mutation");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before selected path mutation");
    let original_payload = fs::read_to_string(&path).expect("archive payload should be readable");

    let rename_preview = RuntimeSessionArchive::preview_rename_selected_slot_from_path(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
        " manual-selected ",
    )
    .expect("selected path rename preview should resolve latest manual slot");
    assert_eq!(rename_preview.source_slot_id, "manual-new");
    assert_eq!(
        rename_preview.destination_slot_id.as_deref(),
        Some("manual-selected")
    );
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain after rename preview"),
        original_payload
    );

    let manifest = RuntimeSessionArchive::rename_selected_slot_at_path_atomically(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
        "manual-selected",
    )
    .expect("selected path rename commit should update archive atomically");
    assert_eq!(
        manifest.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-old", "manual-selected"]
    );

    let after_rename_payload =
        fs::read_to_string(&path).expect("renamed archive payload should be readable");
    let duplicate = RuntimeSessionArchive::rename_selected_slot_at_path_atomically(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
        "autosave",
    );
    assert!(matches!(
        duplicate,
        Err(RuntimeSessionArchiveError::DuplicateSlotId { slot_id }) if slot_id == "autosave"
    ));
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain after rejected rename"),
        after_rename_payload
    );

    let metadata_preview = RuntimeSessionArchive::preview_update_selected_slot_metadata_from_path(
        &path,
        RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
        RuntimeSessionMetadata::default()
            .with_display_name("Archived Path Manual")
            .with_tag(" archived ")
            .with_updated_at_unix_millis(70),
    )
    .expect("selected path metadata preview should resolve oldest manual slot");
    assert_eq!(metadata_preview.source_slot_id, "manual-old");
    assert_eq!(metadata_preview.metadata.tags, vec!["archived"]);
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain after metadata preview"),
        after_rename_payload
    );

    RuntimeSessionArchive::update_selected_slot_metadata_at_path_atomically(
        &path,
        RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
        RuntimeSessionMetadata::default()
            .with_display_name("Archived Path Manual")
            .with_tag(" archived ")
            .with_updated_at_unix_millis(70),
    )
    .expect("selected path metadata commit should update archive atomically");
    let loaded_after_metadata =
        RuntimeSessionArchive::load_from_path(&path).expect("archive should reload after metadata");
    assert_eq!(
        loaded_after_metadata
            .slot("manual-old")
            .expect("manual-old slot should remain after metadata commit")
            .metadata
            .tags,
        vec!["archived"]
    );

    let after_metadata_payload =
        fs::read_to_string(&path).expect("metadata-updated payload should be readable");
    let touch_preview = RuntimeSessionArchive::preview_touch_selected_slot_from_path(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
        95,
    )
    .expect("selected path touch preview should resolve renamed manual slot");
    assert_eq!(touch_preview.source_slot_id, "manual-selected");
    assert_eq!(touch_preview.metadata.updated_at_unix_millis, Some(95));
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain after touch preview"),
        after_metadata_payload
    );

    RuntimeSessionArchive::touch_selected_slot_at_path_atomically(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
        95,
    )
    .expect("selected path touch commit should update archive atomically");

    let after_touch_payload =
        fs::read_to_string(&path).expect("touched payload should be readable");
    let remove_preview = RuntimeSessionArchive::preview_remove_selected_slot_from_path(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("autosave"),
    )
    .expect("selected path remove preview should resolve autosave slot");
    assert_eq!(remove_preview.source_slot_id, "autosave");
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain after remove preview"),
        after_touch_payload
    );

    let manifest = RuntimeSessionArchive::remove_selected_slot_at_path_atomically(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("autosave"),
    )
    .expect("selected path remove commit should update archive atomically");
    assert_eq!(
        manifest.slot_ids().collect::<Vec<_>>(),
        vec!["manual-old", "manual-selected"]
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("archive should reload after selected path mutations")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["manual-old", "manual-selected"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}
