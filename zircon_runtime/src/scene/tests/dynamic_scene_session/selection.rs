use std::fs;

use crate::scene::{
    EntityId, LevelMetadata, NodeKind, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, RuntimeSessionSlot,
    RuntimeSessionSlotSelector, World,
};

use super::{tagged_slot, temporary_archive_leftovers, unique_temp_root};

#[test]
fn runtime_session_archive_selector_resolves_in_memory_slots() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-new", "manual", 50),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate selector fixture slots");

    let latest_manual = archive
        .select_slot(RuntimeSessionSlotSelector::latest_updated_with_tag(
            " manual ",
        ))
        .expect("latest manual slot should resolve");

    assert_eq!(latest_manual.selected_slot_id, "manual-new");
    assert_eq!(
        latest_manual.selector,
        RuntimeSessionSlotSelector::LatestUpdatedWithTag {
            tag: "manual".to_string()
        }
    );
    assert_eq!(latest_manual.summary.metadata.tags, vec!["manual"]);

    assert_eq!(
        archive
            .select_slot(RuntimeSessionSlotSelector::latest_updated())
            .expect("latest archive slot should resolve from the updated index")
            .selected_slot_id,
        "manual-new"
    );
    assert_eq!(
        archive
            .select_slot(RuntimeSessionSlotSelector::oldest_updated())
            .expect("oldest archive slot should resolve from the updated index")
            .selected_slot_id,
        "manual-old"
    );
    assert_eq!(
        archive
            .select_slot(RuntimeSessionSlotSelector::oldest_updated_with_tag(
                "manual"
            ))
            .expect("oldest tagged slot should resolve from the updated tag index")
            .selected_slot_id,
        "manual-old"
    );

    let selected = RuntimeSessionSlotSelector::latest_updated_with_tag("manual")
        .resolve_slot(&archive)
        .expect("selector should return a borrowed generation-bound slot handle");
    assert_eq!(selected.archive_generation(), archive.generation());
    assert_eq!(selected.archive_revision(), archive.revision());
    assert_eq!(selected.slot().slot_id, "manual-new");
    assert!(std::ptr::eq(
        selected.slot(),
        archive
            .slot("manual-new")
            .expect("selected slot should stay borrowed")
    ));

    let explicit = archive
        .select_slot(RuntimeSessionSlotSelector::slot_id("autosave"))
        .expect("explicit slot id should resolve");
    assert_eq!(explicit.selected_slot_id, "autosave");
    assert_eq!(explicit.summary.metadata.tags, vec!["autosave"]);

    let missing = archive.select_slot(RuntimeSessionSlotSelector::latest_updated_with_tag(
        "missing",
    ));
    assert!(matches!(
        missing,
        Err(RuntimeSessionArchiveError::MissingSlot { slot_id })
            if slot_id == "<latest-updated tag=\"missing\">"
    ));
}

#[test]
fn runtime_session_archive_updated_indexes_break_ties_by_slot_id() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "tie-a", "manual", 50),
        tagged_slot(&source, "tie-z", "manual", 50),
    ])
    .expect("archive should validate updated-index tie fixtures");

    assert_eq!(
        archive
            .select_slot(RuntimeSessionSlotSelector::oldest_updated())
            .expect("oldest tie should resolve from the updated index")
            .selected_slot_id,
        "tie-a"
    );
    assert_eq!(
        archive
            .select_slot(RuntimeSessionSlotSelector::latest_updated_with_tag(
                "manual"
            ))
            .expect("latest tagged tie should resolve from the updated tag index")
            .selected_slot_id,
        "tie-z"
    );
}

#[test]
fn runtime_session_archive_selected_retention_protects_resolved_slot() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-new", "manual", 50),
        tagged_slot(&source, "manual-mid", "manual", 20),
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate retention fixture slots");

    let preview_rename = archive
        .preview_rename_selected_slot(
            RuntimeSessionSlotSelector::oldest_updated_with_tag(" manual "),
            " manual-archived ",
        )
        .expect("selected rename preview should resolve oldest manual slot");
    assert_eq!(preview_rename.source_slot_id, "manual-old");
    assert_eq!(
        preview_rename.destination_slot_id.as_deref(),
        Some("manual-archived")
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-mid", "manual-new", "manual-old"]
    );

    let preview = archive
        .preview_prune_slots_with_tag_and_selected_protection(
            "manual",
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
            RuntimeSessionSlotSelector::slot_id("manual-mid"),
        )
        .expect("selected protection should preview retention through selector");
    assert_eq!(preview.removed_slot_ids, vec!["manual-old"]);
    assert_eq!(
        preview.retained_slot_ids,
        vec!["autosave", "manual-mid", "manual-new"]
    );

    let report = archive
        .prune_slots_with_tag_and_selected_protection(
            "manual",
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
            RuntimeSessionSlotSelector::slot_id("manual-mid"),
        )
        .expect("selected protection should keep resolved slot during retention");

    assert_eq!(report, preview);
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-mid", "manual-new"]
    );
}

