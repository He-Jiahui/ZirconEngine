use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use zircon_runtime::asset::project::{
    AssetSourceUnit, PreviewState, ProjectManager, ProjectManifest, ProjectPaths,
};
use zircon_runtime::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};
use zircon_runtime_interface::project::{AssetRef, RelPath};

use crate::core::asset::{AssetSourceWritePolicy, EditorAssetIndex};
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetDependencyCoverage,
    EditorAssetDetailsGeneration, EditorAssetManager, EditorAssetReadFacts, EditorAssetReadMissing,
    EditorAssetReadObservation, EditorAssetReadScope, EditorAssetReadSnapshot,
    EditorAssetReadStale, EditorAssetReadUnknown, EditorAssetRegistryReference,
};

use super::super::DefaultEditorAssetManager;

#[test]
fn readonly_asset_without_project_is_unknown_and_preserves_the_reference() {
    let manager = DefaultEditorAssetManager::new();
    let reference = reference(AssetUuid::new(), None);
    let result = read(&manager, &reference, None);
    assert!(matches!(
        result.observation,
        EditorAssetReadObservation::Unknown(EditorAssetReadUnknown::NoActiveProject)
    ));
    assert_eq!(result.requested, reference);
    assert!(result.scope.is_none());
}

#[test]
fn readonly_asset_missing_guid_does_not_adopt_an_occupied_hint() {
    let occupant = entry(AssetUuid::new(), "res://data/tree.toml");
    let fixture = Fixture::new(&[occupant.clone()]);
    let manager = fixture.manager(&[occupant]);
    let reference = reference(AssetUuid::new(), None);
    let result = read(&manager, &reference, None);
    assert!(matches!(
        result.observation,
        EditorAssetReadObservation::Missing(EditorAssetReadMissing::GuidNotRegistered)
    ));
    assert_eq!(result.requested, reference);
    assert!(result.scope.is_some());
}

#[test]
fn readonly_asset_requires_the_exact_guid_and_sub_pair() {
    let id = AssetUuid::new();
    let labeled = entry(id, "res://data/tree.toml#Channel0");
    let fixture = Fixture::new(&[labeled.clone()]);
    let manager = fixture.manager(&[]);
    let exact = reference(id, Some("Channel0"));
    assert_eq!(known(&read(&manager, &exact, None)).registry_entry, labeled);

    for sub in [None, Some("channel0"), Some("Channel1")] {
        let requested = reference(id, sub);
        let result = read(&manager, &requested, None);
        assert!(matches!(
            result.observation,
            EditorAssetReadObservation::Missing(EditorAssetReadMissing::GuidSubMismatch {
                registered_sub: Some(ref actual)
            }) if actual == "Channel0"
        ));
        assert_eq!(result.requested, requested);
    }
}

#[test]
fn readonly_asset_empty_edges_and_absent_projection_keep_coverage_unknown() {
    let id = AssetUuid::new();
    let fixture = Fixture::new(&[entry(id, "res://data/tree.toml")]);
    let manager = fixture.manager(&[]);
    let result = read(&manager, &reference(id, None), None);
    let facts = known(&result);
    assert!(facts.asset.is_none());
    assert!(facts.details.is_none());
    assert!(facts.dependencies.outgoing.is_empty());
    assert!(facts.dependencies.incoming.is_empty());
    assert_eq!(
        facts.dependencies.asset_edge_coverage,
        EditorAssetDependencyCoverage::Unknown
    );
    assert_eq!(
        facts.dependencies.semantic_coverage,
        EditorAssetDependencyCoverage::Unknown
    );
    assert!(facts.source_policy_access.is_none());
    assert_eq!(facts.registry_entry.source_digest(), "7f63a3d20dfab0ed");
}

#[test]
fn readonly_asset_keeps_unprojected_referencers_and_missing_dependency_ids() {
    let id = AssetUuid::new();
    let missing = AssetUuid::new();
    let target = entry(id, "res://data/tree.toml").with_dependencies(vec![missing]);
    let referencer =
        entry(AssetUuid::new(), "res://data/unlisted.toml").with_dependencies(vec![id]);
    let fixture = Fixture::new(&[target.clone(), referencer.clone()]);
    let manager = fixture.manager(&[target]);
    let result = read(&manager, &reference(id, None), None);
    let facts = known(&result);
    assert!(facts.details.as_ref().unwrap().referenced_by.is_empty());
    assert_eq!(
        facts.dependencies.incoming,
        vec![EditorAssetRegistryReference::Known(referencer)]
    );
    assert_eq!(
        facts.dependencies.outgoing,
        vec![EditorAssetRegistryReference::Missing(missing)]
    );
}

