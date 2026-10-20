use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::project::lock_project_generation;
use crate::asset::project::{ProjectManager, ProjectManifest};
use crate::asset::watch::{AssetChange, AssetChangeKind, AssetWatchBatch};
use crate::asset::{AssetManager, AssetUri};
use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};
use zircon_runtime_interface::project::RelPath;

use super::ProjectAssetManager;

fn copy_project_template(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_project_template(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn renderable_project_open_publishes_lazy_model_management_without_deadlock() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_renderable_open_{}_{}",
        std::process::id(),
        unique
    ));
    let template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../templates/projects/renderable-empty");
    copy_project_template(&template, &root);

    let (sender, receiver) = mpsc::sync_channel(1);
    let worker_root = root.clone();
    let worker = thread::spawn(move || {
        let result = (|| {
            let manager = ProjectAssetManager::default();
            let project = ProjectManager::open(&worker_root).map_err(|error| error.to_string())?;
            AssetManager::open_prepared_project(&manager, project)
                .map_err(|error| error.to_string())?;
            let model_id = manager
                .resolve_asset_id(&AssetUri::parse("res://models/cube.obj").unwrap())
                .ok_or("template model was not published")?;
            Ok::<_, String>(
                manager
                    .model_asset_management_records()
                    .iter()
                    .any(|record| record.model_id == model_id),
            )
        })();
        sender.send(result).unwrap();
    });
    let has_model = receiver
        .recv_timeout(Duration::from_secs(20))
        .expect("opening the renderable project must finish without a generation lock cycle")
        .expect("opening the renderable project must succeed");
    assert!(
        has_model,
        "the published management generation must include the lazy model"
    );
    worker.join().unwrap();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn independent_managers_serialize_project_open_and_watch_generation() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_concurrent_open_watch_{}_{}",
        std::process::id(),
        unique
    ));
    let template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../templates/projects/renderable-empty");
    copy_project_template(&template, &root);

    let editor = ProjectAssetManager::default();
    AssetManager::open_project(&editor, root.to_str().unwrap()).unwrap();
    let watch_errors = AssetManager::subscribe_asset_watch_errors(&editor);
    let generation = lock_project_generation(&root).unwrap();
    let (sender, receiver) = mpsc::sync_channel(2);

    let watch_editor = editor.clone();
    let watch_sender = sender.clone();
    let watch_worker = thread::spawn(move || {
        watch_editor.process_watch_batch_in_generation(AssetWatchBatch {
            changes: vec![AssetChange::new(
                AssetChangeKind::Modified,
                AssetUri::parse("res://models/cube.obj").unwrap(),
                None,
            )],
            ..AssetWatchBatch::default()
        });
        watch_sender.send("watch").unwrap();
    });

    let runtime_root = root.clone();
    let open_worker = thread::spawn(move || {
        let runtime = ProjectAssetManager::default();
        let result = AssetManager::open_project(&runtime, runtime_root.to_str().unwrap())
            .map_err(|error| error.to_string());
        sender.send("open").unwrap();
        result
    });

    assert!(
        receiver.recv_timeout(Duration::from_millis(100)).is_err(),
        "both generation paths must wait for the same physical project"
    );
    drop(generation);
    receiver.recv_timeout(Duration::from_secs(40)).unwrap();
    receiver.recv_timeout(Duration::from_secs(40)).unwrap();
    watch_worker.join().unwrap();
    open_worker.join().unwrap().unwrap();
    assert!(
        watch_errors.try_recv().is_err(),
        "watch generation must commit cleanly"
    );
    drop(editor);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_startup_snapshot_survives_disk_manifest_rewrite_after_activation() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_prepared_project_{}_{}",
        std::process::id(),
        unique
    ));
    fs::create_dir_all(root.join("assets")).unwrap();
    let manifest_path = root.join("zircon-project.toml");
    ProjectManifest::new(
        "Activated Snapshot One",
        AssetUri::parse("res://scenes/one.scene.toml").unwrap(),
        1,
    )
    .save(&manifest_path)
    .unwrap();
    let project = ProjectManager::open(&root).unwrap();
    let manager = ProjectAssetManager::default();
    let opened = AssetManager::open_prepared_project(&manager, project).unwrap();
    assert_eq!(opened.name, "Activated Snapshot One");

    ProjectManifest::new(
        "Activated Snapshot Two",
        AssetUri::parse("res://scenes/two.scene.toml").unwrap(),
        2,
    )
    .save(&manifest_path)
    .unwrap();

    let current = AssetManager::current_project_snapshot(&manager).unwrap();

    assert_eq!(
        current.manifest().default_scene.to_string(),
        "res://scenes/one.scene.toml"
    );
    drop(current);
    drop(manager);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn prepared_project_uses_manager_owned_io_pool_for_environment_ibl_staging() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_ibl_executor_{}_{}",
        std::process::id(),
        unique
    ));
    fs::create_dir_all(root.join("assets")).unwrap();
    ProjectManifest::new(
        "IBL Executor Injection",
        AssetUri::parse("res://scenes/ibl.scene.toml").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();

    let task_pool = TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1));
    let manager = ProjectAssetManager::new(task_pool.clone());
    let project = ProjectManager::open(&root).unwrap();
    AssetManager::open_prepared_project(&manager, project).unwrap();

    let project = AssetManager::current_project_snapshot(&manager).unwrap();
    let injected = project
        .environment_ibl_parallel_executor_for_test()
        .expect("prepared project should retain the manager runtime IO pool");
    assert!(injected.shares_execution_owner_with(&task_pool));

    drop(project);
    drop(manager);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_scan_stops_pending_watchers_without_publishing_a_generation() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_failed_scan_watchers_{}_{}",
        std::process::id(),
        unique
    ));
    let mut manifest = ProjectManifest::new(
        "Failed Scan Watchers",
        AssetUri::parse("res://data/collision.json").unwrap(),
        1,
    );
    manifest.asset_roots = vec![
        RelPath::parse("game-assets").unwrap(),
        RelPath::parse("shared-assets").unwrap(),
    ];
    manifest.save(root.join("zircon-project.toml")).unwrap();
    let first = root.join("game-assets/data/collision.json");
    let second = root.join("shared-assets/data/collision.json");
    fs::create_dir_all(first.parent().unwrap()).unwrap();
    fs::create_dir_all(second.parent().unwrap()).unwrap();
    fs::write(first, "{}").unwrap();
    fs::write(second, "{}").unwrap();

    let manager = ProjectAssetManager::default();
    let project = ProjectManager::open(&root).unwrap();

    assert!(AssetManager::open_prepared_project(&manager, project).is_err());
    assert!(manager.project_read().is_none());
    assert!(manager
        .watcher_activation
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .is_none());
    assert!(manager
        .watchers
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .is_empty());

    drop(manager);
    let _ = fs::remove_dir_all(root);
}

