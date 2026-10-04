use std::fs;
use std::time::Duration;

use zircon_runtime_interface::hub_protocol::{
    HubRecentProjectsStore, HubRecentProjectsWritePolicy,
};

use crate::projects::{
    load_shared_recent_projects, normalize_project_root, project_metadata_key,
    reconcile_shared_recent_projects, RecentProject,
};
use crate::settings::{BuildProfile, HubConfig, HubLanguage};

use super::*;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let target_directory = std::env::var_os("CARGO_TARGET_DIR")
        .expect("Hub runtime filesystem tests require coordinator-managed CARGO_TARGET_DIR");
    let path = PathBuf::from(target_directory).join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn create_valid_source_checkout(source_path: &Path) {
    fs::create_dir_all(source_path.join("tools")).unwrap();
    fs::create_dir_all(source_path.join("zircon_runtime")).unwrap();
    fs::write(
        source_path.join("Cargo.toml"),
        "[workspace]\nmembers = [\"zircon_runtime\"]\n",
    )
    .unwrap();
    fs::write(source_path.join("tools").join("zircon_build.py"), "").unwrap();
}

#[test]
fn published_view_models_share_an_epoch_and_advance_lossless_revisions() {
    let temp = temp_test_dir("zircon-hub-state-publication-revision");
    let mut session = HubRuntimeSession::load_from_paths(
        temp.join("hub.toml"),
        temp.join("recent_projects.json"),
    )
    .unwrap();

    let first = session.publish_view_model();
    let repeated_read = session.view_model();
    let second = session
        .apply_action(HubActionRequest {
            action_id: "show-page".to_string(),
            target_id: Some("editor".to_string()),
            payload: None,
        })
        .unwrap();

    assert!(!first.backend_epoch.is_empty());
    assert_eq!(first.backend_epoch, second.backend_epoch);
    assert_eq!(first.state_revision, "1");
    assert_eq!(repeated_read.state_revision, first.state_revision);
    assert_eq!(second.state_revision, "2");
    assert_eq!(second.active_page, "editor");
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn separate_runtime_sessions_use_distinct_backend_epochs() {
    let first_temp = temp_test_dir("zircon-hub-state-publication-epoch-a");
    let second_temp = temp_test_dir("zircon-hub-state-publication-epoch-b");
    let first = HubRuntimeSession::load_from_paths(
        first_temp.join("hub.toml"),
        first_temp.join("recent_projects.json"),
    )
    .unwrap();
    let second = HubRuntimeSession::load_from_paths(
        second_temp.join("hub.toml"),
        second_temp.join("recent_projects.json"),
    )
    .unwrap();

    assert_ne!(first.backend_epoch, second.backend_epoch);
    fs::remove_dir_all(first_temp).unwrap();
    fs::remove_dir_all(second_temp).unwrap();
}

#[test]
fn startup_selection_preserves_persisted_stale_project_path() {
    let recent_projects = vec![RecentProject::fixture("Recent", "E:/Projects/Recent", 30)];

    let selected =
        startup_selected_project_path(Some(Path::new("E:/Projects/Missing")), &recent_projects);

    assert_eq!(selected, Some(PathBuf::from("E:/Projects/Missing")));
}

#[test]
fn focus_refresh_merges_editor_recents_without_rewriting_shared_registry() {
    let temp = temp_test_dir("zircon-hub-tauri-focus-recents");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let editor_project_path = temp.join("EditorGame");
    fs::create_dir_all(&editor_project_path).unwrap();

    let mut session = HubRuntimeSession::load_from_paths(
        config_path.clone(),
        shared_recent_projects_path.clone(),
    )
    .expect("Tauri runtime session should load");
    let editor_project = RecentProject::fixture("Editor Game", &editor_project_path, 42);
    reconcile_shared_recent_projects(
        &shared_recent_projects_path,
        &[],
        std::slice::from_ref(&editor_project),
    )
    .expect("Editor fixture should write the shared recent-project registry");

    let registry: serde_json::Value =
        serde_json::from_slice(&fs::read(&shared_recent_projects_path).unwrap()).unwrap();
    fs::write(
        &shared_recent_projects_path,
        serde_json::to_vec(&registry).unwrap(),
    )
    .unwrap();
    let shared_registry_before_refresh = fs::read(&shared_recent_projects_path).unwrap();

    assert!(session
        .refresh_shared_recent_projects_on_focus()
        .expect("focus refresh should merge the Editor update"));
    assert_eq!(session.config.recent_projects, vec![editor_project.clone()]);
    assert_eq!(
        HubConfig::load(&config_path).unwrap().recent_projects,
        vec![editor_project]
    );
    assert_eq!(
        fs::read(&shared_recent_projects_path).unwrap(),
        shared_registry_before_refresh
    );

    let config_after_refresh = fs::read(&config_path).unwrap();
    assert!(!session
        .refresh_shared_recent_projects_on_focus()
        .expect("unchanged shared registry should not refresh again"));
    assert_eq!(fs::read(&config_path).unwrap(), config_after_refresh);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn focus_refresh_retries_hub_config_persistence_after_a_save_failure() {
    let temp = temp_test_dir("zircon-hub-tauri-focus-recents-retry");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let editor_project_path = temp.join("EditorGame");
    fs::create_dir_all(&editor_project_path).unwrap();

    let mut session = HubRuntimeSession::load_from_paths(
        config_path.clone(),
        shared_recent_projects_path.clone(),
    )
    .expect("Tauri runtime session should load");
    let editor_project = RecentProject::fixture("Editor Game", &editor_project_path, 42);
    reconcile_shared_recent_projects(
        &shared_recent_projects_path,
        &[],
        std::slice::from_ref(&editor_project),
    )
    .expect("Editor fixture should write the shared recent-project registry");

    let blocked_parent = temp.join("blocked-parent");
    fs::write(&blocked_parent, "not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");
    assert!(session.refresh_shared_recent_projects_on_focus().is_err());

    session.config_path = config_path.clone();
    assert!(!session
        .refresh_shared_recent_projects_on_focus()
        .expect("focus refresh should retry the failed Hub config write"));
    assert_eq!(
        HubConfig::load(&config_path).unwrap().recent_projects,
        vec![editor_project]
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn focus_refresh_does_not_reopen_an_editor_tombstone_after_config_save_fails() {
    let temp = temp_test_dir("zircon-hub-focus-save-failure-tombstone");
    let config_path = temp.join("hub.toml");
    let registry_path = temp.join("recent_projects.json");
    let project = RecentProject::fixture("Game", temp.join("Game"), 42);
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), registry_path.clone())
            .expect("load empty Hub session");
    session.config.recent_projects = vec![project.clone()];

    let blocked_parent = temp.join("blocked-parent");
    fs::write(&blocked_parent, "not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");
    assert!(session.refresh_shared_recent_projects_on_focus().is_err());
    assert_eq!(session.shared_recent_projects_snapshot.projects().len(), 1);

    let store = HubRecentProjectsStore::new(&registry_path);
    store
        .update(
            HubRecentProjectsWritePolicy::with_timeout(Duration::from_millis(50)),
            |registry| registry.remove(&project.path),
        )
        .expect("Editor should commit a deletion after Hub's failed config save");
    let deleted = store.load_projection().unwrap().registry().clone();
    session.config_path = config_path.clone();
    assert!(session
        .refresh_shared_recent_projects_on_focus()
        .expect("retry should load the Editor tombstone"));

    assert!(session.config.recent_projects.is_empty());
    assert!(HubConfig::load(&config_path)
        .unwrap()
        .recent_projects
        .is_empty());
    assert_eq!(store.load_projection().unwrap().registry(), &deleted);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn focus_refresh_reconciles_pending_hub_recents_with_editor_registry() {
    let temp = temp_test_dir("zircon-hub-tauri-focus-recents-merge");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let hub_project = RecentProject::fixture("Hub Game", temp.join("HubGame"), 24);
    let editor_project = RecentProject::fixture("Editor Game", temp.join("EditorGame"), 42);

    let mut session = HubRuntimeSession::load_from_paths(
        config_path.clone(),
        shared_recent_projects_path.clone(),
    )
    .expect("Tauri runtime session should load");
    session.config.recent_projects = vec![hub_project.clone()];
    reconcile_shared_recent_projects(
        &shared_recent_projects_path,
        &[],
        std::slice::from_ref(&editor_project),
    )
    .expect("Editor fixture should write the shared recent-project registry");

    assert!(session
        .refresh_shared_recent_projects_on_focus()
        .expect("focus refresh should reconcile the pending Hub update"));
    for project in [&hub_project, &editor_project] {
        let expected_key = project_metadata_key(normalize_project_root(&project.path));
        let actual_keys = session
            .config
            .recent_projects
            .iter()
            .map(|candidate| project_metadata_key(&candidate.path))
            .collect::<Vec<_>>();
        assert!(
            session
                .config
                .recent_projects
                .iter()
                .any(|candidate| project_metadata_key(&candidate.path) == expected_key),
            "missing {expected_key:?} from Hub recents {actual_keys:?}"
        );
    }
    let shared_projects = load_shared_recent_projects(&shared_recent_projects_path).unwrap();
    for project in [&hub_project, &editor_project] {
        let expected_key = project_metadata_key(normalize_project_root(&project.path));
        assert!(shared_projects
            .iter()
            .any(|candidate| project_metadata_key(&candidate.path) == expected_key));
    }

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn load_from_paths_syncs_shared_recents_repairs_and_persists_runtime_state() {
    let temp = temp_test_dir("zircon-hub-tauri-runtime-load");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let project_path = temp.join("Game");
    let source_path = temp.join("ZirconEngine");
    fs::create_dir_all(&project_path).unwrap();
    create_valid_source_checkout(&source_path);

    let mut config = HubConfig::default();
    config.recent_projects = vec![RecentProject::fixture("Game", &project_path, 4)];
    config.project_metadata.insert(
        project_metadata_key(&project_path),
        crate::projects::ProjectMetadata {
            pinned: true,
            engine_id: Some("missing-engine".to_string()),
            last_selected_template: None,
        },
    );
    config.settings.default_source_dir = source_path.clone();
    config.settings.default_build_output_dir = temp.join("out");
    config.runtime.selected_project_path = Some(project_path.clone());
    config.save(&config_path).unwrap();
    let session = HubRuntimeSession::load_from_paths(
        config_path.clone(),
        shared_recent_projects_path.clone(),
    )
    .expect("Tauri runtime session should load and persist");

    assert_eq!(session.selected_project_path, Some(project_path.clone()));
    assert_eq!(session.config.engines.len(), 1);
    assert_eq!(
        session.config.active_engine_id.as_deref(),
        Some(source_engine_id(&source_path).as_str())
    );
    assert_eq!(
        session
            .config
            .project_metadata
            .get(&project_metadata_key(&project_path))
            .and_then(|metadata| metadata.engine_id.as_deref()),
        None
    );
    let saved = HubConfig::load(&config_path).unwrap();
    assert_eq!(saved.runtime.selected_project_path, Some(project_path));
    assert_eq!(
        crate::projects::load_shared_recent_projects(&shared_recent_projects_path)
            .expect("read shared recent projects")
            .len(),
        1
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn restart_preserves_editor_tombstone_for_previously_synced_recent() {
    let temp = temp_test_dir("zircon-hub-recent-offline-editor-delete");
    let config_path = temp.join("hub.toml");
    let registry_path = temp.join("recent_projects.json");
    let project = RecentProject::fixture("Game", temp.join("Game"), 4);
    let mut config = HubConfig::default();
    config.recent_projects = vec![project.clone()];
    config.save(&config_path).unwrap();

    let first = HubRuntimeSession::load_from_paths(config_path.clone(), registry_path.clone())
        .expect("first Hub startup should migrate legacy recent projects");
    assert_eq!(first.config.recent_projects.len(), 1);
    drop(first);

    let store = HubRecentProjectsStore::new(&registry_path);
    store
        .update(
            HubRecentProjectsWritePolicy::with_timeout(Duration::from_millis(50)),
            |registry| registry.remove(&project.path),
        )
        .expect("Editor should persist a deletion while Hub is offline");
    let deleted = store.load_projection().unwrap().registry().clone();
    assert!(deleted.projects.is_empty());
    assert_eq!(deleted.tombstones.len(), 1);

    let restarted = HubRuntimeSession::load_from_paths(config_path.clone(), registry_path.clone())
        .expect("Hub restart should accept the newer shared projection");
    let after_restart = store.load_projection().unwrap().registry().clone();
    assert!(restarted.config.recent_projects.is_empty());
    assert!(HubConfig::load(&config_path)
        .unwrap()
        .recent_projects
        .is_empty());
    assert_eq!(after_restart.revision(), deleted.revision());
    assert_eq!(after_restart.tombstones, deleted.tombstones);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn persist_retry_does_not_reopen_an_editor_tombstone_after_config_save_fails() {
    let temp = temp_test_dir("zircon-hub-persist-save-failure-tombstone");
    let config_path = temp.join("hub.toml");
    let registry_path = temp.join("recent_projects.json");
    let project = RecentProject::fixture("Game", temp.join("Game"), 42);
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), registry_path.clone())
            .expect("load empty Hub session");
    session.config.recent_projects = vec![project.clone()];

    let blocked_parent = temp.join("blocked-parent");
    fs::write(&blocked_parent, "not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");
    assert!(session.persist_unchecked().is_err());
    assert_eq!(session.shared_recent_projects_snapshot.projects().len(), 1);

    let store = HubRecentProjectsStore::new(&registry_path);
    store
        .update(
            HubRecentProjectsWritePolicy::with_timeout(Duration::from_millis(50)),
            |registry| registry.remove(&project.path),
        )
        .expect("Editor should commit a deletion after Hub's failed config save");
    let deleted = store.load_projection().unwrap().registry().clone();
    session.config_path = config_path.clone();
    session
        .persist_unchecked()
        .expect("retry should preserve the Editor tombstone");

    assert!(session.config.recent_projects.is_empty());
    assert!(HubConfig::load(&config_path)
        .unwrap()
        .recent_projects
        .is_empty());
    assert_eq!(store.load_projection().unwrap().registry(), &deleted);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn legacy_config_imports_once_into_a_clean_revision_zero_registry() {
    let temp = temp_test_dir("zircon-hub-recent-legacy-import");
    let config_path = temp.join("hub.toml");
    let registry_path = temp.join("recent_projects.json");
    let project = RecentProject::fixture("Legacy", temp.join("Legacy"), 4);
    let mut config = HubConfig::default();
    config.recent_projects = vec![project.clone()];
    config.save(&config_path).unwrap();

    let first = HubRuntimeSession::load_from_paths(config_path.clone(), registry_path.clone())
        .expect("clean first startup should import legacy recents");
    let initial = HubRecentProjectsStore::new(&registry_path)
        .load_projection()
        .unwrap()
        .registry()
        .clone();
    assert_eq!(first.config.recent_projects.len(), 1);
    assert_eq!(initial.projects.len(), 1);
    assert!(initial.revision() > 0);
    assert_eq!(
        HubConfig::load(&config_path)
            .unwrap()
            .last_seen_shared_recent_revision,
        Some(initial.revision()),
    );
    drop(first);

    let second = HubRuntimeSession::load_from_paths(config_path, registry_path.clone())
        .expect("a later startup should read the synchronized registry");
    assert_eq!(second.config.recent_projects.len(), 1);
    assert_eq!(
        HubRecentProjectsStore::new(&registry_path)
            .load_projection()
            .unwrap()
            .registry()
            .revision(),
        initial.revision(),
    );
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn legacy_config_is_not_reimported_after_a_synced_registry_resets_to_zero() {
    let temp = temp_test_dir("zircon-hub-recent-registry-reset");
    let config_path = temp.join("hub.toml");
    let registry_path = temp.join("recent_projects.json");
    let project = RecentProject::fixture("Legacy", temp.join("Legacy"), 4);
    let mut config = HubConfig::default();
    config.recent_projects = vec![project];
    config.save(&config_path).unwrap();

    let first = HubRuntimeSession::load_from_paths(config_path.clone(), registry_path.clone())
        .expect("first Hub startup should migrate legacy recents");
    assert!(first.config.last_seen_shared_recent_revision.is_some());
    drop(first);
    fs::remove_file(&registry_path).unwrap();

    let restarted = HubRuntimeSession::load_from_paths(config_path.clone(), registry_path.clone())
        .expect("a previously synchronized Hub should accept an empty rebuilt registry");
    assert!(restarted.config.recent_projects.is_empty());
    assert!(HubConfig::load(&config_path)
        .unwrap()
        .recent_projects
        .is_empty());
    assert_eq!(
        HubRecentProjectsStore::new(&registry_path)
            .load_projection()
            .unwrap()
            .registry()
            .revision(),
        0,
    );
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn legacy_import_conflict_reloads_a_new_editor_tombstone() {
    let temp = temp_test_dir("zircon-hub-recent-legacy-conflict");
    let registry_path = temp.join("recent_projects.json");
    let store = HubRecentProjectsStore::new(&registry_path);
    let observed = store
        .load_projection()
        .expect("observe clean empty registry");
    let legacy = RecentProject::fixture("Game", temp.join("Game"), 1);

    store
        .update(
            HubRecentProjectsWritePolicy::with_timeout(Duration::from_millis(50)),
            |registry| registry.remove(&legacy.path),
        )
        .expect("Editor deletion should commit after Hub's startup read");
    let deleted = store.load_projection().unwrap().registry().clone();

    let startup = reconcile_startup_shared_recent_projects_from_observed(
        &registry_path,
        observed,
        std::slice::from_ref(&legacy),
        None,
    )
    .expect("revision conflict should reload canonical Editor state");
    let after = store.load_projection().unwrap().registry().clone();
    assert!(startup.projects().is_empty());
    assert_eq!(startup.revision(), deleted.revision());
    assert_eq!(after, deleted);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn legacy_import_skips_corruption_after_a_clean_startup_read() {
    let temp = temp_test_dir("zircon-hub-recent-legacy-corruption-race");
    let registry_path = temp.join("recent_projects.json");
    let store = HubRecentProjectsStore::new(&registry_path);
    let observed = store
        .load_projection()
        .expect("observe clean empty registry");
    let legacy = RecentProject::fixture("Stale", temp.join("Stale"), 1);
    fs::write(&registry_path, b"{ malformed").unwrap();

    let snapshot = reconcile_startup_shared_recent_projects_from_observed(
        &registry_path,
        observed,
        std::slice::from_ref(&legacy),
        None,
    )
    .expect("rebuildable corruption should leave startup available without legacy import");

    assert!(snapshot.projects().is_empty());
    assert_eq!(snapshot.revision(), 0);
    assert_eq!(fs::read(&registry_path).unwrap(), b"{ malformed");
    assert!(!fs::read_dir(&temp).unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains(".corrupt-")));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn legacy_config_cannot_override_an_existing_revisioned_registry() {
    let temp = temp_test_dir("zircon-hub-recent-canonical-registry");
    let config_path = temp.join("hub.toml");
    let registry_path = temp.join("recent_projects.json");
    let stale = RecentProject::fixture("Stale", temp.join("Stale"), 4);
    let current = RecentProject::fixture("Current", temp.join("Current"), 9);
    let mut config = HubConfig::default();
    config.recent_projects = vec![stale];
    config.save(&config_path).unwrap();
    reconcile_shared_recent_projects(&registry_path, &[], std::slice::from_ref(&current))
        .expect("Editor should write the canonical registry first");
    let before = HubRecentProjectsStore::new(&registry_path)
        .load_projection()
        .unwrap()
        .registry()
        .clone();

    let session = HubRuntimeSession::load_from_paths(config_path.clone(), registry_path.clone())
        .expect("Hub should accept the canonical shared revision");
    let after = HubRecentProjectsStore::new(&registry_path)
        .load_projection()
        .unwrap()
        .registry()
        .clone();
    assert_eq!(session.config.recent_projects.len(), 1);
    assert_eq!(session.config.recent_projects[0].summary.name, "Current");
    assert_eq!(after, before);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn malformed_shared_registry_does_not_admit_legacy_config_rows() {
    let temp = temp_test_dir("zircon-hub-recent-malformed-registry");
    let config_path = temp.join("hub.toml");
    let registry_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.recent_projects = vec![RecentProject::fixture("Stale", temp.join("Stale"), 4)];
    config.save(&config_path).unwrap();
    fs::write(&registry_path, b"{ malformed").unwrap();

    let session = HubRuntimeSession::load_from_paths(config_path.clone(), registry_path.clone())
        .expect("corrupt rebuildable history should not block Hub startup");
    assert!(session.config.recent_projects.is_empty());
    assert!(HubConfig::load(&config_path)
        .unwrap()
        .recent_projects
        .is_empty());
    assert_eq!(
        HubConfig::load(&config_path)
            .unwrap()
            .last_seen_shared_recent_revision,
        Some(0),
    );
    assert_eq!(fs::read(&registry_path).unwrap(), b"{ malformed");
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn save_settings_action_applies_typed_payload_and_refreshes_source_engine() {
    let temp = temp_test_dir("zircon-hub-tauri-save-settings-payload");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let source_path = temp.join("ZirconEngine");
    let build_output = temp.join("build-output");
    let device_install = temp.join("device-install");
    create_valid_source_checkout(&source_path);
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .expect("Tauri runtime session should load");

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "save-settings".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "pythonPath": "py",
                    "cargoPath": "cargo",
                    "rustupPath": "rustup",
                    "defaultProjectDir": temp.join("projects").to_string_lossy(),
                    "defaultSourceDir": source_path.to_string_lossy(),
                    "defaultBuildOutputDir": build_output.to_string_lossy(),
                    "defaultDeviceInstallDir": device_install.to_string_lossy(),
                    "buildProfile": "release",
                    "jobs": 3,
                    "language": "English"
                }
            })),
        })
        .expect("save-settings should accept typed settings payload");

    assert_eq!(session.config.settings.build_profile, BuildProfile::Release);
    assert_eq!(session.config.settings.jobs, 3);
    assert_eq!(session.config.settings.language, HubLanguage::English);
    assert_eq!(session.config.settings.default_source_dir, source_path);
    assert_eq!(
        session.config.settings.default_build_output_dir,
        build_output
    );
    assert_eq!(
        session.config.settings.default_device_install_dir,
        device_install
    );
    let expected_engine_id = source_engine_id(&source_path);
    assert_eq!(
        session.config.active_engine_id.as_deref(),
        Some(expected_engine_id.as_str())
    );
    let active_engine = session
        .config
        .engines
        .iter()
        .find(|engine| engine.id == expected_engine_id)
        .expect("payload Source Engine should be registered");
    assert_eq!(active_engine.source_dir, source_path);
    assert_eq!(active_engine.output_dir, build_output);
    assert_eq!(view_model.settings.language, "English");
    assert_eq!(view_model.task_summary.label, "Settings saved");

    let saved = HubConfig::load(&config_path).unwrap();
    assert_eq!(saved.settings.build_profile, BuildProfile::Release);
    assert_eq!(saved.settings.language, HubLanguage::English);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn save_settings_refreshes_source_scoped_catalogs_in_returned_view_model() {
    let temp = temp_test_dir("zircon-hub-tauri-save-settings-catalogs");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let source_path = temp.join("ZirconEngine");
    let build_output = temp.join("build-output");
    let device_install = temp.join("device-install");
    let asset_path = source_path
        .join("zircon_editor")
        .join("assets")
        .join("icons")
        .join("source-settings-tool.svg");
    let plugin_manifest_path = source_path
        .join("zircon_plugins")
        .join("source_settings_tools")
        .join("plugin.toml");
    let learn_path = source_path
        .join("docs")
        .join("settings")
        .join("source-settings-refresh.md");
    create_valid_source_checkout(&source_path);
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    fs::write(&asset_path, "<svg></svg>").unwrap();
    fs::create_dir_all(plugin_manifest_path.parent().unwrap()).unwrap();
    fs::write(
        &plugin_manifest_path,
        r#"id = "source_settings_tools"
display_name = "Source Settings Tools"
description = "Source plugin loaded after settings save."
category = "editor"
maturity = "stable"
supported_targets = ["editor_host"]

[[modules]]
name = "source.settings"
kind = "editor"
"#,
    )
    .unwrap();
    fs::create_dir_all(learn_path.parent().unwrap()).unwrap();
    fs::write(
        &learn_path,
        "# Source Settings Refresh\n\nSource Engine docs loaded after settings save.\n",
    )
    .unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .expect("Tauri runtime session should load");

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "save-settings".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "pythonPath": "py",
                    "cargoPath": "cargo",
                    "rustupPath": "rustup",
                    "defaultProjectDir": temp.join("projects").to_string_lossy(),
                    "defaultSourceDir": source_path.to_string_lossy(),
                    "defaultBuildOutputDir": build_output.to_string_lossy(),
                    "defaultDeviceInstallDir": device_install.to_string_lossy(),
                    "buildProfile": "debug",
                    "jobs": 2,
                    "language": "English"
                }
            })),
        })
        .expect("save-settings should refresh source-scoped catalogs");

    let expected_engine_id = source_engine_id(&source_path);
    assert_eq!(
        view_model.active_source_engine_id.as_deref(),
        Some(expected_engine_id.as_str())
    );
    let asset_debug = view_model
        .assets
        .iter()
        .map(|asset| format!("{}:{}", asset.name, asset.source_key))
        .collect::<Vec<_>>();
    assert!(
        view_model.assets.iter().any(|asset| {
            asset.name == "source-settings-tool.svg" && asset.source_key == "engine"
        }),
        "assets should include refreshed Source Engine asset, got {asset_debug:?}"
    );
    let plugin_debug = view_model
        .plugins
        .iter()
        .map(|plugin| format!("{}:{}", plugin.id, plugin.scope_key))
        .collect::<Vec<_>>();
    assert!(
        view_model.plugins.iter().any(|plugin| {
            plugin.id == "source_settings_tools"
                && plugin.scope_key == "engine"
                && plugin.editor_scoped
        }),
        "plugins should include refreshed Source Engine plugin, got {plugin_debug:?}"
    );
    let learn_debug = view_model
        .learn_resources
        .iter()
        .map(|resource| format!("{}:{}", resource.title, resource.source_key))
        .collect::<Vec<_>>();
    assert!(
        view_model.learn_resources.iter().any(|resource| {
            resource.title == "Source Settings Refresh" && resource.source_key == "engine"
        }),
        "learn resources should include refreshed Source Engine doc, got {learn_debug:?}"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn apply_action_records_payload_validation_failure_as_recoverable_status() {
    let temp = temp_test_dir("zircon-hub-tauri-payload-validation-status");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.language = HubLanguage::Chinese;
    config.save(&config_path).unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .expect("Tauri runtime session should load");

    let model = session
        .apply_action(HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": "projects/Game",
                "template": "renderable-empty"
            })),
        })
        .expect("payload validation failures should return refreshed Hub state");

    assert_eq!(model.task_summary.label, "操作失败");
    assert_eq!(
        model.task_summary.detail,
        "项目位置必须是绝对路径：projects/Game"
    );
    assert_eq!(
        model.task_summary.recovery.as_deref(),
        Some("检查操作 payload 后从 Hub 重试")
    );
    assert_eq!(model.task_summary.tone, "error");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn persist_failure_sets_recoverable_status_and_recovers_after_retry() {
    let temp = temp_test_dir("zircon-hub-tauri-persist-failure");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    HubConfig::default().save(&config_path).unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .expect("Tauri runtime session should load");
    let blocked_parent = temp.join("blocked-parent");
    fs::write(&blocked_parent, "not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");

    let error = session
        .apply_action(HubActionRequest {
            action_id: "show-page".to_string(),
            target_id: Some("settings".to_string()),
            payload: None,
        })
        .expect_err("blocked config parent should fail persist");

    assert!(error.to_string().contains("I/O error"));
    assert_eq!(session.task_status.label, "Save Hub state failed");
    assert_eq!(
        session
            .task_status
            .recovery
            .as_ref()
            .map(|message| message.render(HubLanguage::English))
            .as_deref(),
        Some("Check the Hub config path and retry the action")
    );

    session.config_path = config_path.clone();
    let model = session
        .apply_action(HubActionRequest {
            action_id: "show-page".to_string(),
            target_id: Some("projects".to_string()),
            payload: None,
        })
        .expect("restored config path should persist again");

    assert_eq!(model.active_page, "projects");
    assert_eq!(
        HubConfig::load(&config_path).unwrap().runtime.selected_page,
        HubPage::Projects
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn project_view_action_status_localizes_in_chinese_view_model() {
    let temp = temp_test_dir("zircon-hub-tauri-project-view-localized");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.language = HubLanguage::Chinese;
    config.save(&config_path).unwrap();
    let mut session =
        HubRuntimeSession::load_from_paths(config_path.clone(), shared_recent_projects_path)
            .expect("Tauri runtime session should load");

    let filter_model = session
        .apply_action(HubActionRequest {
            action_id: "set-project-filter".to_string(),
            target_id: Some("missing".to_string()),
            payload: None,
        })
        .expect("project filter action should return refreshed state");
    assert_eq!(filter_model.task_summary.label, "项目已筛选");
    assert_eq!(filter_model.task_summary.detail, "显示缺失项目");

    let sort_model = session
        .apply_action(HubActionRequest {
            action_id: "set-project-sort".to_string(),
            target_id: Some("name".to_string()),
            payload: None,
        })
        .expect("project sort action should return refreshed state");
    assert_eq!(sort_model.task_summary.label, "项目已排序");
    assert_eq!(sort_model.task_summary.detail, "按名称排序");

    let all_model = session
        .apply_action(HubActionRequest {
            action_id: "view-all-projects".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("view-all-projects action should return refreshed state");
    assert_eq!(all_model.task_summary.label, "全部项目");
    assert_eq!(all_model.task_summary.detail, "显示全部最近项目");

    fs::remove_dir_all(temp).unwrap();
}
