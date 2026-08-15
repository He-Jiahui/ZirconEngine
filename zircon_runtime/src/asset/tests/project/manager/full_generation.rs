use super::*;
use crate::core::resource::io::transaction::DurableCommitReport;
use crate::core::resource::ResourceRecord;

#[test]
fn full_generation_commit_failure_rolls_back_every_visible_file_and_live_registry() {
    let root = unique_temp_project_root("project_manager_full_generation_rollback");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "FullGenerationRollbackSandbox",
        AssetUri::parse("res://data/first.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let assets = paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    let first_path = assets.join("data/first.counted");
    let second_path = assets.join("data/second.counted");
    fs::create_dir_all(first_path.parent().unwrap()).unwrap();
    fs::write(&first_path, "first-v1").unwrap();
    fs::write(&second_path, "second-v1").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();

    let first_uri = AssetUri::parse("res://data/first.counted").unwrap();
    let second_uri = AssetUri::parse("res://data/second.counted").unwrap();
    let artifact_paths = [&first_uri, &second_uri]
        .into_iter()
        .map(|uri| {
            let record = manager.registry().get_by_locator(uri).unwrap();
            paths
                .asset_artifact_root()
                .join(record.artifact_locator().unwrap().path())
        })
        .collect::<Vec<_>>();
    let meta_paths = [
        first_path.with_file_name("first.counted.zmeta"),
        second_path.with_file_name("second.counted.zmeta"),
    ];
    let registry_path = paths.registry_root().join("asset-registry.json");
    let artifact_before = artifact_paths
        .iter()
        .map(fs::read)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let meta_before = meta_paths
        .iter()
        .map(fs::read)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let persisted_registry_before = fs::read(&registry_path).unwrap();
    let mut resource_registry_before = manager.registry().values().cloned().collect::<Vec<_>>();
    resource_registry_before.sort_by_key(|record| record.id().to_string());
    let asset_registry_before = manager.asset_registry().clone();

    fs::write(&first_path, "first-v2").unwrap();
    fs::write(&second_path, "second-v2").unwrap();
    manager
        .scan_and_import_with_commit_failure_before_registry(None)
        .unwrap_err();

    assert_eq!(
        artifact_paths
            .iter()
            .map(fs::read)
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        artifact_before
    );
    assert_eq!(
        meta_paths
            .iter()
            .map(fs::read)
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        meta_before
    );
    assert_eq!(fs::read(registry_path).unwrap(), persisted_registry_before);
    let mut resource_registry_after = manager.registry().values().cloned().collect::<Vec<_>>();
    resource_registry_after.sort_by_key(|record| record.id().to_string());
    assert_eq!(resource_registry_after, resource_registry_before);
    assert_eq!(manager.asset_registry(), &asset_registry_before);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn full_generation_rejects_a_sidecar_that_changes_after_prepare() {
    let root = unique_temp_project_root("project_manager_full_generation_stale_meta");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "FullGenerationStaleMetaSandbox",
        AssetUri::parse("res://data/stale.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/stale.counted");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "stale-v1").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let uri = AssetUri::parse("res://data/stale.counted").unwrap();
    let record = manager.registry().get_by_locator(&uri).unwrap();
    let artifact_path = paths
        .asset_artifact_root()
        .join(record.artifact_locator().unwrap().path());
    let artifact_before = fs::read(&artifact_path).unwrap();
    let registry_before = fs::read(paths.registry_root().join("asset-registry.json")).unwrap();

    fs::write(&source, "stale-v2").unwrap();
    let mut candidate = manager.clone();
    let (_, prepared) = candidate.prepare_full_generation(None).unwrap();
    let meta_path = source.with_file_name("stale.counted.zmeta");
    let mut concurrently_updated = AssetMetaDocument::load(&meta_path).unwrap();
    concurrently_updated.preview_state = crate::asset::project::PreviewState::Dirty;
    concurrently_updated.save(&meta_path).unwrap();

    let error = prepared.commit().unwrap_err();

    assert!(error
        .to_string()
        .contains("project metadata changed while full generation was prepared"));
    assert_eq!(
        AssetMetaDocument::load(&meta_path).unwrap().preview_state,
        crate::asset::project::PreviewState::Dirty
    );
    assert_eq!(fs::read(artifact_path).unwrap(), artifact_before);
    assert_eq!(
        fs::read(paths.registry_root().join("asset-registry.json")).unwrap(),
        registry_before
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_generation_restart_rolls_back_every_preterminal_phase() {
    for (label, inject) in [
        (
            "staging",
            ProjectManager::scan_and_import_with_staging_interruption
                as fn(&mut ProjectManager) -> Result<Vec<ResourceRecord>, AssetImportError>,
        ),
        (
            "target-replace",
            ProjectManager::scan_and_import_with_target_replace_interruption,
        ),
        (
            "last-committed",
            ProjectManager::scan_and_import_with_last_commit_interruption,
        ),
    ] {
        let mut fixture = durable_generation_fixture(&format!("rollback_{label}"));
        let before = fixture.snapshot();
        fs::write(&fixture.source, "generation-v2").unwrap();
        let mut manager = fixture.manager.take().unwrap();

        inject(&mut manager).expect_err("injected interruption must retain a recovery journal");
        assert!(journal_entry_count(&fixture.paths) > 0);
        drop(manager);

        let mut reopened = ProjectManager::open(&fixture.root).unwrap();
        assert_eq!(fixture.snapshot(), before, "restart must restore {label}");
        assert_eq!(journal_entry_count(&fixture.paths), 0);
        reopened
            .register_asset_importer(counted_data_importer())
            .unwrap();
        reopened.scan_and_import().unwrap();
        assert_ne!(fixture.snapshot(), before);

        let _ = fs::remove_dir_all(fixture.root);
    }
}

#[test]
fn project_generation_restart_preserves_every_terminal_generation() {
    for after_cleanup in [false, true] {
        let mut fixture = durable_generation_fixture(if after_cleanup {
            "terminal_cleanup"
        } else {
            "terminal_all_committed"
        });
        let before = fixture.snapshot();
        fs::write(&fixture.source, "generation-v2").unwrap();
        let mut manager = fixture.manager.take().unwrap();

        manager
            .scan_and_import_with_terminal_interruption(after_cleanup)
            .expect_err("injected terminal interruption must retain its journal");
        let committed = fixture.snapshot();
        assert_ne!(committed, before);
        assert!(journal_entry_count(&fixture.paths) > 0);
        drop(manager);

        let _reopened = ProjectManager::open(&fixture.root).unwrap();
        assert_eq!(fixture.snapshot(), committed);
        assert_eq!(journal_entry_count(&fixture.paths), 0);

        let _ = fs::remove_dir_all(fixture.root);
    }
}

#[test]
fn unsynced_commit_point_installs_the_live_generation_before_reporting_pending_recovery() {
    let mut fixture = durable_generation_fixture("unsynced_commit_point");
    let before = fixture.snapshot();
    let uri = AssetUri::parse("res://data/generation.counted").unwrap();
    let mut manager = fixture.manager.take().unwrap();
    let source_hash_before = manager
        .registry()
        .get_by_locator(&uri)
        .unwrap()
        .source_hash
        .clone();
    fs::write(&fixture.source, "generation-v2").unwrap();

    let error = manager
        .scan_and_import_with_commit_point_sync_failure()
        .expect_err("an unsynced commit point must be reported after installing its generation");

    assert!(error.to_string().contains("durability is unresolved"));
    assert_ne!(fixture.snapshot(), before);
    assert_ne!(
        manager.registry().get_by_locator(&uri).unwrap().source_hash,
        source_hash_before
    );
    assert!(journal_entry_count(&fixture.paths) > 0);
    let _ = fs::remove_dir_all(fixture.root);
}

#[test]
fn project_open_rejects_a_forged_generation_targeting_a_source_file() {
    let mut fixture = durable_generation_fixture("forged_source_target");
    let journal_directory = fixture.paths.derived_root().join("project-generation");
    let source_before = fs::read(&fixture.source).unwrap();
    let mut report = DurableCommitReport::default();
    crate::core::resource::io::transaction::commit_prepared_files(
        &journal_directory,
        "project",
        vec![
            crate::core::resource::io::transaction::PreparedFileWrite::new(
                fixture.source.clone(),
                b"forged".to_vec(),
            ),
        ],
        crate::core::resource::io::transaction::TransactionFault::CrashAfterStaging(0),
        &mut report,
    )
    .expect_err("staging interruption must retain the forged intent");
    drop(fixture.manager.take());

    let error = ProjectManager::open(&fixture.root)
        .expect_err("project recovery policy must reject authoring-source targets");

    assert!(error
        .to_string()
        .contains("outside the durable publication set"));
    assert_eq!(fs::read(&fixture.source).unwrap(), source_before);
    assert!(journal_entry_count(&fixture.paths) > 0);
    let _ = fs::remove_dir_all(fixture.root);
}

#[test]
fn project_open_rejects_a_forged_generation_targeting_the_chunk_namespace() {
    let mut fixture = durable_generation_fixture("forged_chunk_target");
    let journal_directory = fixture.paths.derived_root().join("project-generation");
    let forged = fixture
        .paths
        .asset_artifact_root()
        .join("chunks")
        .join(format!(
            "{}.zasset",
            zircon_runtime_interface::resource::ResourceId::new()
        ));
    let mut report = DurableCommitReport::default();
    crate::core::resource::io::transaction::commit_prepared_files(
        &journal_directory,
        "project",
        vec![
            crate::core::resource::io::transaction::PreparedFileWrite::new(
                forged.clone(),
                b"forged".to_vec(),
            ),
        ],
        crate::core::resource::io::transaction::TransactionFault::CrashAfterStaging(0),
        &mut report,
    )
    .expect_err("staging interruption must retain the forged intent");
    drop(fixture.manager.take());

    let error = ProjectManager::open(&fixture.root)
        .expect_err("project recovery policy must reject the artifact chunk namespace");

    assert!(error
        .to_string()
        .contains("outside the durable publication set"));
    assert!(!forged.exists());
    assert!(journal_entry_count(&fixture.paths) > 0);
    let _ = fs::remove_dir_all(fixture.root);
}

struct DurableGenerationFixture {
    root: std::path::PathBuf,
    paths: ProjectPaths,
    manager: Option<ProjectManager>,
    source: std::path::PathBuf,
    artifact: std::path::PathBuf,
    meta: std::path::PathBuf,
    registry: std::path::PathBuf,
}

impl DurableGenerationFixture {
    fn snapshot(&self) -> Vec<Vec<u8>> {
        [&self.artifact, &self.meta, &self.registry]
            .into_iter()
            .map(fs::read)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }
}

fn durable_generation_fixture(label: &str) -> DurableGenerationFixture {
    let root = unique_temp_project_root(&format!("project_manager_durable_{label}"));
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "DurableGenerationSandbox",
        AssetUri::parse("res://data/generation.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/generation.counted");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "generation-v1").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let uri = AssetUri::parse("res://data/generation.counted").unwrap();
    let artifact = paths.asset_artifact_root().join(
        manager
            .registry()
            .get_by_locator(&uri)
            .unwrap()
            .artifact_locator()
            .unwrap()
            .path(),
    );

    DurableGenerationFixture {
        root,
        meta: source.with_file_name("generation.counted.zmeta"),
        registry: paths.registry_root().join("asset-registry.json"),
        paths,
        manager: Some(manager),
        source,
        artifact,
    }
}

fn journal_entry_count(paths: &ProjectPaths) -> usize {
    fs::read_dir(paths.derived_root().join("project-generation"))
        .map(|entries| entries.count())
        .unwrap_or(0)
}
