use std::fs;

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    AssetManager, AssetUri, DataAsset, DataAssetFormat, FunctionAssetImporter, ImportedAsset,
};
use zircon_runtime_interface::project::RelPath;

use super::ProjectAssetManager;

#[test]
fn pipeline_relocation_commits_live_resource_rename_and_change_event() {
    let root = unique_temp_project_root("pipeline_source_relocation");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "PipelineSourceRelocation",
        AssetUri::parse("res://data/original.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source_path = paths
        .asset_root(&RelPath::project_assets())
        .join("data/original.counted");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "pipeline-relocation-v1").unwrap();
    let source = AssetUri::parse("res://data/original.counted").unwrap();
    let target = AssetUri::parse("res://moved/renamed.counted").unwrap();

    let manager = ProjectAssetManager::default();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager
        .open_prepared_project(ProjectManager::open(&root).unwrap())
        .unwrap();
    let source_uuid = manager
        .current_project_manager()
        .unwrap()
        .asset_registry()
        .entry_by_path(&source)
        .unwrap()
        .uuid();
    let source_id = manager.resolve_asset_id(&source).unwrap();
    let changes = AssetManager::subscribe_asset_changes(&manager);

    let statuses = manager
        .relocate_project_source(source_uuid, target.clone())
        .expect("pipeline relocation should commit the durable generation");

    assert!(statuses
        .iter()
        .any(|status| status.uri == target.to_string()));
    assert_eq!(manager.resolve_asset_id(&target), Some(source_id));
    assert_eq!(manager.resolve_asset_id(&source), None);
    let change = changes
        .try_recv()
        .expect("pipeline relocation publishes a renamed asset change");
    assert_eq!(change.kind, crate::asset::watch::AssetChangeKind::Renamed);
    assert_eq!(change.uri, target);
    assert_eq!(change.previous_uri, Some(source));

    let _ = fs::remove_dir_all(root);
}

fn counted_data_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new(
            "test.pipeline.counted.data",
            "test.pipeline.counted",
            AssetKind::Data,
            1,
        )
        .with_source_extensions(["counted"]),
        import_counted_data,
    )
}

fn import_counted_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json: serde_json::json!({ "pipeline": true }),
        }),
    ))
}

