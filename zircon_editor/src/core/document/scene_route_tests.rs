#![cfg(windows)]

use std::cell::Cell;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::AssetUri;

use crate::core::project::{
    NewProjectDraft, NewProjectTemplate, ProjectAuthority, ProjectAuthorityError,
    SceneCreateRequest, SceneOpenRequest,
};
use crate::core::recovery::DocumentJournalCoordinator;

use super::{
    AuthoringSceneInstaller, DocumentLifecycleAuthority, SceneAssetCatalog, SceneDocumentRoute,
    SceneDocumentRouteError, SceneDocumentRouteResult,
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

struct RecordingInstaller {
    installed_scene_count: usize,
}

struct FailingInstaller;

struct DirtySceneBlockingInstaller {
    install_count: usize,
}

struct RecordingCatalog {
    import_count: Cell<usize>,
    remove_count: Cell<usize>,
    reject_import: bool,
}

impl RecordingCatalog {
    fn accepting() -> Self {
        Self {
            import_count: Cell::new(0),
            remove_count: Cell::new(0),
            reject_import: false,
        }
    }

    fn rejecting_import() -> Self {
        Self {
            import_count: Cell::new(0),
            remove_count: Cell::new(0),
            reject_import: true,
        }
    }
}

impl SceneAssetCatalog for RecordingCatalog {
    fn import_scene(&self, scene_uri: &AssetUri) -> Result<(), ProjectAuthorityError> {
        self.import_count.set(self.import_count.get() + 1);
        if self.reject_import {
            return Err(ProjectAuthorityError::SceneTarget {
                uri: scene_uri.to_string(),
                reason: "test catalog rejected the created scene",
            });
        }
        Ok(())
    }

    fn remove_scene(&self, _scene_uri: &AssetUri) -> Result<(), ProjectAuthorityError> {
        self.remove_count.set(self.remove_count.get() + 1);
        Ok(())
    }
}

impl AuthoringSceneInstaller for FailingInstaller {
    type Error = &'static str;

    fn prepare_scene_transition(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn install_scene(
        &mut self,
        _document: &crate::core::project::ProjectSceneDocument,
    ) -> Result<(), Self::Error> {
        Err("authoring world rejected scene")
    }
}

impl AuthoringSceneInstaller for DirtySceneBlockingInstaller {
    type Error = &'static str;

    fn prepare_scene_transition(&mut self) -> Result<(), Self::Error> {
        Err("save or discard the current scene before opening another scene")
    }

    fn install_scene(
        &mut self,
        _document: &crate::core::project::ProjectSceneDocument,
    ) -> Result<(), Self::Error> {
        self.install_count += 1;
        Ok(())
    }
}

impl AuthoringSceneInstaller for RecordingInstaller {
    type Error = String;

    fn prepare_scene_transition(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn install_scene(
        &mut self,
        _document: &crate::core::project::ProjectSceneDocument,
    ) -> Result<(), Self::Error> {
        self.installed_scene_count += 1;
        Ok(())
    }
}

#[test]
fn dirty_scene_transition_admission_rejects_open_before_world_installation_or_lifecycle_change() {
    let (location, project) = project_fixture("route-dirty-transition");
    let root = project.paths().root().to_path_buf();
    let lifecycle = DocumentLifecycleAuthority::default();
    let session = lifecycle.begin_project_session(&root).session;
    let ticket = lifecycle.issue_scene_picker_ticket(&root).unwrap();
    let journal = DocumentJournalCoordinator::new(&root);
    let route = SceneDocumentRoute::new(&project, &lifecycle, &journal, ticket);
    let request = SceneOpenRequest::new(AssetUri::parse("res://scenes/main.scene.toml").unwrap());
    let mut installer = DirtySceneBlockingInstaller { install_count: 0 };

    let error = route.open(request, &mut installer).unwrap_err();
    assert!(matches!(
        error,
        SceneDocumentRouteError::Transition(
            "save or discard the current scene before opening another scene"
        )
    ));
    assert_eq!(installer.install_count, 0);
    assert!(
        lifecycle
            .active_scene_document(session, &root, "res://scenes/main.scene.toml")
            .unwrap()
            .is_none()
    );

    let catalog = RecordingCatalog::accepting();
    let created_uri = AssetUri::parse("res://scenes/new.scene.toml").unwrap();
    let create_error = route
        .create(
            SceneCreateRequest::new(created_uri),
            &mut installer,
            &catalog,
        )
        .unwrap_err();
    assert!(matches!(
        create_error,
        SceneDocumentRouteError::Transition(_)
    ));
    assert_eq!(catalog.import_count.get(), 0);
    assert!(!root.join("assets/scenes/new.scene.toml").exists());

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn scene_document_route_installs_once_then_reports_the_current_scene_as_already_active() {
    let (location, project) = project_fixture("route-open");
    let root = project.paths().root().to_path_buf();
    let lifecycle = DocumentLifecycleAuthority::default();
    let session = lifecycle.begin_project_session(&root).session;
    let ticket = lifecycle.issue_scene_picker_ticket(&root).unwrap();
    let journal = DocumentJournalCoordinator::new(&root);
    let route = SceneDocumentRoute::new(&project, &lifecycle, &journal, ticket);
    let request = SceneOpenRequest::new(AssetUri::parse("res://scenes/main.scene.toml").unwrap());
    let mut installer = RecordingInstaller {
        installed_scene_count: 0,
    };

    let activated = route.open(request.clone(), &mut installer).unwrap();
    let document = match activated {
        SceneDocumentRouteResult::Activated(activation) => {
            assert!(!activation.activation.already_active);
            activation.activation.document
        }
        result => panic!("expected an activated scene document, got {result:?}"),
    };
    assert_eq!(installer.installed_scene_count, 1);
    assert!(journal.journal_path(document).is_ok());

    let repeated = route.open(request, &mut installer).unwrap();
    assert!(matches!(
        repeated,
        SceneDocumentRouteResult::AlreadyActive { document: active } if active == document
    ));
    assert_eq!(installer.installed_scene_count, 1);

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn scene_route_reserves_lifecycle_before_installing_and_commits_afterward() {
    let source = include_str!("scene_route.rs");
    let install = source
        .split_once("fn install_and_commit")
        .expect("scene route owns one install-and-commit transition")
        .1;

    assert!(
        source.contains("prepare_scene_activation_while_routed")
            && install.contains("installer.install_scene(&document)")
            && install.contains("commit_scene_activation_while_routed(reservation)"),
        "scene route must reserve lifecycle identity before replacing the authoring world and only commit it after installation"
    );
}

#[test]
fn scene_document_route_rejects_a_picker_result_after_its_project_session_closes() {
    let (location, project) = project_fixture("route-stale");
    let root = project.paths().root().to_path_buf();
    let lifecycle = DocumentLifecycleAuthority::default();
    let _stale_session = lifecycle.begin_project_session(&root).session;
    let stale_ticket = lifecycle.issue_scene_picker_ticket(&root).unwrap();
    let _current_session = lifecycle.begin_project_session(&root).session;
    let journal = DocumentJournalCoordinator::new(&root);
    let route = SceneDocumentRoute::new(&project, &lifecycle, &journal, stale_ticket);
    let mut installer = RecordingInstaller {
        installed_scene_count: 0,
    };
    let catalog = RecordingCatalog::accepting();

    let error = route
        .open(
            SceneOpenRequest::new(AssetUri::parse("res://scenes/main.scene.toml").unwrap()),
            &mut installer,
        )
        .unwrap_err();
    assert!(matches!(error, SceneDocumentRouteError::Lifecycle(_)));
    assert_eq!(installer.installed_scene_count, 0);

    let create_error = route
        .create(
            crate::core::project::SceneCreateRequest::new(
                AssetUri::parse("res://scenes/stale.scene.toml").unwrap(),
            ),
            &mut installer,
            &catalog,
        )
        .unwrap_err();
    assert!(matches!(
        create_error,
        SceneDocumentRouteError::Lifecycle(_)
    ));
    assert!(!root.join("assets/scenes/stale.scene.toml").exists());

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn cancelled_or_conflicting_scene_creation_keeps_the_active_document_unchanged() {
    let (location, project) = project_fixture("route-create-failure");
    let root = project.paths().root().to_path_buf();
    let lifecycle = DocumentLifecycleAuthority::default();
    let session = lifecycle.begin_project_session(&root).session;
    let ticket = lifecycle.issue_scene_picker_ticket(&root).unwrap();
    let journal = DocumentJournalCoordinator::new(&root);
    let route = SceneDocumentRoute::new(&project, &lifecycle, &journal, ticket);
    let scene_uri = AssetUri::parse("res://scenes/new.scene.toml").unwrap();
    let mut installer = RecordingInstaller {
        installed_scene_count: 0,
    };
    let catalog = RecordingCatalog::accepting();

    // Cancelling a picker means the caller submits no request at all.
    assert!(
        lifecycle
            .active_scene_document(session, &root, &scene_uri.to_string())
            .unwrap()
            .is_none()
    );
    assert!(!root.join("assets/scenes/new.scene.toml").exists());

    let created = route
        .create(
            crate::core::project::SceneCreateRequest::new(scene_uri.clone()),
            &mut installer,
            &catalog,
        )
        .unwrap();
    let created_document = match created {
        SceneDocumentRouteResult::Activated(activation) => activation.activation.document,
        result => panic!("expected created scene activation, got {result:?}"),
    };
    let duplicate = route
        .create(
            crate::core::project::SceneCreateRequest::new(scene_uri),
            &mut installer,
            &catalog,
        )
        .unwrap_err();
    assert!(matches!(duplicate, SceneDocumentRouteError::Project(_)));
    assert_eq!(
        lifecycle
            .active_scene_document(session, &root, "res://scenes/new.scene.toml")
            .unwrap(),
        Some(created_document)
    );
    assert_eq!(installer.installed_scene_count, 1);
    assert_eq!(catalog.import_count.get(), 1);
    assert_eq!(catalog.remove_count.get(), 0);

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn failed_scene_installation_compensates_the_created_scene_asset() {
    let (location, project) = project_fixture("route-install-failure");
    let root = project.paths().root().to_path_buf();
    let lifecycle = DocumentLifecycleAuthority::default();
    let session = lifecycle.begin_project_session(&root).session;
    let ticket = lifecycle.issue_scene_picker_ticket(&root).unwrap();
    let journal = DocumentJournalCoordinator::new(&root);
    let route = SceneDocumentRoute::new(&project, &lifecycle, &journal, ticket);
    let scene_uri = AssetUri::parse("res://scenes/rejected.scene.toml").unwrap();
    let mut installer = FailingInstaller;
    let catalog = RecordingCatalog::accepting();

    let error = route
        .create(
            crate::core::project::SceneCreateRequest::new(scene_uri.clone()),
            &mut installer,
            &catalog,
        )
        .unwrap_err();

    assert!(matches!(error, SceneDocumentRouteError::Install(_)));
    assert_eq!(catalog.import_count.get(), 1);
    assert_eq!(catalog.remove_count.get(), 1);
    assert!(!root.join("assets/scenes/rejected.scene.toml").exists());
    assert!(
        lifecycle
            .active_scene_document(session, &root, &scene_uri.to_string())
            .unwrap()
            .is_none()
    );
    let staging_names = fs::read_dir(root.join("assets/scenes"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        staging_names
            .iter()
            .all(|name| !name.contains(".zircon-scene-staging-")),
        "rejected scene creation left staging files: {staging_names:?}"
    );

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn rejected_catalog_import_removes_the_source_before_any_authoring_installation() {
    let (location, project) = project_fixture("route-catalog-failure");
    let root = project.paths().root().to_path_buf();
    let lifecycle = DocumentLifecycleAuthority::default();
    let session = lifecycle.begin_project_session(&root).session;
    let ticket = lifecycle.issue_scene_picker_ticket(&root).unwrap();
    let journal = DocumentJournalCoordinator::new(&root);
    let route = SceneDocumentRoute::new(&project, &lifecycle, &journal, ticket);
    let scene_uri = AssetUri::parse("res://scenes/catalog-rejected.scene.toml").unwrap();
    let mut installer = RecordingInstaller {
        installed_scene_count: 0,
    };
    let catalog = RecordingCatalog::rejecting_import();

    let error = route
        .create(
            crate::core::project::SceneCreateRequest::new(scene_uri.clone()),
            &mut installer,
            &catalog,
        )
        .unwrap_err();

    assert!(matches!(
        error,
        SceneDocumentRouteError::Project(ProjectAuthorityError::SceneTarget { .. })
    ));
    assert_eq!(catalog.import_count.get(), 1);
    assert_eq!(catalog.remove_count.get(), 1);
    assert_eq!(installer.installed_scene_count, 0);
    assert!(
        !root
            .join("assets/scenes/catalog-rejected.scene.toml")
            .exists()
    );
    assert!(
        lifecycle
            .active_scene_document(session, &root, &scene_uri.to_string())
            .unwrap()
            .is_none()
    );

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

fn project_fixture(
    label: &str,
) -> (
    std::path::PathBuf,
    zircon_runtime::asset::project::ProjectManager,
) {
    let location = std::env::temp_dir().join(format!(
        "zircon-editor-scene-document-{label}-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&location);
    fs::create_dir_all(&location).unwrap();
    let created = ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: "Scene Route".to_string(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    let mut project = created.into_project();
    project.scan_and_import().unwrap();
    (location, project)
}