#[test]
fn runtime_session_archive_selected_path_query_and_remove_are_atomic() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-new", "manual", 50),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate selected path fixture slots");
    let root = unique_temp_root("runtime_session_selected_path");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before selected path query");
    let original_payload = fs::read_to_string(&path).expect("archive payload should be readable");

    let selected = RuntimeSessionArchive::select_slot_from_path(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag(" manual "),
    )
    .expect("selected path query should validate archive and resolve selector");
    assert_eq!(selected.selected_slot_id, "manual-new");
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain after query"),
        original_payload
    );

    let manifest = RuntimeSessionArchive::remove_selected_slot_at_path_atomically(
        &path,
        RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
    )
    .expect("selected path remove should update archive atomically");
    assert_eq!(
        manifest.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-new"]
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("archive should reload after selected path remove")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave", "manual-new"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_selected_metadata_update_targets_resolved_slot() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-new", "manual", 50),
    ])
    .expect("archive should validate selected metadata fixture slots");

    archive
        .update_selected_slot_metadata(
            RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
            RuntimeSessionMetadata::default()
                .with_display_name("Archived Manual")
                .with_tag(" archived ")
                .with_updated_at_unix_millis(70),
        )
        .expect("selected metadata update should target oldest manual slot");

    assert_eq!(
        archive
            .slot("manual-old")
            .expect("oldest manual slot should remain")
            .metadata
            .display_name
            .as_deref(),
        Some("Archived Manual")
    );
    assert_eq!(
        archive
            .slot("manual-old")
            .expect("oldest manual slot should remain")
            .metadata
            .tags,
        vec!["archived"]
    );
    assert_eq!(
        archive
            .slot("manual-new")
            .expect("latest manual slot should remain")
            .metadata
            .tags,
        vec!["manual"]
    );

    assert_eq!(
        archive
            .select_slot(RuntimeSessionSlotSelector::latest_updated_with_tag(
                "manual"
            ))
            .expect("metadata update must refresh the manual tag index")
            .selected_slot_id,
        "manual-new"
    );
    assert_eq!(
        archive
            .select_slot(RuntimeSessionSlotSelector::latest_updated_with_tag(
                "archived"
            ))
            .expect("metadata update must publish the archived tag index")
            .selected_slot_id,
        "manual-old"
    );
}

#[test]
fn runtime_session_archive_live_slot_indexes_follow_rename_and_remove() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-new", "manual", 50),
    ])
    .expect("archive should validate live index fixture slots");

    archive
        .rename_slot("manual-new", "manual-current")
        .expect("rename should update the live slot id index");
    assert!(archive.slot("manual-new").is_none());
    assert_eq!(
        archive
            .select_slot(RuntimeSessionSlotSelector::latest_updated_with_tag(
                "manual"
            ))
            .expect("rename must preserve the tag index row")
            .selected_slot_id,
        "manual-current"
    );

    assert!(archive.remove_slot("manual-current").is_some());
    assert!(archive.slot("manual-current").is_none());
    assert_eq!(
        archive
            .select_slot(RuntimeSessionSlotSelector::latest_updated_with_tag(
                "manual"
            ))
            .expect("remove must retire the removed tag index row")
            .selected_slot_id,
        "manual-old"
    );
}

#[test]
fn runtime_session_archive_selected_transfer_helpers_use_resolved_slots() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-new", "manual", 50),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate selected transfer fixture slots");

    archive
        .copy_selected_slot(
            RuntimeSessionSlotSelector::latest_updated_with_tag(" manual "),
            " manual-copy ",
        )
        .expect("selected copy should resolve latest manual slot");
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-copy", "manual-new", "manual-old"]
    );
    assert_eq!(
        archive
            .slot("manual-copy")
            .expect("selected copy should exist")
            .metadata
            .tags,
        vec!["manual"]
    );

    let exported = archive
        .selected_single_slot_archive(RuntimeSessionSlotSelector::oldest_updated_with_tag(
            "manual",
        ))
        .expect("selected export should resolve oldest manual slot");
    assert_eq!(exported.slot_ids().collect::<Vec<_>>(), vec!["manual-old"]);

    let incoming = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "incoming-old", "import", 5),
        tagged_slot(&source, "incoming-new", "import", 60),
    ])
    .expect("incoming archive should validate selected import fixture slots");
    archive
        .import_selected_slot_from_archive_with_metadata(
            &incoming,
            RuntimeSessionSlotSelector::latest_updated_with_tag(" import "),
            " imported-copy ",
            RuntimeSessionMetadata::default()
                .with_display_name("Imported Copy")
                .with_tag(" selected-import ")
                .with_updated_at_unix_millis(90),
        )
        .expect("selected import should resolve source archive selector");

    let imported = archive
        .slot("imported-copy")
        .expect("selected import destination should exist");
    assert_eq!(
        imported.metadata.display_name.as_deref(),
        Some("Imported Copy")
    );
    assert_eq!(imported.metadata.tags, vec!["selected-import"]);
    assert_eq!(
        incoming.slot_ids().collect::<Vec<_>>(),
        vec!["incoming-new", "incoming-old"]
    );
}

