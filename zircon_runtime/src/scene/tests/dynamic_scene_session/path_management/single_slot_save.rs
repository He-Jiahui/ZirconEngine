use super::*;

#[test]
fn runtime_session_archive_saves_single_slot_archive_from_path_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_single_slot_archive");
    let source_path = root.join("sessions").join("archive.zrsession.json");
    let target_path = root.join("exports").join("autosave.zrsession.json");
    archive
        .save_to_path_atomically(&source_path)
        .expect("source archive should save before single-slot archive export");
    let source_payload =
        fs::read_to_string(&source_path).expect("source archive payload should be readable");

    let manifest = RuntimeSessionArchive::save_single_slot_archive_from_path_atomically(
        &source_path,
        "autosave",
        &target_path,
    )
    .expect("one source slot should save as a standalone archive");

    assert_eq!(manifest.slot_ids().collect::<Vec<_>>(), vec!["autosave"]);
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&target_path)
            .expect("standalone slot archive should reload")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave"]
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&target_path)
            .expect("standalone slot archive should reload for metadata")
            .slot("autosave")
            .expect("autosave slot should exist")
            .metadata
            .tags,
        vec!["autosave"]
    );
    assert_eq!(
        fs::read_to_string(&source_path)
            .expect("source archive payload should remain readable after export"),
        source_payload
    );
    assert!(temporary_archive_leftovers(
        source_path
            .parent()
            .expect("source path should have parent")
    )
    .is_empty());
    assert!(temporary_archive_leftovers(
        target_path
            .parent()
            .expect("target path should have parent")
    )
    .is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_saves_single_slot_archive_from_memory_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "quicksave", "quicksave", 50),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_memory_single_slot_archive");
    let target_path = root.join("exports").join("manual.zrsession.json");

    let manifest = archive
        .save_single_slot_archive_to_path_atomically("manual", &target_path)
        .expect("one in-memory slot should save as a standalone archive");

    assert_eq!(manifest.slot_ids().collect::<Vec<_>>(), vec!["manual"]);
    let exported = RuntimeSessionArchive::load_from_path(&target_path)
        .expect("standalone in-memory slot archive should reload");
    assert_eq!(exported.slot_ids().collect::<Vec<_>>(), vec!["manual"]);
    assert_eq!(
        exported
            .slot("manual")
            .expect("manual slot should exist")
            .metadata
            .tags,
        vec!["manual"]
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["manual", "quicksave"]
    );
    assert!(temporary_archive_leftovers(
        target_path
            .parent()
            .expect("target path should have parent")
    )
    .is_empty());

    let _ = fs::remove_dir_all(root);
}
