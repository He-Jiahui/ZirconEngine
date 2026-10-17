use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use crate::asset::project::{
    lock_project_generation, ProjectGenerationAdmissionObserver, ProjectManager, ProjectManifest,
};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    write_checker_png, write_default_material, write_triangle_obj, write_valid_wgsl,
};
use crate::asset::watch::{AssetChange, AssetChangeKind, AssetWatchBatch};
use crate::asset::{AssetManager, AssetUri, AssetUuid};
use crate::core::resource::{ResourceManager, ResourceRecord};
use crate::core::CoreError;
use zircon_runtime_interface::project::RelPath;

use super::ProjectAssetManager;

struct Fixture {
    root: PathBuf,
    manager: ProjectAssetManager,
    source: AssetUri,
    model: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = unique_temp_project_root(label);
        let source = AssetUri::parse("res://data/target.json").unwrap();
        fs::create_dir_all(root.join("assets/data")).unwrap();
        ProjectManifest::new(
            "Candidate Activation Fixture",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(root.join("zircon-project.toml"))
        .unwrap();
        fs::write(root.join("assets/data/target.json"), br#"{"version":1}"#).unwrap();
        write_valid_wgsl(root.join("assets/shaders/pbr.wgsl"));
        write_checker_png(root.join("assets/textures/checker.png"));
        write_default_material(root.join("assets/materials/default.zmaterial"));
        let model = root.join("incoming/triangle.obj");
        write_triangle_obj(model.clone());

        // Persist the initial sidecars before starting PAM watchers. Every reopen below
        // scans unchanged sources, so the admission ordering has no startup watch work.
        let mut project = ProjectManager::open(&root).unwrap();
        project
            .register_first_wave_plugin_fixture_importers_for_test()
            .unwrap();
        project.scan_and_import().unwrap();
        let manager = ProjectAssetManager::default();
        manager
            .register_first_wave_plugin_fixture_importers_for_test()
            .unwrap();
        AssetManager::open_prepared_project(&manager, project).unwrap();
        Self {
            root,
            manager,
            source,
            model,
        }
    }

    fn source_uuid(&self) -> AssetUuid {
        self.manager
            .current_project_snapshot()
            .unwrap()
            .asset_registry()
            .entry_by_path(&self.source)
            .unwrap()
            .uuid()
    }

    fn operation(
        &self,
        mutation: Mutation,
    ) -> impl FnOnce() -> Result<(), String> + Send + 'static {
        let manager = self.manager.clone();
        let source = self.source.clone();
        let uuid = self.source_uuid();
        let model = self.model.clone();
        move || {
            mutation
                .run(&manager, source, uuid, &model)
                .map_err(|error| error.to_string())
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.manager.close_project();
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Clone, Copy, Debug)]
enum Mutation {
    Targeted,
    Model,
    Reimport,
    Deletion,
    Relocation,
    Generated,
}

impl Mutation {
    fn run(
        self,
        manager: &ProjectAssetManager,
        source: AssetUri,
        uuid: AssetUuid,
        model: &Path,
    ) -> Result<(), CoreError> {
        match self {
            Self::Targeted => AssetManager::import_asset(manager, &source.to_string())
                .map(|status| assert!(status.is_some_and(|status| status.imported))),
            Self::Model => AssetManager::import_model_source(manager, model)
                .map(|receipt| assert!(!receipt.committed_records().is_empty())),
            Self::Reimport => AssetManager::reimport_all(manager).map(|statuses| {
                assert!(statuses
                    .iter()
                    .any(|status| status.uri == source.to_string()))
            }),
            Self::Deletion => manager
                .delete_project_source(uuid)
                .map(|statuses| assert!(!statuses.is_empty())),
            Self::Relocation => manager
                .relocate_project_source(uuid, AssetUri::parse("res://moved/target.json").unwrap())
                .map(|statuses| assert!(!statuses.is_empty())),
            Self::Generated => manager
                .publish_generated_project_source(source, br#"{"version":2}"#.to_vec())
                .map(|receipt| assert!(receipt.status().is_some())),
        }
    }
}

fn spawn_at_admission<T: Send + 'static>(
    action: impl FnOnce() -> T + Send + 'static,
) -> (thread::JoinHandle<T>, mpsc::Receiver<()>) {
    let (sender, receiver) = mpsc::channel();
    let worker = thread::spawn(move || {
        let _observer = ProjectGenerationAdmissionObserver::new(sender);
        action()
    });
    (worker, receiver)
}

fn wait_for_admission(receiver: mpsc::Receiver<()>) {
    receiver
        .recv_timeout(Duration::from_secs(20))
        .expect("the specified operation must enqueue its real project generation waiter");
}

fn resource_records(resources: &ResourceManager) -> Vec<ResourceRecord> {
    let mut records = resources.registry().values().cloned().collect::<Vec<_>>();
    records.sort_by_key(|record| record.primary_locator().to_string());
    records
}

fn committed_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        if !directory.is_dir() {
            return;
        }
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                collect(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    for directory in ["assets", ".zircon/cache/assets", ".zircon/registry"] {
        collect(root, &root.join(directory), &mut files);
    }
    files
}

fn rejects_reopened_activation(mutation: Mutation) {
    let fixture = Fixture::new(&format!("candidate_activation_{mutation:?}"));
    let original = fixture.manager.current_project_snapshot().unwrap();
    let old_identity = original.activation_identity().clone();
    let catalog = original.catalog_input_generation();
    let old_watchers = fixture
        .manager
        .watcher_activation
        .lock()
        .unwrap()
        .clone()
        .unwrap();
    let expected_uris = original
        .registry()
        .values()
        .map(|record| record.primary_locator().to_string())
        .collect::<BTreeSet<_>>();

    let disk_gate = lock_project_generation(&fixture.root).unwrap();
    let opener_manager = fixture.manager.clone();
    let (opener, opener_admitted) = spawn_at_admission(move || {
        AssetManager::open_prepared_project(&opener_manager, original)
            .map(|_| ())
            .map_err(|error| error.to_string())
    });
    wait_for_admission(opener_admitted);
    let preparation_epoch = fixture.manager.current_project_preparation_epoch();
    assert!(fixture.manager.is_project_activation_current(&old_identity));

    // Capture the actual replacement publication inside its existing generation fence.
    // The callback retains only resources and a path, avoiding a PAM subscriber cycle.
    let opener_thread = opener.thread().id();
    let resources = fixture.manager.resource_manager();
    let root = fixture.root.clone();
    let captured = AtomicBool::new(false);
    let (published_sender, published_receiver) = mpsc::channel();
    let changes = fixture
        .manager
        .subscribe_asset_changes_with_wake(Arc::new(move || {
            if thread::current().id() == opener_thread && !captured.swap(true, Ordering::AcqRel) {
                published_sender
                    .send((resource_records(&resources), committed_files(&root)))
                    .unwrap();
            }
        }));
    let (mutator, mutator_admitted) = spawn_at_admission(fixture.operation(mutation));
    wait_for_admission(mutator_admitted);
    assert_eq!(
        fixture.manager.current_project_preparation_epoch(),
        preparation_epoch
    );
    assert!(fixture.manager.is_project_activation_current(&old_identity));

    // FIFO admission makes the real unchanged snapshot reopen publish before the
    // already-cloned direct mutation can acquire this root's preparation authority.
    drop(disk_gate);
    opener
        .join()
        .unwrap()
        .expect("unchanged prepared snapshot reopen must succeed");
    let result = mutator.join().unwrap();
    let error =
        result.expect_err("a direct candidate from the replaced activation must not commit");
    assert!(error.contains("superseded"), "{error}");

    let active = fixture.manager.current_project_snapshot().unwrap();
    assert_ne!(active.activation_identity(), &old_identity);
    assert_eq!(
        active.catalog_input_generation().sequence(),
        catalog.sequence()
    );
    assert!(Arc::ptr_eq(&active.catalog_input_generation(), &catalog));
    let current_watchers = fixture
        .manager
        .watcher_activation
        .lock()
        .unwrap()
        .clone()
        .unwrap();
    assert!(!Arc::ptr_eq(&old_watchers, &current_watchers));
    let (published_resources, published_files) = published_receiver
        .recv_timeout(Duration::from_secs(20))
        .expect("replacement publication must capture its committed state");
    assert_eq!(
        resource_records(&fixture.manager.resource_manager()),
        published_resources
    );
    assert_eq!(committed_files(&fixture.root), published_files);
    let published_changes = changes.try_iter().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(published_changes.len(), expected_uris.len());
    assert!(published_changes.iter().all(|change| {
        change.kind == AssetChangeKind::Added && expected_uris.contains(&change.uri.to_string())
    }));

    // A new call snapshots the published activation and may perform the same mutation.
    let current_identity = active.activation_identity().clone();
    fixture.operation(mutation)().expect("a current-activation mutation must succeed");
    assert!(fixture
        .manager
        .is_project_activation_current(&current_identity));
}

#[test]
fn project_candidate_activation_rejects_direct_targeted_import_after_equal_sequence_reopen() {
    rejects_reopened_activation(Mutation::Targeted);
}

#[test]
fn project_candidate_activation_rejects_direct_model_import_after_equal_sequence_reopen() {
    rejects_reopened_activation(Mutation::Model);
}

#[test]
fn project_candidate_activation_rejects_full_reimport_after_equal_sequence_reopen() {
    rejects_reopened_activation(Mutation::Reimport);
}

#[test]
fn project_candidate_activation_rejects_deletion_after_equal_sequence_reopen() {
    rejects_reopened_activation(Mutation::Deletion);
}

#[test]
fn project_candidate_activation_rejects_relocation_after_equal_sequence_reopen() {
    rejects_reopened_activation(Mutation::Relocation);
}

#[test]
fn project_candidate_activation_rejects_generated_source_after_equal_sequence_reopen() {
    rejects_reopened_activation(Mutation::Generated);
}

#[test]
fn project_candidate_activation_allows_direct_import_after_failed_unpublished_replacement() {
    let fixture = Fixture::new("candidate_activation_failed_replacement");
    let original = fixture.manager.current_project_snapshot().unwrap();
    let identity = original.activation_identity().clone();
    let catalog = original.catalog_input_generation();
    let failed_root = fixture.root.join("failed-replacement");
    let mut manifest = ProjectManifest::new(
        "Unpublished Collision",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.asset_roots = vec![
        RelPath::parse("game-assets").unwrap(),
        RelPath::parse("shared-assets").unwrap(),
    ];
    for directory in ["game-assets/data", "shared-assets/data"] {
        fs::create_dir_all(failed_root.join(directory)).unwrap();
        fs::write(failed_root.join(directory).join("collision.json"), "{}").unwrap();
    }
    manifest
        .save(failed_root.join("zircon-project.toml"))
        .unwrap();
    let replacement = ProjectManager::open(&failed_root).unwrap();
    let replacement_gate = lock_project_generation(&failed_root).unwrap();
    let active_gate = lock_project_generation(&fixture.root).unwrap();
    let opener_manager = fixture.manager.clone();
    let (opener, opener_admitted) = spawn_at_admission(move || {
        AssetManager::open_prepared_project(&opener_manager, replacement)
            .map(|_| ())
            .map_err(|error| error.to_string())
    });
    wait_for_admission(opener_admitted);
    let epoch = fixture.manager.current_project_preparation_epoch();
    let (mutator, mutator_admitted) = spawn_at_admission(fixture.operation(Mutation::Targeted));
    wait_for_admission(mutator_admitted);

    drop(replacement_gate);
    assert!(opener.join().unwrap().is_err());
    assert!(fixture.manager.is_project_activation_current(&identity));
    assert_eq!(fixture.manager.current_project_preparation_epoch(), epoch);
    assert!(Arc::ptr_eq(
        &fixture
            .manager
            .current_project_snapshot()
            .unwrap()
            .catalog_input_generation(),
        &catalog,
    ));
    drop(active_gate);
    mutator
        .join()
        .unwrap()
        .expect("failed unpublished replacement must preserve the active candidate");
    assert!(fixture.manager.is_project_activation_current(&identity));
}

#[test]
fn project_candidate_activation_scoped_imports_keep_the_early_admission_check() {
    let fixture = Fixture::new("candidate_activation_scoped_admission");
    let snapshot = fixture.manager.current_project_snapshot().unwrap();
    let identity = snapshot.activation_identity().clone();
    fixture.manager.open_prepared_project(snapshot).unwrap();
    let targeted = fixture
        .manager
        .import_asset_for_project(&identity, &fixture.source.to_string())
        .unwrap_err();
    assert!(targeted.to_string().contains("inactive project activation"));
    let model = fixture
        .manager
        .import_model_source_for_project(&identity, &fixture.root.join("missing.obj"))
        .unwrap_err();
    assert!(model.to_string().contains("inactive project activation"));
}

#[test]
fn project_candidate_activation_watch_refresh_snapshots_after_queued_reopen() {
    let fixture = Fixture::new("candidate_activation_watch_order");
    let snapshot = fixture.manager.current_project_snapshot().unwrap();
    let old_identity = snapshot.activation_identity().clone();
    let catalog = snapshot.catalog_input_generation();
    let disk_gate = lock_project_generation(&fixture.root).unwrap();
    let opener_manager = fixture.manager.clone();
    let (opener, opener_admitted) = spawn_at_admission(move || {
        AssetManager::open_prepared_project(&opener_manager, snapshot)
            .map(|_| ())
            .map_err(|error| error.to_string())
    });
    wait_for_admission(opener_admitted);
    let watch_manager = fixture.manager.clone();
    let source = fixture.source.clone();
    let watch_errors = fixture.manager.subscribe_asset_watch_errors();
    let (watch, watch_admitted) = spawn_at_admission(move || {
        watch_manager.process_watch_batch_in_generation(AssetWatchBatch {
            changes: vec![AssetChange::new(AssetChangeKind::Modified, source, None)],
            ..AssetWatchBatch::default()
        });
    });
    wait_for_admission(watch_admitted);
    drop(disk_gate);
    opener
        .join()
        .unwrap()
        .expect("unchanged snapshot reopen must succeed");
    let published = fixture.manager.current_project_snapshot().unwrap();
    let published_identity = published.activation_identity().clone();
    assert_ne!(&published_identity, &old_identity);
    watch.join().unwrap();
    assert!(fixture
        .manager
        .is_project_activation_current(&published_identity));
    assert!(Arc::ptr_eq(
        &fixture
            .manager
            .current_project_snapshot()
            .unwrap()
            .catalog_input_generation(),
        &catalog,
    ));
    assert!(watch_errors.try_recv().is_err());
}
