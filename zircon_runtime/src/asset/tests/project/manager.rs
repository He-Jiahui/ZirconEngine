use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::asset::project::AssetMetaDocument;
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    sample_animation_sequence_asset, sample_physics_material_asset, sample_sound_asset,
    write_checker_png, write_default_animation_clip, write_default_animation_graph,
    write_default_animation_sequence, write_default_animation_skeleton,
    write_default_animation_state_machine, write_default_material, write_default_physics_material,
    write_default_scene, write_test_wav, write_triangle_obj, write_valid_wgsl,
};
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetKind, AssetUri, AssetUuid, DataAsset, DataAssetFormat, FunctionAssetImporter,
    ImportedAsset, ImportedAssetEntry,
};
use crate::core::resource::ResourceState;
use crate::ui::template::UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION;

static COUNTED_IMPORT_CALLS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn project_manager_scans_assets_imports_library_and_loads_artifacts() {
    let root = unique_temp_project_root("project_manager");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(paths.assets_root().join("materials").join("grid.zmaterial"));
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let mut manager = project_manager_with_first_wave_plugin_fixtures(&root);
    let imported = manager.scan_and_import().unwrap();

    assert_eq!(manager.manifest().name, "Sandbox");
    assert!(imported.len() >= 5);
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://models/triangle.obj").unwrap())
        .is_some());
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://materials/grid.zmaterial").unwrap())
        .is_some());

    let model = manager
        .load_artifact(&AssetUri::parse("res://models/triangle.obj").unwrap())
        .unwrap();
    match model {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    assert!(paths.library_root().join("models").is_dir());
    assert!(paths.library_root().join("materials").is_dir());

    let model_meta_path = paths
        .assets_root()
        .join("models")
        .join("triangle.obj.zmeta");
    assert!(
        model_meta_path.exists(),
        "expected sidecar meta file to be generated"
    );
    let model_meta = AssetMetaDocument::load(&model_meta_path).unwrap();
    let model_record = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://models/triangle.obj").unwrap())
        .unwrap();
    assert_eq!(
        model_meta.url,
        AssetUri::parse("res://models/triangle.obj").unwrap()
    );
    assert_eq!(model_meta.asset_kind, AssetKind::Model);
    assert_eq!(model_record.id(), AssetId::from_asset_uuid(model_meta.uuid));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_imports_physics_and_animation_assets_into_runtime_library() {
    let root = unique_temp_project_root("project_manager_physics_animation");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));
    write_default_physics_material(
        paths
            .assets_root()
            .join("physics")
            .join("materials")
            .join("default.physics_material.toml"),
    );
    write_default_animation_skeleton(
        paths
            .assets_root()
            .join("animation")
            .join("hero.skeleton.zranim"),
    );
    write_default_animation_clip(
        paths
            .assets_root()
            .join("animation")
            .join("hero.clip.zranim"),
    );
    write_default_animation_sequence(
        paths
            .assets_root()
            .join("animation")
            .join("hero.sequence.zranim"),
    );
    write_default_animation_graph(
        paths
            .assets_root()
            .join("animation")
            .join("hero.graph.zranim"),
    );
    write_default_animation_state_machine(
        paths
            .assets_root()
            .join("animation")
            .join("hero.state_machine.zranim"),
    );

    let mut manager = ProjectManager::open(&root).unwrap();
    let imported = manager.scan_and_import().unwrap();

    assert!(imported.len() >= 6);
    assert!(manager
        .registry()
        .get_by_locator(
            &AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap()
        )
        .is_some());
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://animation/hero.sequence.zranim").unwrap())
        .is_some());
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://animation/hero.graph.zranim").unwrap())
        .is_some());
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://animation/hero.state_machine.zranim").unwrap())
        .is_some());

    let physics_material = manager
        .load_artifact(
            &AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap(),
        )
        .unwrap();
    let sequence = manager
        .load_artifact(&AssetUri::parse("res://animation/hero.sequence.zranim").unwrap())
        .unwrap();

    assert_eq!(
        physics_material,
        ImportedAsset::PhysicsMaterial(sample_physics_material_asset())
    );
    assert_eq!(
        sequence,
        ImportedAsset::AnimationSequence(sample_animation_sequence_asset())
    );

    assert!(paths
        .library_root()
        .join("physics")
        .join("materials")
        .is_dir());
    assert!(paths
        .library_root()
        .join("animation")
        .join("skeletons")
        .is_dir());
    assert!(paths
        .library_root()
        .join("animation")
        .join("clips")
        .is_dir());
    assert!(paths
        .library_root()
        .join("animation")
        .join("sequences")
        .is_dir());
    assert!(paths
        .library_root()
        .join("animation")
        .join("graphs")
        .is_dir());
    assert!(paths
        .library_root()
        .join("animation")
        .join("state_machines")
        .is_dir());

    let physics_meta = AssetMetaDocument::load(
        paths
            .assets_root()
            .join("physics")
            .join("materials")
            .join("default.physics_material.toml.zmeta"),
    )
    .unwrap();
    assert_eq!(
        physics_meta.url,
        AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_imports_sound_assets_into_runtime_library() {
    let root = unique_temp_project_root("project_manager_sound");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://audio/ping.wav").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_test_wav(paths.assets_root().join("audio").join("ping.wav"));

    let mut manager = project_manager_with_first_wave_plugin_fixtures(&root);
    let imported = manager.scan_and_import().unwrap();

    assert_eq!(imported.len(), 1);
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://audio/ping.wav").unwrap())
        .is_some());

    let sound = manager
        .load_artifact(&AssetUri::parse("res://audio/ping.wav").unwrap())
        .unwrap();

    assert_eq!(
        sound,
        ImportedAsset::Sound(sample_sound_asset("res://audio/ping.wav"))
    );
    assert!(paths.library_root().join("sound").is_dir());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_restores_ready_artifacts_from_meta_after_restart() {
    let root = unique_temp_project_root("project_manager_restart_restore");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://data/settings.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let data_path = paths.assets_root().join("data").join("settings.counted");
    if let Some(parent) = data_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&data_path, r#"{ "answer": 42 }"#).unwrap();

    COUNTED_IMPORT_CALLS.store(0, Ordering::SeqCst);
    let uri = AssetUri::parse("res://data/settings.counted").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    assert_eq!(COUNTED_IMPORT_CALLS.load(Ordering::SeqCst), 1);

    let record = manager.registry().get_by_locator(&uri).unwrap();
    let artifact_locator = record.artifact_locator().cloned().unwrap();
    let meta = AssetMetaDocument::load(
        paths
            .assets_root()
            .join("data")
            .join("settings.counted.zmeta"),
    )
    .unwrap();
    assert_eq!(
        meta.preview_state,
        crate::asset::project::PreviewState::Ready
    );
    assert_eq!(meta.artifact_locator.as_ref(), Some(&artifact_locator));
    assert_eq!(meta.importer_id, "test.counted.data");
    assert!(!meta.config_hash.is_empty());

    let mut restarted = ProjectManager::open(&root).unwrap();
    restarted.scan_and_import().unwrap();
    assert_eq!(
        COUNTED_IMPORT_CALLS.load(Ordering::SeqCst),
        1,
        "restart scan should restore the ready artifact without the custom importer"
    );

    let recovered = restarted.registry().get_by_locator(&uri).unwrap();
    assert_eq!(recovered.state, ResourceState::Ready);
    assert_eq!(recovered.importer_id, "test.counted.data");
    assert_eq!(recovered.artifact_locator(), Some(&artifact_locator));

    let imported = restarted.load_artifact(&uri).unwrap();
    match imported {
        ImportedAsset::Data(asset) => assert!(asset.text.contains("\"answer\"")),
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_records_failed_imports_and_continues_scanning() {
    let root = unique_temp_project_root("project_manager_failed_import");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://shaders/pbr.wgsl").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    fs::create_dir_all(paths.assets_root().join("models")).unwrap();
    fs::write(
        paths
            .assets_root()
            .join("models")
            .join("missing_backend.fbx"),
        b"fbx",
    )
    .unwrap();

    let mut manager = project_manager_with_first_wave_plugin_fixtures(&root);
    let imported = manager.scan_and_import().unwrap();

    assert_eq!(imported.len(), 2);
    let shader = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://shaders/pbr.wgsl").unwrap())
        .expect("valid shader should still import after another file fails");
    assert_eq!(shader.state, ResourceState::Ready);
    assert!(shader.artifact_locator().is_some());

    let failed = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://models/missing_backend.fbx").unwrap())
        .expect("failed import should still have a registry record");
    assert_eq!(failed.kind, crate::asset::AssetKind::Model);
    assert_eq!(failed.state, ResourceState::Error);
    assert!(failed.artifact_locator().is_none());
    assert!(failed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("backend is not installed")));

    let failed_meta = AssetMetaDocument::load(
        paths
            .assets_root()
            .join("models")
            .join("missing_backend.fbx.zmeta"),
    )
    .unwrap();
    assert_eq!(failed_meta.importer_id, "zircon.optional.model.fbx");
    assert_eq!(
        failed_meta.preview_state,
        crate::asset::project::PreviewState::Error
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_records_ui_schema_migration_in_meta() {
    let root = unique_temp_project_root("project_manager_ui_migration");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://ui/legacy.ui.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let ui_path = paths.assets_root().join("ui").join("legacy.ui.toml");
    if let Some(parent) = ui_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&ui_path, version_one_ui_layout_toml()).unwrap();

    let mut manager = project_manager_with_first_wave_plugin_fixtures(&root);
    manager.scan_and_import().unwrap();

    let meta = AssetMetaDocument::load(paths.assets_root().join("ui").join("legacy.ui.toml.zmeta"))
        .unwrap();
    assert_eq!(meta.importer_id, "ui_document_importer.typed_toml");
    assert_eq!(meta.source_schema_version, Some(1));
    assert_eq!(
        meta.target_schema_version,
        Some(UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION)
    );
    assert!(meta.migration_summary.contains("SourceVersionBumped"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_clears_stale_migration_meta_for_non_migrating_importer() {
    let root = unique_temp_project_root("project_manager_clear_stale_migration");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://data/settings.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let data_path = paths.assets_root().join("data").join("settings.json");
    if let Some(parent) = data_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&data_path, r#"{ "answer": 42 }"#).unwrap();

    let uri = AssetUri::parse("res://data/settings.json").unwrap();
    let mut stale_meta = AssetMetaDocument::new(AssetUuid::new(), uri, AssetKind::Data);
    stale_meta.source_schema_version = Some(1);
    stale_meta.target_schema_version = Some(99);
    stale_meta.migration_summary = "stale migration data".to_string();
    stale_meta
        .save(paths.assets_root().join("data").join("settings.json.zmeta"))
        .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let meta =
        AssetMetaDocument::load(paths.assets_root().join("data").join("settings.json.zmeta"))
            .unwrap();
    assert_eq!(meta.importer_id, "zircon.builtin.data.json");
    assert_eq!(meta.source_schema_version, None);
    assert_eq!(meta.target_schema_version, None);
    assert!(meta.migration_summary.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_records_import_dependency_ids_and_missing_dependency_diagnostics() {
    let root = unique_temp_project_root("project_manager_dependencies");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "DependencySandbox",
        AssetUri::parse("res://materials/grid.dep").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let material_path = paths.assets_root().join("materials").join("grid.dep");
    let texture_path = paths.assets_root().join("textures").join("checker.deptex");
    fs::create_dir_all(material_path.parent().unwrap()).unwrap();
    fs::create_dir_all(texture_path.parent().unwrap()).unwrap();
    fs::write(&material_path, "material").unwrap();
    fs::write(&texture_path, "texture").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.dep.material", "test.dep", AssetKind::Material, 1)
                .with_source_extensions(["dep"]),
            import_material_with_dependencies,
        ))
        .unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.dep.texture", "test.dep", AssetKind::Texture, 1)
                .with_source_extensions(["deptex"]),
            import_texture_dependency,
        ))
        .unwrap();

    manager.scan_and_import().unwrap();

    let texture = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://textures/checker.deptex").unwrap())
        .expect("texture record");
    let material = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://materials/grid.dep").unwrap())
        .expect("material record");

    assert_eq!(material.dependency_ids, vec![texture.id()]);
    assert!(material.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unresolved asset dependency res://textures/missing.deptex")
    }));

    let mut restarted = ProjectManager::open(&root).unwrap();
    restarted.scan_and_import().unwrap();
    let restarted_material = restarted
        .registry()
        .get_by_locator(&AssetUri::parse("res://materials/grid.dep").unwrap())
        .expect("restarted material record");
    assert_eq!(restarted_material.dependency_ids, vec![texture.id()]);
    assert!(restarted_material.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unresolved asset dependency res://textures/missing.deptex")
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_imports_labeled_subassets_as_separate_artifacts() {
    let root = unique_temp_project_root("project_manager_multi_asset_labels");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "MultiAssetSandbox",
        AssetUri::parse("res://bundles/atlas.multi").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source_path = paths.assets_root().join("bundles").join("atlas.multi");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "atlas").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.multi.bundle", "test.multi", AssetKind::Data, 1)
                .with_source_extensions(["multi"])
                .with_additional_output_kinds([AssetKind::Texture]),
            import_multi_asset_bundle,
        ))
        .unwrap();

    let imported = manager.scan_and_import().unwrap();

    let root_uri = AssetUri::parse("res://bundles/atlas.multi").unwrap();
    let texture_uri = AssetUri::parse("res://bundles/atlas.multi#Texture0").unwrap();
    let root_record = manager
        .registry()
        .get_by_locator(&root_uri)
        .expect("root record");
    let texture_record = manager
        .registry()
        .get_by_locator(&texture_uri)
        .expect("labeled texture record");
    let meta = AssetMetaDocument::load(
        paths
            .assets_root()
            .join("bundles")
            .join("atlas.multi.zmeta"),
    )
    .unwrap();

    assert_eq!(imported.len(), 2);
    assert_eq!(meta.entries.len(), 2);
    let root_entry = meta
        .entries
        .iter()
        .find(|entry| entry.url == root_uri)
        .expect("root entry");
    let texture_entry = meta
        .entries
        .iter()
        .find(|entry| entry.url == texture_uri)
        .expect("texture entry");
    assert_eq!(root_entry.uuid, meta.uuid);
    assert_ne!(texture_entry.uuid, meta.uuid);
    assert_eq!(root_record.id(), AssetId::from_asset_uuid(root_entry.uuid));
    assert_eq!(
        texture_record.id(),
        AssetId::from_asset_uuid(texture_entry.uuid)
    );
    assert_ne!(
        root_record.artifact_locator(),
        texture_record.artifact_locator()
    );
    assert!(meta
        .entries
        .iter()
        .any(|entry| entry.url == texture_uri && entry.asset_kind == AssetKind::Texture));

    match manager.load_artifact(&root_uri).unwrap() {
        ImportedAsset::Data(asset) => assert_eq!(asset.text, "atlas"),
        other => panic!("unexpected root artifact: {other:?}"),
    }
    match manager.load_artifact(&texture_uri).unwrap() {
        ImportedAsset::Texture(asset) => assert_eq!(asset.rgba, vec![255, 0, 255, 255]),
        other => panic!("unexpected subasset artifact: {other:?}"),
    }

    let mut restarted = ProjectManager::open(&root).unwrap();
    restarted.scan_and_import().unwrap();
    let restored_texture = restarted
        .registry()
        .get_by_locator(&texture_uri)
        .expect("restored labeled texture record");
    assert_eq!(restored_texture.id(), texture_record.id());
    assert_eq!(
        restarted.load_artifact(&texture_uri).unwrap(),
        manager.load_artifact(&texture_uri).unwrap()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_records_duplicate_imported_asset_label_as_failed_import() {
    let root = unique_temp_project_root("project_manager_duplicate_labels");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "DuplicateLabelSandbox",
        AssetUri::parse("res://bundles/duplicate.multi").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source_path = paths.assets_root().join("bundles").join("duplicate.multi");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "duplicate").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.multi.duplicate", "test.multi", AssetKind::Data, 1)
                .with_source_extensions(["multi"])
                .with_additional_output_kinds([AssetKind::Texture]),
            import_duplicate_label_bundle,
        ))
        .unwrap();

    manager.scan_and_import().unwrap();

    let root_record = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://bundles/duplicate.multi").unwrap())
        .expect("failed root record");
    assert_eq!(root_record.state, ResourceState::Error);
    assert!(root_record.artifact_locator().is_none());
    assert!(root_record.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("duplicate asset label Texture0")
    }));
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://bundles/duplicate.multi#Texture0").unwrap())
        .is_none());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_returns_structured_error_for_unknown_label_load() {
    let root = unique_temp_project_root("project_manager_unknown_label");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "UnknownLabelSandbox",
        AssetUri::parse("res://bundles/atlas.multi").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source_path = paths.assets_root().join("bundles").join("atlas.multi");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "atlas").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.multi.bundle", "test.multi", AssetKind::Data, 1)
                .with_source_extensions(["multi"])
                .with_additional_output_kinds([AssetKind::Texture]),
            import_multi_asset_bundle,
        ))
        .unwrap();
    manager.scan_and_import().unwrap();

    let error = manager
        .load_artifact(&AssetUri::parse("res://bundles/atlas.multi#Missing").unwrap())
        .expect_err("missing label should be structured");

    match error {
        AssetImportError::MissingAssetLabel { source_uri, label } => {
            assert_eq!(
                source_uri,
                AssetUri::parse("res://bundles/atlas.multi").unwrap()
            );
            assert_eq!(label, "Missing");
        }
        other => panic!("unexpected missing-label error: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

fn version_one_ui_layout_toml() -> &'static str {
    r#"
[asset]
kind = "layout"
id = "legacy.layout"
version = 1
display_name = "Legacy Layout"

[root]
node_id = "legacy_root"
kind = "native"
type = "VerticalBox"
control_id = "LegacyRoot"
"#
}

fn counted_data_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new("test.counted.data", "test.counted", AssetKind::Data, 1)
            .with_source_extensions(["counted"]),
        import_counted_data,
    )
}