#[test]
fn runtime_session_archive_selected_single_slot_export_to_path_is_atomic() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-new", "manual", 50),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate selected export fixture slots");
    let root = unique_temp_root("runtime_session_selected_export_path");
    let target_path = root.join("exports").join("manual.zrsession.json");

    let manifest = archive
        .save_selected_single_slot_archive_to_path_atomically(
            RuntimeSessionSlotSelector::latest_updated_with_tag(" manual "),
            &target_path,
        )
        .expect("selected single-slot archive should save atomically");

    assert_eq!(manifest.slot_ids().collect::<Vec<_>>(), vec!["manual-new"]);
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&target_path)
            .expect("selected export archive should reload")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["manual-new"]
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-new", "manual-old"]
    );
    assert!(temporary_archive_leftovers(
        target_path
            .parent()
            .expect("target path should have parent")
    )
    .is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_selected_restore_apply_and_diff_use_resolved_slots() {
    let (old_world, old_entity) = world_with_named_node(NodeKind::Mesh, "Old Manual Mesh");
    let (new_world, new_entity) = world_with_named_node(NodeKind::Camera, "Latest Manual Camera");
    let archive = RuntimeSessionArchive::from_slots(vec![
        named_slot("manual-old", &old_world, "Old Manual", "manual", 10),
        named_slot("manual-new", &new_world, "Latest Manual", "manual", 50),
    ])
    .expect("archive should validate selected restore fixture slots");

    let restored = archive
        .restore_selected_slot_to_empty_world(RuntimeSessionSlotSelector::latest_updated_with_tag(
            " manual ",
        ))
        .expect("selected restore should resolve latest manual slot");
    assert_eq!(restored.node_records().len(), 1);
    assert_eq!(
        restored
            .find_node(new_entity)
            .expect("latest selected entity should restore")
            .name,
        "Latest Manual Camera"
    );

    let selected_world_diff = archive
        .diff_selected_slot_with_world(
            RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
            &restored,
        )
        .expect("selected diff should compare resolved slot with restored world");
    assert_eq!(selected_world_diff.slot_id, "manual-new");
    assert!(selected_world_diff.matches);
    assert_eq!(selected_world_diff.slot_entity_count, 1);
    assert_eq!(selected_world_diff.target_entity_count, 1);

    let mut live_world = World::empty();
    let existing_entity = live_world.spawn_node(NodeKind::AmbientLight);
    assert_eq!(existing_entity, new_entity);
    let remap = archive
        .apply_selected_slot(
            RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
            &mut live_world,
        )
        .expect("selected apply should remap source entity into live world");
    let mapped_entity = remap
        .get(new_entity)
        .expect("selected apply should map source entity");
    assert_ne!(mapped_entity, new_entity);
    assert_eq!(
        live_world
            .find_node(mapped_entity)
            .expect("mapped selected entity should exist")
            .name,
        "Latest Manual Camera"
    );
    assert!(live_world.find_node(existing_entity).is_some());

    let manager = crate::scene::DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), LevelMetadata::default());
    let stale_entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::PointLight);
        world
            .rename_node(entity, "Stale Light")
            .expect("stale entity should be named");
        entity
    });

    let restore_report = archive
        .restore_selected_slot_into_level(
            RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
            &level,
        )
        .expect("selected restore should replace level with oldest manual slot");
    assert_eq!(restore_report.slot_id, "manual-old");
    assert_eq!(restore_report.entity_count, 1);
    assert_eq!(level.metadata().display_name.as_deref(), Some("Old Manual"));
    level.with_world(|world| {
        assert!(world
            .node_records()
            .iter()
            .all(|record| record.name != "Stale Light"));
        assert_eq!(
            world
                .find_node(old_entity)
                .expect("oldest selected entity should restore into level")
                .name,
            "Old Manual Mesh"
        );
    });

    let selected_level_diff = archive
        .diff_selected_slot_with_level(
            RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
            &level,
        )
        .expect("selected diff should compare resolved slot with restored level");
    assert_eq!(selected_level_diff.slot_id, "manual-old");
    assert!(selected_level_diff.matches);

    let level_remap = archive
        .apply_selected_slot_to_level(
            RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
            &level,
        )
        .expect("selected apply should append resolved slot to level");
    let mapped_level_entity = level_remap
        .get(new_entity)
        .expect("selected level apply should map source entity");
    assert_ne!(mapped_level_entity, new_entity);
    level.with_world(|world| {
        assert_eq!(world.node_records().len(), 2);
        assert_eq!(
            world
                .find_node(mapped_level_entity)
                .expect("mapped selected level entity should exist")
                .name,
            "Latest Manual Camera"
        );
    });
}