#[test]
fn readonly_asset_and_details_share_the_captured_catalog_after_replacement() {
    let id = AssetUuid::new();
    let target = entry(id, "res://data/tree.toml");
    let fixture = Fixture::new(&[target.clone()]);
    let manager = fixture.manager(&[target]);
    let requested = reference(id, None);
    let before = read(&manager, &requested, None);
    let scope = before.scope.as_ref().unwrap();
    let facts = known(&before);
    assert!(Arc::ptr_eq(
        facts.details.as_ref().unwrap(),
        &scope.catalog().details(&id.to_string()).unwrap()
    ));
    assert_eq!(
        facts.asset.as_deref(),
        scope.catalog().asset(&id.to_string())
    );

    // Same numeric revisions are insufficient: this is another immutable publication.
    let replacement = Arc::new(scope.catalog().as_ref().clone());
    manager.write_state_recovering_poison().catalog_generation = Arc::clone(&replacement);
    let stale = read(&manager, &requested, Some(scope));
    assert!(matches!(
        stale.observation,
        EditorAssetReadObservation::Stale(EditorAssetReadStale::ExpectedPublicationMismatch)
    ));
    assert!(Arc::ptr_eq(
        stale.scope.as_ref().unwrap().catalog(),
        &replacement
    ));
    assert_eq!(known(&before).registry_entry.uuid(), id);
    assert!(!scope.same_publication(stale.scope.as_ref().unwrap()));
}

#[test]
fn readonly_asset_scope_rejects_a_different_input_arc_with_the_same_sequence() {
    let fixture = Fixture::new(&[]);
    let manager = fixture.manager(&[]);
    let requested = reference(AssetUuid::new(), None);
    let current = read(&manager, &requested, None);
    let scope = current.scope.unwrap();
    let detached_input = Arc::new(scope.input().as_ref().clone());
    assert_eq!(detached_input.sequence(), scope.input().sequence());
    let disconnected = EditorAssetReadScope::new(
        scope.activation().clone(),
        detached_input,
        Arc::clone(scope.catalog()),
    );
    assert!(matches!(
        read(&manager, &requested, Some(&disconnected)).observation,
        EditorAssetReadObservation::Stale(EditorAssetReadStale::ExpectedPublicationMismatch)
    ));
}

#[test]
fn readonly_asset_same_root_reopen_rejects_the_retired_activation() {
    let fixture = Fixture::new(&[]);
    let manager = fixture.manager(&[]);
    let requested = reference(AssetUuid::new(), None);
    let old = read(&manager, &requested, None).scope.unwrap();
    let reopened = ProjectManager::open(&fixture.root).unwrap();
    assert_eq!(
        reopened.manifest().project_guid,
        old.input().manifest().project_guid
    );
    assert_ne!(reopened.activation_identity(), old.activation());

    // A partial replacement is stale even without a caller-supplied fence.
    manager.write_state_recovering_poison().project = Some(reopened.clone());
    assert!(matches!(
        read(&manager, &requested, None).observation,
        EditorAssetReadObservation::Stale(EditorAssetReadStale::ActivationMismatch)
    ));
    install(&manager, reopened, &[]);
    assert!(matches!(
        read(&manager, &requested, Some(&old)).observation,
        EditorAssetReadObservation::Stale(EditorAssetReadStale::ExpectedPublicationMismatch)
    ));
    assert!(matches!(
        read(&manager, &requested, None).observation,
        EditorAssetReadObservation::Missing(EditorAssetReadMissing::GuidNotRegistered)
    ));
}

#[test]
fn readonly_asset_deactivation_distinguishes_a_retired_scope_from_no_scope() {
    let fixture = Fixture::new(&[]);
    let manager = fixture.manager(&[]);
    let requested = reference(AssetUuid::new(), None);
    let old = read(&manager, &requested, None).scope.unwrap();
    EditorAssetManager::deactivate_runtime_project(&manager);
    assert!(matches!(
        read(&manager, &requested, Some(&old)).observation,
        EditorAssetReadObservation::Stale(EditorAssetReadStale::ProjectRetired)
    ));
    assert!(matches!(
        read(&manager, &requested, None).observation,
        EditorAssetReadObservation::Unknown(EditorAssetReadUnknown::NoActiveProject)
    ));
}

#[test]
fn readonly_asset_missing_index_is_unknown_and_disconnected_registry_is_stale() {
    let fixture = Fixture::new(&[]);
    let manager = fixture.manager(&[]);
    let requested = reference(AssetUuid::new(), None);
    let index = manager
        .read_state_recovering_poison()
        .asset_index
        .clone()
        .unwrap();
    let disconnected = Arc::new(index.lock().unwrap().runtime_registry().as_ref().clone());
    index.lock().unwrap().replace_runtime_registry(disconnected);
    assert!(matches!(
        read(&manager, &requested, None).observation,
        EditorAssetReadObservation::Stale(EditorAssetReadStale::RegistryMismatch)
    ));
    manager.write_state_recovering_poison().asset_index = None;
    assert!(matches!(
        read(&manager, &requested, None).observation,
        EditorAssetReadObservation::Unknown(EditorAssetReadUnknown::IndexUnavailable)
    ));
}