fn unique_temp_project_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "zircon_{label}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn relocating_a_prefab_keeps_the_open_scene_savable() {
    use crate::asset::assets::ProjectDocumentError;
    use crate::asset::watch::AssetChangeKind;
    use crate::asset::{AssetReference, ReferenceResolutionError, SceneAsset};
    use crate::core::math::{Transform, Vec3};
    use crate::scene::world::{SceneProjectError, World};

    let root = unique_temp_project_root("prefab_relocation_save");
    eprintln!("R34 fixture root={}", root.display());
    let paths = ProjectPaths::from_root(&root).unwrap();
    let asset_root = RelPath::project_assets();
    paths.ensure_layout(&[asset_root.clone()]).unwrap();
    let assets = paths.asset_root(&asset_root);
    for folder in ["prefabs", "archive", "scenes"] {
        fs::create_dir_all(assets.join(folder)).unwrap();
    }
    let leaf = AssetUri::parse("res://prefabs/leaf.prefab.toml").unwrap();
    let old = AssetUri::parse("res://prefabs/squad.prefab.toml").unwrap();
    let moved = AssetUri::parse("res://archive/squad.prefab.toml").unwrap();
    let scene_uri = AssetUri::parse("res://scenes/prefab-move.scene.toml").unwrap();
    let old_hint = "assets/prefabs/squad.prefab.toml";
    let new_hint = "assets/archive/squad.prefab.toml";
    let scene_path = assets.join(scene_uri.path());
    ProjectManifest::new("PrefabRelocationSave", scene_uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    fs::write(
        &scene_path,
        prefab_move_node("entities", 90, "SceneSaveProbe", None, 0.0),
    )
    .unwrap();
    let keep_fixture = std::env::var_os("ZIRCON_R34_KEEP_FIXTURE").is_some();

    // The real built-in importer owns the source and sidecar; obtain each GUID before writing its
    // referencer. The source-only no-move control runs before starting an active manager/watcher.
    let leaf_source = format!(
        "uri = \"{leaf}\"\nname = \"Leaf\"\n{}",
        prefab_move_node("scene.entities", 11, "LeafRoot", None, 0.0)
    );
    fs::write(assets.join(leaf.path()), leaf_source.as_bytes()).unwrap();
    let mut seed = ProjectManager::open(&root).unwrap();
    seed.scan_and_import().unwrap();
    let leaf_entry = seed.asset_registry().entry_by_path(&leaf).unwrap();
    assert_eq!(leaf_entry.type_marker(), AssetKind::Prefab);
    let leaf_uuid = leaf_entry.uuid();
    let nested_reference = format!("{{ uuid = \"{leaf_uuid}\", url = \"{leaf}\" }}");
    let mut squad_source = format!(
        "uri = \"{old}\"\nname = \"Squad\"\n{}",
        prefab_move_node("scene.entities", 100, "Root", None, 0.0)
    );
    for (id, name, local_x, value) in [
        (101, "LeftSlot", -1.0, "NestedLeft"),
        (102, "RightSlot", 1.0, "NestedRight"),
    ] {
        squad_source.push_str(&prefab_move_node(
            "scene.entities",
            id,
            name,
            Some(100),
            0.0,
        ));
        squad_source.push_str(&prefab_move_instance(
            "scene.entities",
            &nested_reference,
            local_x,
            "LeafRoot",
            Some(value),
        ));
    }
    let old_source = assets.join(old.path());
    let new_source = assets.join(moved.path());
    fs::write(&old_source, squad_source.as_bytes()).unwrap();
    seed.scan_and_import().unwrap();
    let entry = seed.asset_registry().entry_by_path(&old).unwrap();
    assert_eq!(entry.type_marker(), AssetKind::Prefab);
    let squad_uuid = entry.uuid();
    let old_reference = AssetReference::new(squad_uuid, old.clone());
    let new_reference = AssetReference::new(squad_uuid, moved.clone());
    let persisted = seed.persist_runtime_reference(&old_reference).unwrap();
    assert_eq!(
        persisted.project_ref().unwrap().path_hint().as_str(),
        old_hint
    );
    let scene_reference =
        format!("{{ kind = \"project\", guid = \"{squad_uuid}\", path_hint = \"{old_hint}\" }}");
    let mut scene_source = String::new();
    for (id, x, local_x, value) in [
        (42, -4.0, -2.0, Some("res://prefabs/squad.prefab.toml")),
        (43, 4.0, 2.0, Some("SceneRight")),
        (44, 7.0, 3.0, None),
    ] {
        scene_source.push_str(&prefab_move_node("entities", id, "SquadInstance", None, x));
        scene_source.push_str(&prefab_move_instance(
            "entities",
            &scene_reference,
            local_x,
            "Root",
            value,
        ));
    }
    scene_source.push_str(&prefab_move_node(
        "entities",
        90,
        "SceneSaveProbe",
        None,
        0.0,
    ));
    fs::write(&scene_path, scene_source.as_bytes()).unwrap();
    if keep_fixture {
        fs::write(root.join("r34-initial-scene.toml"), scene_source.as_bytes()).unwrap();
    }

    let (mut control, original_revision) = World::load_scene_document(&seed, &scene_uri)
        .expect("same physical Scene opens before any move")
        .into_parts();
    assert!(control
        .update_transform(90, Transform::from_translation(Vec3::new(1.25, 0.0, 0.0)))
        .unwrap());
    let control_receipt = control
        .save_scene_to_project(&seed, &scene_uri, &original_revision)
        .expect("the very same source forms must save without the asset move");
    control_receipt.ensure_durable().unwrap();
    assert!(control_receipt
        .source_revision()
        .matches_path(&scene_path)
        .unwrap());
    let control_bytes = fs::read(&scene_path).unwrap();
    assert_ne!(control_bytes, scene_source.as_bytes());
    if keep_fixture {
        fs::write(root.join("r34-before-move-scene.toml"), &control_bytes).unwrap();
    }
    let disk_before = control.to_scene_asset(&seed).unwrap();
    drop(control);
    eprintln!("R34 no-move control: real Scene write durable, probe x=1.25");

    let manager = ProjectAssetManager::default();
    manager.open_prepared_project(seed).unwrap();
    let before = AssetManager::current_project_snapshot(&manager).unwrap();
    let (mut open_world, old_revision) = World::load_scene_document(&before, &scene_uri)
        .expect("activation must open the persisted control")
        .into_parts();
    assert!(old_revision.matches_path(&scene_path).unwrap());
    assert_eq!(open_world.to_scene_asset(&before).unwrap(), disk_before);
    let ImportedAsset::Prefab(squad_before) = before.load_artifact(&old).unwrap() else {
        panic!("Squad must be a real imported Prefab");
    };
    let ImportedAsset::Prefab(leaf_before) = before.load_artifact(&leaf).unwrap() else {
        panic!("Leaf must be a real imported Prefab");
    };
    assert_eq!(leaf_before.scene.entities[0].entity, 11);
    assert!(before
        .asset_registry()
        .get_dependencies_by_uuid(squad_uuid)
        .contains(&leaf_uuid));
    let pairs = |scene: &SceneAsset, entity: u64| {
        scene
            .entities
            .iter()
            .find(|row| row.entity == entity)
            .unwrap()
            .prefab_instance
            .clone()
            .unwrap()
    };
    for (id, x, value) in [(101, -1.0, "NestedLeft"), (102, 1.0, "NestedRight")] {
        let instance = pairs(&squad_before.scene, id);
        assert_eq!(
            instance.prefab,
            AssetReference::new(leaf_uuid, leaf.clone())
        );
        assert_eq!(instance.local_transform.translation[0], x);
        assert_eq!(instance.overrides.len(), 1);
        let row = &instance.overrides[0];
        assert_eq!(
            (&*row.entity_path, &*row.property_path),
            ("LeafRoot", "name")
        );
        assert_eq!(row.value, serde_json::json!(value));
    }
    for (id, x, value) in [
        (42, -2.0, Some("res://prefabs/squad.prefab.toml")),
        (43, 2.0, Some("SceneRight")),
        (44, 3.0, None),
    ] {
        let instance = pairs(&disk_before, id);
        assert_eq!(instance.prefab, old_reference);
        assert_eq!(instance.local_transform.translation[0], x);
        assert_eq!(instance.overrides.len(), usize::from(value.is_some()));
        if let Some(value) = value {
            let row = &instance.overrides[0];
            assert_eq!((&*row.entity_path, &*row.property_path), ("Root", "name"));
            assert_eq!(row.value, serde_json::json!(value));
        }
    }
    assert!(open_world
        .update_transform(90, Transform::from_translation(Vec3::new(4.25, 0.0, 0.0)))
        .unwrap());

    let changes = AssetManager::subscribe_asset_changes(&manager);
    let statuses = manager
        .relocate_project_source(squad_uuid, moved.clone())
        .unwrap();
    assert!(statuses
        .iter()
        .any(|status| status.uri == moved.to_string()));
    let renamed = std::iter::from_fn(|| changes.try_recv().ok())
        .take(32)
        .any(|change| {
            change.kind == AssetChangeKind::Renamed
                && change.uri == moved
                && change.previous_uri.as_ref() == Some(&old)
        });
    assert!(renamed, "public move must publish the exact Renamed edge");
    let current = AssetManager::current_project_snapshot(&manager).unwrap();
    assert!(current.asset_registry().entry_by_path(&old).is_none());
    assert_eq!(
        current
            .asset_registry()
            .entry_by_path(&moved)
            .unwrap()
            .uuid(),
        squad_uuid
    );
    let new_entry = current.asset_registry().entry_by_uuid(squad_uuid).unwrap();
    assert_eq!(new_entry.path(), &moved);
    assert_eq!(new_entry.type_marker(), AssetKind::Prefab);
    assert!(!old_source.try_exists().unwrap());
    assert_eq!(fs::read(&new_source).unwrap(), squad_source.as_bytes());
    assert_eq!(
        fs::read(assets.join(leaf.path())).unwrap(),
        leaf_source.as_bytes()
    );
    assert_eq!(fs::read(&scene_path).unwrap(), control_bytes);
    assert!(old_revision.matches_path(&scene_path).unwrap());
    let live_before_save = open_world.to_scene_asset(&current).unwrap();
    for id in [42, 43, 44] {
        assert_eq!(pairs(&live_before_save, id), pairs(&disk_before, id));
    }
    assert_eq!(open_world.local_transform(90).unwrap().translation.x, 4.25);
    eprintln!("R34 durable move: {squad_uuid} {old} -> {moved}; live World still has old locator and probe x=4.25");

    let save = open_world.save_scene_to_project(&current, &scene_uri, &old_revision);
    let committed = match save {
        Ok(receipt) => {
            receipt.ensure_durable().unwrap();
            assert!(receipt.source_revision().matches_path(&scene_path).unwrap());
            assert_ne!(fs::read(&scene_path).unwrap(), control_bytes);
            true
        }
        Err(SceneProjectError::ProjectDocument(ProjectDocumentError::Reference(
            ReferenceResolutionError::MissingPath { path },
        ))) if path == old.to_string() => {
            assert_eq!(fs::read(&scene_path).unwrap(), control_bytes);
            assert!(old_revision.matches_path(&scene_path).unwrap());
            false
        }
        Err(other) => panic!("R34 unexpected save failure, not behavioral RED: {other:?}"),
    };
    assert_eq!(
        open_world.to_scene_asset(&current).unwrap(),
        live_before_save,
        "saving may update the publication snapshot, not the live World"
    );
    let expected_hint = if committed { new_hint } else { old_hint };
    let disk_text = fs::read_to_string(&scene_path).unwrap();
    let saved_doc: toml::Value = toml::from_str(&disk_text).unwrap();
    for entity in saved_doc["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row.get("prefab_instance").is_some())
    {
        let reference = &entity["prefab_instance"]["prefab"];
        assert_eq!(reference["path_hint"].as_str(), Some(expected_hint));
        assert_eq!(reference["guid"].as_str().unwrap(), squad_uuid.to_string());
    }

    // Close the live asset generation; scan and load bytes on a completely independent manager.
    assert!(AssetManager::close_project(&manager).unwrap().is_some());
    drop(manager);
    let mut fresh = ProjectManager::open(&root).unwrap();
    fresh.scan_and_import().unwrap();
    let (reopened, reopened_revision) = World::load_scene_document(&fresh, &scene_uri)
        .expect("a fresh open can resolve the same GUID even while the on-disk hint is still stale")
        .into_parts();
    assert!(reopened_revision.matches_path(&scene_path).unwrap());
    let mut expected_scene = if committed {
        live_before_save
    } else {
        disk_before
    };
    for node in &mut expected_scene.entities {
        if let Some(instance) = &mut node.prefab_instance {
            instance.prefab = new_reference.clone();
        }
    }
    assert_eq!(reopened.to_scene_asset(&fresh).unwrap(), expected_scene);
    let ImportedAsset::Prefab(squad_after) = fresh.load_artifact(&moved).unwrap() else {
        panic!("reopened project must load relocated Squad as Prefab");
    };
    assert_eq!(squad_after.scene, squad_before.scene);
    assert!(fresh
        .asset_registry()
        .get_dependencies_by_uuid(squad_uuid)
        .contains(&leaf_uuid));
    let fresh_x = if committed { 4.25 } else { 1.25 };
    assert_eq!(reopened.local_transform(90).unwrap().translation.x, fresh_x);
    eprintln!("R34 independent reopen: hint={expected_hint}, resolved={moved}, probe x={fresh_x}, scene/Prefab rows unchanged");

    assert!(committed, "PREFAB_RELOCATION_SAVE_RED: MissingPath({old}); source unchanged; independent reopen repaired the locator");

    // The watcher is closed. An unrelated source can occupy the old path without being
    // indexed yet; that must revoke the stale-locator repair without losing an unsaved edit.
    fs::write(&old_source, leaf_source.as_bytes()).unwrap();
    assert!(fresh.asset_registry().entry_by_path(&old).is_none());
    assert!(open_world
        .update_transform(90, Transform::from_translation(Vec3::new(7.0, 0.0, 0.0)))
        .unwrap());
    let blocked = open_world
        .save_scene_to_project(&fresh, &scene_uri, &reopened_revision)
        .expect_err("an occupied previous source must not be remapped on save");
    assert!(
        matches!(
            &blocked,
            SceneProjectError::ProjectDocument(ProjectDocumentError::Reference(
                ReferenceResolutionError::MissingPath { path },
            )) if path == &old.to_string()
        ),
        "expected the unresolved old path, got {blocked:?}"
    );
    assert_eq!(fs::read(&scene_path).unwrap(), disk_text.as_bytes());
    assert!(reopened_revision.matches_path(&scene_path).unwrap());
    assert_eq!(open_world.local_transform(90).unwrap().translation.x, 7.0);
    eprintln!("R34 occupied old path: save rejected, scene bytes unchanged, live probe x=7.0");

    if !keep_fixture {
        fs::remove_dir_all(&root).unwrap();
    }
}

fn prefab_move_transform(x: f32) -> String {
    format!("{{ translation = [{x:.2}, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }}")
}

fn prefab_move_node(table: &str, id: u64, name: &str, parent: Option<u64>, x: f32) -> String {
    let parent = parent
        .map(|id| format!("parent = {id}\n"))
        .unwrap_or_default();
    format!(
        "\n[[{table}]]\nentity = {id}\nname = \"{name}\"\n{parent}transform = {}\n",
        prefab_move_transform(x)
    )
}

fn prefab_move_instance(
    table: &str,
    reference: &str,
    x: f32,
    path: &str,
    value: Option<&str>,
) -> String {
    let mut text = format!(
        "[{table}.prefab_instance]\nprefab = {reference}\nlocal_transform = {}\n",
        prefab_move_transform(x)
    );
    if let Some(value) = value {
        text.push_str(&format!("\n[[{table}.prefab_instance.overrides]]\nentity_path = \"{path}\"\nproperty_path = \"name\"\nvalue = \"{value}\"\n"));
    }
    text
}
