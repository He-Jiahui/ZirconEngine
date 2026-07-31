use crate::asset::registry::AssetRegistryIndex;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetKind, AssetUuid};
use crate::foundation::persistence::atomic_file::AtomicWriteFault;

use super::{registry_root, unique_root, uri, write_asset};

#[test]
fn watch_incremental_state_equals_a_fresh_full_rebuild() {
    let project = unique_root("incremental_equivalence");
    let first = project.join("assets");
    let second = project.join("shared");
    let texture = AssetUuid::new();
    let material = AssetUuid::new();
    write_asset(
        &first,
        "textures/hero.png",
        texture,
        AssetKind::Texture,
        vec![],
    );
    let material_source = write_asset(
        &second,
        "materials/hero.zmaterial",
        material,
        AssetKind::Material,
        vec![uri("res://textures/hero.png")],
    );
    let roots = vec![first, second];
    let persisted_root = registry_root(&project);
    let mut incremental =
        AssetRegistryIndex::rebuild_from_project(&roots, &persisted_root).unwrap();

    let mut meta = crate::asset::project::AssetMetaDocument::load(
        material_source.with_file_name("hero.zmaterial.zmeta"),
    )
    .unwrap();
    meta.source_digest = "changed".to_string();
    meta.dependencies.clear();
    meta.save(material_source.with_file_name("hero.zmaterial.zmeta"))
        .unwrap();
    incremental
        .apply_watch_changes(
            &roots,
            &persisted_root,
            &[AssetChange::new(
                AssetChangeKind::Modified,
                uri("res://materials/hero.zmaterial"),
                None,
            )],
        )
        .unwrap();

    let rebuilt = AssetRegistryIndex::rebuild_from_project(&roots, &persisted_root).unwrap();
    assert_eq!(incremental.entries(), rebuilt.entries());
    assert_eq!(incremental.get_dependencies_by_uuid(material), Vec::new());
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn copied_asset_sorted_before_original_preserves_existing_owner_and_remints_copy() {
    let project = unique_root("incremental_duplicate_order");
    let assets = project.join("assets");
    let duplicate = AssetUuid::new();
    write_asset(
        &assets,
        "z-original.data",
        duplicate,
        AssetKind::Data,
        vec![],
    );
    let persisted_root = registry_root(&project);
    let mut incremental =
        AssetRegistryIndex::rebuild_from_project(std::slice::from_ref(&assets), &persisted_root)
            .unwrap();
    write_asset(&assets, "a-copy.data", duplicate, AssetKind::Data, vec![]);

    incremental
        .apply_watch_changes(
            std::slice::from_ref(&assets),
            &persisted_root,
            &[AssetChange::new(
                AssetChangeKind::Added,
                uri("res://a-copy.data"),
                None,
            )],
        )
        .unwrap();

    let rebuilt = AssetRegistryIndex::rebuild_from_project(&[assets], &persisted_root).unwrap();
    assert_eq!(incremental.entries(), rebuilt.entries());
    assert_eq!(
        incremental
            .entry_by_path(&uri("res://z-original.data"))
            .unwrap()
            .uuid(),
        duplicate
    );
    assert_ne!(
        incremental
            .entry_by_path(&uri("res://a-copy.data"))
            .unwrap()
            .uuid(),
        duplicate
    );
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn modified_original_and_added_sorted_earlier_copy_preserve_original_owner() {
    let project = unique_root("incremental_duplicate_modified_and_added");
    let assets = project.join("assets");
    let duplicate = AssetUuid::new();
    let original = write_asset(
        &assets,
        "z-original.data",
        duplicate,
        AssetKind::Data,
        vec![],
    );
    let persisted_root = registry_root(&project);
    let mut index =
        AssetRegistryIndex::rebuild_from_project(std::slice::from_ref(&assets), &persisted_root)
            .unwrap();

    let original_meta_path = original.with_file_name("z-original.data.zmeta");
    let mut original_meta =
        crate::asset::project::AssetMetaDocument::load(&original_meta_path).unwrap();
    original_meta.source_digest = "modified-original".to_string();
    original_meta.save(&original_meta_path).unwrap();
    write_asset(&assets, "a-copy.data", duplicate, AssetKind::Data, vec![]);

    index
        .apply_watch_changes(
            std::slice::from_ref(&assets),
            &persisted_root,
            &[
                AssetChange::new(
                    AssetChangeKind::Modified,
                    uri("res://z-original.data"),
                    None,
                ),
                AssetChange::new(AssetChangeKind::Added, uri("res://a-copy.data"), None),
            ],
        )
        .unwrap();

    assert_eq!(
        index
            .entry_by_path(&uri("res://z-original.data"))
            .unwrap()
            .uuid(),
        duplicate
    );
    assert_ne!(
        index
            .entry_by_path(&uri("res://a-copy.data"))
            .unwrap()
            .uuid(),
        duplicate
    );
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn removed_source_deletes_all_subassets_and_reverse_edges() {
    let project = unique_root("incremental_removed_subassets");
    let assets = project.join("assets");
    let root_uuid = AssetUuid::new();
    let subasset_uuid = AssetUuid::new();
    let bundle_source = write_asset(
        &assets,
        "bundles/atlas.multi",
        root_uuid,
        AssetKind::Data,
        vec![],
    );
    let bundle_meta_path = bundle_source.with_file_name("atlas.multi.zmeta");
    let mut bundle_meta =
        crate::asset::project::AssetMetaDocument::load(&bundle_meta_path).unwrap();
    bundle_meta.entries = vec![
        crate::asset::project::AssetMetaEntry {
            uuid: root_uuid,
            url: uri("res://bundles/atlas.multi"),
            asset_kind: AssetKind::Data,
            artifact_locator: None,
            dependencies: vec![],
            tags: Default::default(),
        },
        crate::asset::project::AssetMetaEntry {
            uuid: subasset_uuid,
            url: uri("res://bundles/atlas.multi#Sprite"),
            asset_kind: AssetKind::Texture,
            artifact_locator: None,
            dependencies: vec![],
            tags: Default::default(),
        },
    ];
    bundle_meta.save(&bundle_meta_path).unwrap();
    let material_uuid = AssetUuid::new();
    write_asset(
        &assets,
        "materials/atlas.zmaterial",
        material_uuid,
        AssetKind::Material,
        vec![uri("res://bundles/atlas.multi#Sprite")],
    );
    let persisted_root = registry_root(&project);
    let mut index =
        AssetRegistryIndex::rebuild_from_project(std::slice::from_ref(&assets), &persisted_root)
            .unwrap();
    assert_eq!(
        index.get_referencers_by_uuid(subasset_uuid),
        vec![material_uuid]
    );

    std::fs::remove_file(bundle_source).unwrap();
    std::fs::remove_file(bundle_meta_path).unwrap();
    index
        .apply_watch_changes(
            std::slice::from_ref(&assets),
            &persisted_root,
            &[AssetChange::new(
                AssetChangeKind::Removed,
                uri("res://bundles/atlas.multi"),
                None,
            )],
        )
        .unwrap();

    assert!(index
        .resolve_asset_id_by_path(&uri("res://bundles/atlas.multi"))
        .is_err());
    assert!(index
        .resolve_asset_id_by_path(&uri("res://bundles/atlas.multi#Sprite"))
        .is_err());
    assert!(index.get_referencers_by_uuid(subasset_uuid).is_empty());
    assert!(index.get_dependencies_by_uuid(material_uuid).is_empty());
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn renamed_source_releases_previous_path_and_preserves_guid() {
    let project = unique_root("incremental_renamed_source");
    let assets = project.join("assets");
    let uuid = AssetUuid::new();
    let old_source = write_asset(&assets, "data/old.data", uuid, AssetKind::Data, vec![]);
    let persisted_root = registry_root(&project);
    let mut index =
        AssetRegistryIndex::rebuild_from_project(std::slice::from_ref(&assets), &persisted_root)
            .unwrap();
    let old_meta = old_source.with_file_name("old.data.zmeta");
    let new_source = old_source.with_file_name("new.data");
    let new_meta = old_source.with_file_name("new.data.zmeta");
    std::fs::rename(&old_source, &new_source).unwrap();
    std::fs::rename(&old_meta, &new_meta).unwrap();
    let mut meta = crate::asset::project::AssetMetaDocument::load(&new_meta).unwrap();
    meta.url = uri("res://data/new.data");
    meta.save(&new_meta).unwrap();

    index
        .apply_watch_changes(
            &[assets],
            &persisted_root,
            &[AssetChange::new(
                AssetChangeKind::Renamed,
                uri("res://data/new.data"),
                Some(uri("res://data/old.data")),
            )],
        )
        .unwrap();

    assert!(index
        .resolve_asset_id_by_path(&uri("res://data/old.data"))
        .is_err());
    assert_eq!(
        index.resolve_asset_id_by_path(&uri("res://data/new.data")),
        Ok(crate::asset::AssetId::from_asset_uuid(uuid))
    );
    assert_eq!(
        index.entry_by_uuid(uuid).unwrap().path(),
        &uri("res://data/new.data")
    );
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn persistence_failure_keeps_live_index_unchanged() {
    let project = unique_root("incremental_candidate_rollback");
    let assets = project.join("assets");
    let uuid = AssetUuid::new();
    let source = write_asset(&assets, "data/state.data", uuid, AssetKind::Data, vec![]);
    let persisted_root = registry_root(&project);
    let mut index =
        AssetRegistryIndex::rebuild_from_project(std::slice::from_ref(&assets), &persisted_root)
            .unwrap();
    let before = index.clone();
    let meta_path = source.with_file_name("state.data.zmeta");
    let mut meta = crate::asset::project::AssetMetaDocument::load(&meta_path).unwrap();
    meta.source_digest = "changed-after-watch".to_string();
    meta.save(&meta_path).unwrap();

    index
        .apply_watch_changes_with_atomic_fault(
            &[assets],
            &persisted_root,
            &[AssetChange::new(
                AssetChangeKind::Modified,
                uri("res://data/state.data"),
                None,
            )],
            AtomicWriteFault::Replace,
        )
        .unwrap_err();

    assert_eq!(index, before);
    assert_eq!(
        index.entry_by_uuid(uuid).unwrap().source_digest(),
        "digest-data/state.data"
    );
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn targeted_source_replacement_refreshes_only_the_path_referencer_closure() {
    let project = unique_root("targeted_source_replacement");
    let assets = project.join("assets");
    let old_texture = AssetUuid::new();
    let material = AssetUuid::new();
    let unrelated = AssetUuid::new();
    let texture_source = write_asset(
        &assets,
        "textures/hero.png",
        old_texture,
        AssetKind::Texture,
        vec![],
    );
    write_asset(
        &assets,
        "materials/hero.zmaterial",
        material,
        AssetKind::Material,
        vec![uri("res://textures/hero.png")],
    );
    write_asset(
        &assets,
        "data/unrelated.data",
        unrelated,
        AssetKind::Data,
        vec![],
    );
    let persisted_root = registry_root(&project);
    let active =
        AssetRegistryIndex::rebuild_from_project(std::slice::from_ref(&assets), &persisted_root)
            .unwrap();

    let mut replacement = crate::asset::project::AssetMetaDocument::load(
        texture_source.with_file_name("hero.png.zmeta"),
    )
    .unwrap();
    let replacement_texture = AssetUuid::new();
    replacement.uuid = replacement_texture;
    replacement.source_digest = "targeted-replacement".to_string();
    let candidate = active.prepare_source_replacement(&mut replacement).unwrap();

    assert_eq!(
        active
            .entry_by_path(&uri("res://textures/hero.png"))
            .unwrap()
            .uuid(),
        old_texture
    );
    assert_eq!(
        candidate
            .entry_by_path(&uri("res://textures/hero.png"))
            .unwrap()
            .uuid(),
        replacement_texture
    );
    assert_eq!(
        candidate.get_dependencies_by_uuid(material),
        vec![replacement_texture]
    );
    assert_eq!(
        candidate.get_referencers_by_uuid(replacement_texture),
        vec![material]
    );
    assert!(candidate.get_referencers_by_uuid(old_texture).is_empty());
    assert_eq!(
        candidate.entry_by_uuid(unrelated),
        active.entry_by_uuid(unrelated)
    );
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn targeted_source_replacement_remints_duplicate_guid_before_candidate_mutation() {
    let project = unique_root("targeted_source_duplicate_guid");
    let assets = project.join("assets");
    let target = AssetUuid::new();
    let owner = AssetUuid::new();
    let target_source = write_asset(&assets, "data/target.data", target, AssetKind::Data, vec![]);
    write_asset(&assets, "data/owner.data", owner, AssetKind::Data, vec![]);
    let persisted_root = registry_root(&project);
    let active =
        AssetRegistryIndex::rebuild_from_project(std::slice::from_ref(&assets), &persisted_root)
            .unwrap();
    let before = active.clone();
    let mut replacement = crate::asset::project::AssetMetaDocument::load(
        target_source.with_file_name("target.data.zmeta"),
    )
    .unwrap();
    replacement.uuid = owner;

    let candidate = active.prepare_source_replacement(&mut replacement).unwrap();

    assert_ne!(replacement.uuid, owner);
    assert_eq!(candidate.entry_by_uuid(owner), active.entry_by_uuid(owner));
    assert_eq!(
        candidate
            .entry_by_path(&uri("res://data/target.data"))
            .unwrap()
            .uuid(),
        replacement.uuid
    );
    assert!(candidate.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        crate::asset::registry::AssetRegistryDiagnostic::DuplicateGuidReminted {
            original,
            path,
            replacement: reminted,
            ..
        } if *original == owner
            && path == &uri("res://data/target.data")
            && *reminted == replacement.uuid
    )));
    assert_eq!(active, before);
    std::fs::remove_dir_all(project).unwrap();
}