#[test]
fn readonly_asset_disagreed_catalog_projection_is_stale() {
    let id = AssetUuid::new();
    let target = entry(id, "res://data/tree.toml");
    let fixture = Fixture::new(&[target]);
    let manager = fixture.manager(&[entry(id, "res://data/reused.toml")]);
    assert!(matches!(
        read(&manager, &reference(id, None), None).observation,
        EditorAssetReadObservation::Stale(EditorAssetReadStale::CatalogProjectionMismatch)
    ));
}

#[test]
fn readonly_asset_same_locator_and_kind_with_a_retired_source_digest_is_stale() {
    let id = AssetUuid::new();
    let target = entry(id, "res://data/tree.toml");
    let fixture = Fixture::new(&[target.clone()]);
    let previous = AssetRegistryEntry::new(
        id,
        target.path().clone(),
        target.type_marker(),
        "retired-source-digest",
    );
    let manager = fixture.manager(&[previous]);
    let subscription = EditorAssetManager::subscribe_editor_asset_changes(&manager);
    let result = EditorAssetManager::readonly_asset_snapshot(
        &manager,
        &reference(id, None),
        None,
        Some(AssetSourceWritePolicy::ProjectOnly),
    );
    assert!(matches!(
        result.observation,
        EditorAssetReadObservation::Stale(EditorAssetReadStale::CatalogProjectionMismatch)
    ));
    // No refresh or inferred repair: retain the captured older projection for diagnosis.
    let projected = result
        .scope
        .unwrap()
        .catalog()
        .asset_shared(&id.to_string())
        .unwrap();
    assert_eq!(projected.locator, target.path().to_string());
    assert_eq!(projected.kind, target.type_marker());
    assert_eq!(projected.source_hash, "retired-source-digest");
    assert!(subscription.try_recv().is_none());
    let current = fixture.manager(&[target.clone()]);
    let result = read(&current, &reference(id, None), None);
    assert_eq!(known(&result).registry_entry, target);
}

#[test]
fn readonly_asset_returns_before_external_consumption_and_does_not_publish() {
    let id = AssetUuid::new();
    let target = entry(id, "res://data/tree.toml");
    let fixture = Fixture::new(&[target.clone()]);
    let manager = fixture.manager(&[target]);
    let subscription = EditorAssetManager::subscribe_editor_asset_changes(&manager);
    let epoch = manager.source_sync_epoch.load(Ordering::Acquire);
    let requested = reference(id, None);
    assert!(!fixture.root.join("assets/data/tree.toml").exists());

    let result = EditorAssetManager::readonly_asset_snapshot(
        &manager,
        &requested,
        None,
        Some(AssetSourceWritePolicy::ProjectOnly),
    );
    assert!(known(&result).source_policy_access.unwrap().is_writable());
    // Policy-writable does not assert source existence or perform any read/admission.
    {
        let _publish = manager
            .publish_gate
            .try_lock()
            .expect("query leaked publish guard");
        let state = manager.state.try_write().expect("query leaked state guard");
        let _index = state
            .asset_index
            .as_ref()
            .unwrap()
            .try_lock()
            .expect("query leaked index guard");
    }
    let reentrant_consumer =
        |snapshot: &EditorAssetReadSnapshot| read(&manager, &requested, snapshot.scope.as_ref());
    assert!(matches!(
        reentrant_consumer(&result).observation,
        EditorAssetReadObservation::Known(_)
    ));
    assert!(subscription.try_recv().is_none());
    assert_eq!(manager.source_sync_epoch.load(Ordering::Acquire), epoch);
}

fn read(
    manager: &dyn EditorAssetManager,
    requested: &AssetRef,
    expected: Option<&EditorAssetReadScope>,
) -> EditorAssetReadSnapshot {
    manager.readonly_asset_snapshot(requested, expected, None)
}

fn known(snapshot: &EditorAssetReadSnapshot) -> &EditorAssetReadFacts {
    match &snapshot.observation {
        EditorAssetReadObservation::Known(facts) => facts,
        other => panic!("expected a known registry entry, got {other:?}"),
    }
}

fn reference(id: AssetUuid, sub: Option<&str>) -> AssetRef {
    AssetRef::try_new(
        id,
        RelPath::parse("assets/data/tree.toml").unwrap(),
        sub.map(str::to_owned),
    )
    .unwrap()
}

fn entry(id: AssetUuid, locator: &str) -> AssetRegistryEntry {
    AssetRegistryEntry::new(
        id,
        AssetUri::parse(locator).unwrap(),
        AssetKind::Data,
        "7f63a3d20dfab0ed",
    )
}