#[derive(Clone, Debug)]
struct ConcreteIdentityObjImporter {
    descriptor: crate::asset::AssetImporterDescriptor,
    calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl ConcreteIdentityObjImporter {
    fn new(calls: std::sync::Arc<std::sync::atomic::AtomicUsize>) -> Self {
        let mut descriptor = crate::asset::AssetImporter::default()
            .registry()
            .descriptor_for_source(Path::new("identity.obj"))
            .unwrap()
            .clone();
        descriptor.id = "test.concrete_identity.obj".into();
        descriptor.plugin_id = "test.concrete_identity".into();
        descriptor.priority += 100;
        Self { descriptor, calls }
    }
}

impl crate::asset::AssetImporterHandler for ConcreteIdentityObjImporter {
    fn descriptor(&self) -> &crate::asset::AssetImporterDescriptor {
        &self.descriptor
    }

    fn import(
        &self,
        context: &crate::asset::AssetImportContext,
    ) -> Result<crate::asset::AssetImportOutcome, crate::asset::AssetImportError> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // Run the actual OBJ importer, retaining its real model and mesh outputs.
        crate::asset::AssetImporter::default().import_context(context)
    }
}

fn concrete_identity_project(label: &str) -> (std::path::PathBuf, AssetUri) {
    let root = crate::asset::tests::project::unique_temp_project_root(label);
    fs::create_dir_all(root.join("assets/models")).unwrap();
    let uri = AssetUri::parse("res://models/identity.obj").unwrap();
    ProjectManifest::new("ConcreteIdentity", uri.clone(), 1)
        .save(root.join("zircon-project.toml"))
        .unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../templates/projects/renderable-empty/assets/models/cube.obj"),
        root.join("assets/models/identity.obj"),
    )
    .unwrap();
    (root, uri)
}