fn project_manager_with_first_wave_plugin_fixtures(root: impl AsRef<Path>) -> ProjectManager {
    let mut manager = ProjectManager::open(root).unwrap();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager
}

fn import_counted_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    COUNTED_IMPORT_CALLS.fetch_add(1, Ordering::SeqCst);
    let text = context.source_text()?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json: serde_json::json!({ "counted": true }),
        }),
    ))
}

fn import_material_with_dependencies(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Material(crate::asset::MaterialAsset {
            name: Some("Grid".to_string()),
            shader: crate::asset::AssetReference::from_locator(
                AssetUri::parse("builtin://shader/pbr.wgsl").unwrap(),
            ),
            base_color: [0.8, 0.8, 0.8, 1.0],
            base_color_texture: None,
            normal_texture: None,
            metallic: 0.0,
            roughness: 1.0,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            emissive: [0.0, 0.0, 0.0],
            emissive_texture: None,
            alpha_mode: crate::asset::AlphaMode::Opaque,
            double_sided: false,
            property_values: Default::default(),
            texture_slots: Default::default(),
            validation_diagnostics: Vec::new(),
        }),
    )
    .with_dependency(AssetUri::parse("res://textures/checker.deptex").unwrap())
    .with_dependency(AssetUri::parse("res://textures/missing.deptex").unwrap()))
}

fn import_texture_dependency(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            context.uri.clone(),
            1,
            1,
            vec![255, 255, 255, 255],
        )),
    ))
}

fn import_multi_asset_bundle(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let texture_uri = AssetUri::parse(&format!("{}#Texture0", context.uri)).unwrap();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json: serde_json::json!({ "bundle": true }),
        }),
    )
    .with_entry(ImportedAssetEntry::new(
        texture_uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            texture_uri,
            1,
            1,
            vec![255, 0, 255, 255],
        )),
    )))
}

fn import_duplicate_label_bundle(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let texture_uri = AssetUri::parse(&format!("{}#Texture0", context.uri)).unwrap();
    let duplicate_uri = texture_uri.clone();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text: "duplicate".to_string(),
            canonical_json: serde_json::json!({ "duplicate": true }),
        }),
    )
    .with_entry(ImportedAssetEntry::new(
        texture_uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            texture_uri,
            1,
            1,
            vec![255, 0, 0, 255],
        )),
    ))
    .with_entry(ImportedAssetEntry::new(
        duplicate_uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            duplicate_uri,
            1,
            1,
            vec![0, 255, 0, 255],
        )),
    )))
}