fn install(
    manager: &DefaultEditorAssetManager,
    project: ProjectManager,
    projected: &[AssetRegistryEntry],
) {
    let index = EditorAssetIndex::from_runtime_project(&project).unwrap();
    let assets = projected
        .iter()
        .map(|entry| {
            Arc::new(EditorAssetCatalogRecord {
                uuid: entry.uuid().to_string(),
                id: entry.uuid().to_string(),
                locator: entry.path().to_string(),
                kind: entry.type_marker(),
                display_name: "Tree".to_owned(),
                file_name: "tree.toml".to_owned(),
                extension: "toml".to_owned(),
                preview_state: PreviewState::Dirty,
                meta_path: String::new(),
                preview_artifact_path: String::new(),
                source_mtime_unix_ms: 0,
                source_hash: entry.source_digest().to_owned(),
                dirty: false,
                diagnostics: Vec::new(),
                direct_reference_uuids: Vec::new(),
            })
        })
        .collect::<Vec<_>>();
    let details = assets
        .iter()
        .map(|asset| {
            Some(Arc::new(EditorAssetDetailsGeneration {
                asset: Arc::clone(asset),
                direct_references: Vec::new().into(),
                referenced_by: Vec::new().into(),
                package_id: None,
                unit: AssetSourceUnit::Single,
                included_files: Vec::new().into(),
                subassets: Vec::new().into(),
            }))
        })
        .collect();
    let catalog = EditorAssetCatalogGeneration::from_parts(
        project.manifest().name.clone(),
        project.paths().root().to_string_lossy().into_owned(),
        project
            .primary_project_asset_root()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        project.paths().cache_root().to_string_lossy().into_owned(),
        project.manifest().default_scene.to_string(),
        1,
        1,
        Vec::new(),
        assets,
        details,
        vec![None; projected.len()],
    );
    let _publish = manager.publish_gate.lock().unwrap();
    let mut state = manager.write_state_recovering_poison();
    state.project_root = Some(project.paths().root().to_owned());
    state.project_name = project.manifest().name.clone();
    state.catalog_generation = Arc::new(catalog);
    state.asset_index = Some(Arc::new(Mutex::new(index)));
    state.project = Some(project);
}

/// Future managed tests create fixtures beside their physically resolved executable,
/// never in the checkout or OS temp. These fixture builders are not run by static checks.
struct Fixture {
    root: PathBuf,
    parent: PathBuf,
}

impl Fixture {
    fn new(entries: &[AssetRegistryEntry]) -> Self {
        let executable = std::env::current_exe().unwrap();
        let binary = ProjectPaths::resolve_existing(executable.parent().unwrap()).unwrap();
        assert!(
            approved_output(binary.display_path()),
            "managed test output root required"
        );
        let parent = binary.operation_path().join("asset-readonly-fixtures");
        fs::create_dir_all(&parent).unwrap();
        let parent = ProjectPaths::resolve_existing(&parent).unwrap();
        assert!(
            approved_output(parent.display_path()),
            "fixture parent escaped output root"
        );
        let root = parent.operation_path().join(AssetUuid::new().to_string());
        fs::create_dir(&root).unwrap();
        let fixture = Self {
            root,
            parent: parent.into_operation_path(),
        };
        let paths = ProjectPaths::from_root(&fixture.root).unwrap();
        paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
        ProjectManifest::new(
            "Readonly Query",
            AssetUri::parse("res://main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        let registry = AssetRegistryIndex::from_entries(entries.iter().cloned()).unwrap();
        fs::write(
            paths.registry_root().join("asset-registry.json"),
            serde_json::to_vec(
                &serde_json::json!({"format_version": 1, "entries": registry.entries()}),
            )
            .unwrap(),
        )
        .unwrap();
        fixture
    }

    fn manager(&self, projected: &[AssetRegistryEntry]) -> DefaultEditorAssetManager {
        let manager = DefaultEditorAssetManager::new();
        install(
            &manager,
            ProjectManager::open(&self.root).unwrap(),
            projected,
        );
        manager
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Resolve the exact owned leaf before any recursive removal.
        let resolved = ProjectPaths::resolve_existing(&self.root).expect("resolve owned fixture");
        assert!(
            approved_output(resolved.display_path()),
            "fixture cleanup escaped output root"
        );
        assert_eq!(
            resolved.operation_path().parent(),
            Some(self.parent.as_path())
        );
        fs::remove_dir_all(resolved.operation_path()).expect("remove only this owned fixture");
    }
}

fn approved_output(path: &Path) -> bool {
    ["D:/cargo-targets", "E:/cargo-targets", "F:/cargo-targets"]
        .iter()
        .any(|root| path.starts_with(root))
}