fn assert_lazy_concrete_geometry(manager: &ProjectAssetManager, uri: &AssetUri) {
    use crate::asset::{Handle, MeshAsset, ModelAsset};
    use crate::core::resource::ResourceKind;
    let handle = manager.handle::<ModelAsset>(uri).unwrap();
    assert!(manager.handle::<MeshAsset>(uri).is_err());
    assert!(manager.assets::<ModelAsset>().contains(handle));
    assert!(manager.assets::<ModelAsset>().get(handle).is_none());
    assert_eq!(manager.resource_manager().ref_count(handle.id()), Some(0));
    let project = manager.current_project_manager().unwrap();
    let meshes = project
        .registry()
        .values()
        .filter(|record| record.kind == ResourceKind::Mesh)
        .collect::<Vec<_>>();
    assert!(
        !meshes.is_empty(),
        "actual OBJ import must publish real mesh subassets"
    );
    for mesh in meshes {
        let handle = manager.handle::<MeshAsset>(&mesh.primary_locator).unwrap();
        assert_eq!(handle.id(), mesh.id);
        assert!(manager.assets::<MeshAsset>().contains(handle));
        assert!(!manager
            .assets::<ModelAsset>()
            .contains(Handle::new(mesh.id)));
        assert!(manager.assets::<MeshAsset>().get(handle).is_none());
    }
}

#[test]
fn cold_restore_keeps_concrete_declarations_without_resident_geometry() {
    let (root, uri) = concrete_identity_project("concrete_identity_cold");
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let first = ProjectAssetManager::default();
    first
        .register_asset_importer(ConcreteIdentityObjImporter::new(calls.clone()))
        .unwrap();
    AssetManager::open_project(&first, root.to_str().unwrap()).unwrap();
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_lazy_concrete_geometry(&first, &uri);
    first.close_project().unwrap();
    drop(first);
    let restored = ProjectAssetManager::default();
    restored
        .register_asset_importer(ConcreteIdentityObjImporter::new(calls.clone()))
        .unwrap();
    AssetManager::open_project(&restored, root.to_str().unwrap()).unwrap();
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "reopening must use the durable artifact variants rather than import again"
    );
    assert_lazy_concrete_geometry(&restored, &uri);
    restored.close_project().unwrap();
    drop(restored);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn targeted_geometry_filtering_keeps_concrete_declarations() {
    let (root, uri) = concrete_identity_project("concrete_identity_targeted");
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let task_pool = TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1));
    let manager = ProjectAssetManager::new(task_pool.clone());
    manager
        .register_asset_importer(ConcreteIdentityObjImporter::new(calls.clone()))
        .unwrap();
    AssetManager::open_project(&manager, root.to_str().unwrap()).unwrap();
    assert_lazy_concrete_geometry(&manager, &uri);
    let before = manager
        .resource_manager()
        .registry()
        .get_by_locator(&uri)
        .unwrap()
        .source_hash
        .clone();
    let source = root.join("assets/models/identity.obj");
    let original = fs::read_to_string(&source).unwrap();
    let modified = original.replacen("v -0.5 -0.5 0.5", "v -0.75 -0.5 0.5", 1);
    assert_ne!(modified, original);
    fs::write(&source, modified).unwrap();
    manager.process_watch_batch_in_generation(AssetWatchBatch {
        changes: vec![AssetChange::new(
            AssetChangeKind::Modified,
            uri.clone(),
            None,
        )],
        ..AssetWatchBatch::default()
    });
    assert!(calls.load(std::sync::atomic::Ordering::SeqCst) >= 2);
    assert_ne!(
        manager
            .resource_manager()
            .registry()
            .get_by_locator(&uri)
            .unwrap()
            .source_hash,
        before
    );
    assert_lazy_concrete_geometry(&manager, &uri);
    manager.close_project().unwrap();
    assert!(
        task_pool.shutdown_until(std::time::Instant::now() + Duration::from_secs(20)),
        "the owned IO pool must join all pending watch work before fixture removal"
    );
    drop(manager);
    drop(task_pool);
    fs::remove_dir_all(root).unwrap();
}