#[test]
fn runtime_session_archive_selected_path_restore_apply_and_diff_use_resolved_slots() {
    let (old_world, _old_entity) = world_with_named_node(NodeKind::Mesh, "Path Old Manual Mesh");
    let (new_world, new_entity) =
        world_with_named_node(NodeKind::Camera, "Path Latest Manual Camera");
    let archive = RuntimeSessionArchive::from_slots(vec![
        named_slot("manual-old", &old_world, "Path Old Manual", "manual", 10),
        named_slot("manual-new", &new_world, "Path Latest Manual", "manual", 50),
    ])
    .expect("archive should validate selected path restore fixture slots");
    let root = unique_temp_root("runtime_session_selected_restore_path");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before selected path restore");

    let restored = RuntimeSessionArchive::restore_selected_slot_from_path_to_empty_world(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
    )
    .expect("selected path restore should resolve latest manual slot");
    assert_eq!(
        restored
            .find_node(new_entity)
            .expect("selected path entity should restore")
            .name,
        "Path Latest Manual Camera"
    );

    let world_diff = RuntimeSessionArchive::diff_selected_slot_from_path_with_world(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
        &restored,
    )
    .expect("selected path diff should resolve latest manual slot");
    assert_eq!(world_diff.slot_id, "manual-new");
    assert!(world_diff.matches);

    let mut live_world = World::empty();
    let existing_entity = live_world.spawn_node(NodeKind::AmbientLight);
    assert_eq!(existing_entity, new_entity);
    let remap = RuntimeSessionArchive::apply_selected_slot_from_path_to_world(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
        &mut live_world,
    )
    .expect("selected path apply should resolve and remap source entity");
    let mapped_entity = remap
        .get(new_entity)
        .expect("selected path apply should map source entity");
    assert_ne!(mapped_entity, new_entity);
    assert_eq!(
        live_world
            .find_node(mapped_entity)
            .expect("mapped selected path entity should exist")
            .name,
        "Path Latest Manual Camera"
    );

    let manager = crate::scene::DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), LevelMetadata::default());
    let restore_report = RuntimeSessionArchive::restore_selected_slot_from_path_into_level(
        &path,
        RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
        &level,
    )
    .expect("selected path restore should replace level with oldest manual slot");
    assert_eq!(restore_report.slot_id, "manual-old");
    assert_eq!(
        level.metadata().display_name.as_deref(),
        Some("Path Old Manual")
    );

    let level_diff = RuntimeSessionArchive::diff_selected_slot_from_path_with_level(
        &path,
        RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
        &level,
    )
    .expect("selected path level diff should resolve oldest manual slot");
    assert_eq!(level_diff.slot_id, "manual-old");
    assert!(level_diff.matches);

    let level_remap = RuntimeSessionArchive::apply_selected_slot_from_path_to_level(
        &path,
        RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
        &level,
    )
    .expect("selected path apply should append latest manual slot to level");
    let mapped_level_entity = level_remap
        .get(new_entity)
        .expect("selected path level apply should map source entity");
    assert_ne!(mapped_level_entity, new_entity);
    level.with_world(|world| {
        assert_eq!(world.node_records().len(), 2);
        assert_eq!(
            world
                .find_node(mapped_level_entity)
                .expect("mapped selected path level entity should exist")
                .name,
            "Path Latest Manual Camera"
        );
    });

    let _ = fs::remove_dir_all(root);
}

fn world_with_named_node(kind: NodeKind, name: &str) -> (World, EntityId) {
    let mut world = World::empty();
    let entity = world.spawn_node(kind);
    world
        .rename_node(entity, name)
        .expect("fixture entity should be named");
    (world, entity)
}

fn named_slot(
    slot_id: &str,
    source: &World,
    display_name: &str,
    tag: &str,
    updated_at_unix_millis: u64,
) -> RuntimeSessionSlot {
    RuntimeSessionSlot::from_world_with_metadata(
        slot_id,
        source,
        RuntimeSessionMetadata::default()
            .with_display_name(display_name)
            .with_tag(tag)
            .with_updated_at_unix_millis(updated_at_unix_millis),
    )
    .expect("named slot should capture")
}
