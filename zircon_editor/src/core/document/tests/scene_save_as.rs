use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, Scene};

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId, HistorySaveMarkOutcome,
};
use crate::core::editing::selection::SceneSelection;
use crate::core::editor_message::DocumentMessage;
use crate::core::project::{
    NewProjectDraft, ProjectAuthority, ProjectAuthorityError, ProjectTemplateId,
    SceneCreateRequest, SceneOpenRequest,
};
use crate::core::recovery::DocumentJournalCoordinator;
use crate::ui::workbench::project::EditorProjectDocument;

use super::{
    ActiveSceneDocumentIdentity, DocumentLifecycleAuthority, SceneAssetCatalog,
    SceneDocumentLifecycleError, SceneDocumentRoute, SceneDocumentRouteError, SceneSaveOutcome,
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    project: ProjectManager,
    lifecycle: DocumentLifecycleAuthority,
    journal: DocumentJournalCoordinator,
    world: Scene,
    location: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let executable = std::env::current_exe().unwrap();
        let directory = ProjectPaths::resolve_existing(executable.parent().unwrap()).unwrap();
        let location = directory
            .operation_path()
            .join("zircon-mvp-fixtures")
            .join(format!(
                "scene-save-as-{label}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&location).unwrap();
        let created = ProjectAuthority
            .create_project(
                &NewProjectDraft {
                    project_name: "Save As".to_string(),
                    location: location.to_string_lossy().into_owned(),
                    template: ProjectTemplateId::RenderableEmpty,
                },
                &crate::tests::support::test_project_creation_provenance(),
            )
            .unwrap();
        let mut project = ProjectAuthority
            .open_resolved_project(created.identity())
            .unwrap()
            .into_project();
        project.scan_and_import().unwrap();
        let document = ProjectAuthority
            .open_scene(&project, SceneOpenRequest::new(main_uri()))
            .unwrap();
        let lifecycle = DocumentLifecycleAuthority::default();
        let root = project.paths().root();
        let journal = DocumentJournalCoordinator::new(root);
        let session = lifecycle.begin_project_session(root).session;
        lifecycle
            .activate_scene_with_binding(
                session,
                root,
                &main_uri().to_string(),
                Some(document.source_revision().clone()),
                |id| journal.bind_project_document(id, document.source_path()),
            )
            .unwrap();
        Self {
            project,
            lifecycle,
            journal,
            world: document.into_world(),
            location,
        }
    }

    fn identity(&self) -> ActiveSceneDocumentIdentity {
        self.lifecycle
            .active_scene_identity(self.project.paths().root())
            .unwrap()
    }

    fn route(&self) -> SceneDocumentRoute<'_> {
        SceneDocumentRoute::new(
            &self.project,
            &self.lifecycle,
            &self.journal,
            self.lifecycle
                .issue_scene_picker_ticket(self.project.paths().root())
                .unwrap(),
        )
    }

    fn source(&self, file: &str) -> PathBuf {
        self.project.paths().root().join("assets/scenes").join(file)
    }

    fn assert_no_staging(&self) {
        assert!(fs::read_dir(self.source("")).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".zircon-scene-staging-")
        }));
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.location);
    }
}

struct Catalog {
    project: RefCell<ProjectManager>,
    reject: bool,
}

impl Catalog {
    fn new(fixture: &Fixture) -> Self {
        Self {
            project: RefCell::new(fixture.project.clone()),
            reject: false,
        }
    }
}

impl SceneAssetCatalog for Catalog {
    fn import_scene(&self, uri: &AssetUri) -> Result<(), ProjectAuthorityError> {
        if self.reject {
            return Err(ProjectAuthorityError::SceneTarget {
                uri: uri.to_string(),
                reason: "Save As catalog failure fixture",
            });
        }
        self.project
            .borrow_mut()
            .scan_and_import()
            .map(|_| ())
            .map_err(|source| ProjectAuthorityError::SceneCatalog { source })
    }

    fn remove_scene(&self, _uri: &AssetUri) -> Result<(), ProjectAuthorityError> {
        self.project
            .borrow_mut()
            .scan_and_import()
            .map(|_| ())
            .map_err(|source| ProjectAuthorityError::SceneCatalog { source })
    }
}

fn main_uri() -> AssetUri {
    AssetUri::parse("res://scenes/main.scene.toml").unwrap()
}

fn request(uri: &str) -> SceneCreateRequest {
    SceneCreateRequest::new(AssetUri::parse(uri).unwrap())
}

#[test]
fn save_as_persists_live_edits_preserves_undo_and_routes_later_save_to_the_new_scene() {
    let fixture = Fixture::new("roundtrip");
    let original_identity = fixture.identity();
    let original_source = fs::read(fixture.source("main.scene.toml")).unwrap();
    let original_journal = fixture
        .journal
        .journal_path(original_identity.document())
        .unwrap();
    let cube = fixture
        .world
        .nodes()
        .iter()
        .find(|node| node.name == "Cube")
        .unwrap()
        .id;
    let level = DefaultLevelManager::default()
        .create_level(fixture.world.clone(), LevelMetadata::default());
    let mut context = CoreEditContext::default();
    context
        .bind_scene(level.clone(), SceneSelection::new(vec![cube], Some(cube)))
        .unwrap();
    let transactions = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(original_identity.document());
    let command = level.with_world(|scene| {
        EditorCommand::rename_node(scene, cube, "Saved As Cube".to_string())
            .unwrap()
            .unwrap()
    });
    let mut edit = transactions
        .begin("Rename before Save As", history)
        .unwrap();
    edit.push(command).unwrap();
    edit.commit().unwrap();
    assert!(transactions.is_dirty(history).unwrap());
    let token = transactions.capture_save_token(history).unwrap();
    let catalog = Catalog::new(&fixture);
    let destination = request("res://scenes/saved-as.scene.toml");
    let identity = fixture
        .route()
        .save_as(destination.clone(), &level.snapshot(), &catalog)
        .unwrap();

    assert_eq!(identity.document(), original_identity.document());
    assert!(identity.activation_revision() > original_identity.activation_revision());
    assert_eq!(identity.scene_uri(), destination.scene_uri().to_string());
    assert_eq!(
        transactions
            .with_context::<CoreEditContext, _>(|context| context.scene_selection().unwrap())
            .unwrap()
            .unwrap(),
        SceneSelection::new(vec![cube], Some(cube))
    );
    assert_ne!(
        fixture.journal.journal_path(identity.document()).unwrap(),
        original_journal
    );
    assert_eq!(
        transactions
            .mark_saved_if_unchanged(history, token)
            .unwrap(),
        HistorySaveMarkOutcome::Marked
    );
    assert!(!transactions.is_dirty(history).unwrap());
    assert_eq!(transactions.history_status(history).unwrap().len, 1);
    let reopened = ProjectAuthority
        .open_scene(
            &catalog.project.borrow(),
            SceneOpenRequest::new(destination.scene_uri().clone()),
        )
        .unwrap();
    assert_eq!(Some(reopened.source_revision()), identity.source_revision());
    assert_eq!(
        reopened.world().find_node(cube).unwrap().name,
        "Saved As Cube"
    );
    assert!(transactions.undo(history).unwrap());
    assert!(transactions.is_dirty(history).unwrap());
    assert_eq!(level.snapshot().find_node(cube).unwrap().name, "Cube");
    assert!(transactions.redo(history).unwrap());
    assert!(!transactions.is_dirty(history).unwrap());

    // The ordinary Save route now selects the new URI and exact publication revision.
    assert!(transactions.undo(history).unwrap());
    let (outcome, saved) = fixture
        .lifecycle
        .save_active_scene(fixture.project.paths().root(), None, |active, expected| {
            assert_eq!(active.scene_uri(), destination.scene_uri().to_string());
            let receipt = EditorProjectDocument::save_scene_to_project(
                &catalog.project.borrow(),
                destination.scene_uri(),
                &level.snapshot(),
                expected,
                None,
            )?;
            Ok::<_, zircon_runtime::scene::world::SceneProjectError>(SceneSaveOutcome::Persisted {
                receipt,
            })
        })
        .unwrap();
    assert!(outcome.is_durable());
    assert_eq!(
        saved,
        Some(DocumentMessage::Saved {
            doc: identity.document()
        })
    );
    let later = ProjectAuthority
        .open_scene(
            &catalog.project.borrow(),
            SceneOpenRequest::new(destination.scene_uri().clone()),
        )
        .unwrap();
    assert_eq!(later.world().find_node(cube).unwrap().name, "Cube");
    assert_eq!(
        fs::read(fixture.source("main.scene.toml")).unwrap(),
        original_source
    );
    assert_eq!(fixture.project.manifest().default_scene, main_uri());
    fixture.assert_no_staging();
}

#[test]
fn save_as_refuses_existing_destinations_and_preserves_identity_and_bytes() {
    let fixture = Fixture::new("existing");
    let catalog = Catalog::new(&fixture);
    let identity = fixture.identity();
    let journal = fixture.journal.journal_path(identity.document()).unwrap();
    let original = fs::read(fixture.source("main.scene.toml")).unwrap();
    fs::write(
        fixture.source("existing.scene.toml"),
        b"caller-owned existing scene",
    )
    .unwrap();
    for uri in [
        "res://scenes/existing.scene.toml",
        "res://scenes/main.scene.toml",
    ] {
        let error = fixture
            .route()
            .save_as(request(uri), &fixture.world, &catalog)
            .unwrap_err();
        assert!(
            matches!(
                error,
                SceneDocumentRouteError::Project(ProjectAuthorityError::SceneAlreadyExists { .. })
            ),
            "{error}"
        );
        assert_eq!(fixture.identity(), identity);
        assert_eq!(
            fixture.journal.journal_path(identity.document()).unwrap(),
            journal
        );
    }
    assert_eq!(
        fs::read(fixture.source("existing.scene.toml")).unwrap(),
        b"caller-owned existing scene"
    );
    assert_eq!(
        fs::read(fixture.source("main.scene.toml")).unwrap(),
        original
    );
    fixture.assert_no_staging();
}

#[cfg(target_os = "linux")]
#[test]
fn save_as_refuses_symlink_components_and_dangling_destinations() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new("symlinks");
    let catalog = Catalog::new(&fixture);
    let identity = fixture.identity();
    let outside = fixture.location.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("foreign.scene.toml"), b"foreign source").unwrap();
    symlink(
        &outside,
        fixture.project.paths().root().join("assets/linked"),
    )
    .unwrap();
    symlink(
        outside.join("foreign.scene.toml"),
        fixture.source("linked.scene.toml"),
    )
    .unwrap();
    symlink(
        outside.join("missing.scene.toml"),
        fixture.source("dangling.scene.toml"),
    )
    .unwrap();
    for uri in [
        "res://linked/new.scene.toml",
        "res://scenes/linked.scene.toml",
        "res://scenes/dangling.scene.toml",
    ] {
        assert!(fixture
            .route()
            .save_as(request(uri), &fixture.world, &catalog)
            .is_err());
        assert_eq!(fixture.identity(), identity);
    }
    assert_eq!(
        fs::read(outside.join("foreign.scene.toml")).unwrap(),
        b"foreign source"
    );
    assert!(!outside.join("new.scene.toml").exists());
    assert!(!outside.join("missing.scene.toml").exists());
    fixture.assert_no_staging();
}

#[test]
fn save_as_missing_parent_and_catalog_failure_leave_the_old_scene_active() {
    let fixture = Fixture::new("rollback");
    let mut catalog = Catalog::new(&fixture);
    let identity = fixture.identity();
    let journal = fixture.journal.journal_path(identity.document()).unwrap();
    assert!(fixture
        .route()
        .save_as(
            request("res://absent/new.scene.toml"),
            &fixture.world,
            &catalog
        )
        .is_err());
    assert!(!fixture
        .project
        .paths()
        .root()
        .join("assets/absent")
        .exists());
    catalog.reject = true;
    assert!(fixture
        .route()
        .save_as(
            request("res://scenes/rejected.scene.toml"),
            &fixture.world,
            &catalog
        )
        .is_err());
    assert!(!fixture.source("rejected.scene.toml").exists());
    assert_eq!(fixture.identity(), identity);
    assert_eq!(
        fixture.journal.journal_path(identity.document()).unwrap(),
        journal
    );
    assert!(catalog
        .project
        .borrow()
        .registry()
        .values()
        .all(|entry| !entry
            .primary_locator()
            .to_string()
            .contains("rejected.scene.toml")));
    fixture.assert_no_staging();
}

#[test]
fn save_as_stale_scene_or_project_picker_never_publishes_a_destination() {
    let fixture = Fixture::new("stale");
    let catalog = Catalog::new(&fixture);
    for change_project in [false, true] {
        let route = fixture.route();
        let root = fixture.project.paths().root();
        if change_project {
            fixture.lifecycle.begin_project_session(root);
        } else {
            let session = fixture.lifecycle.project_session(root).unwrap();
            fixture
                .lifecycle
                .activate_scene(session, root, &main_uri().to_string())
                .unwrap();
        }
        let error = route
            .save_as(
                request("res://scenes/stale.scene.toml"),
                &fixture.world,
                &catalog,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            SceneDocumentRouteError::Lifecycle(
                SceneDocumentLifecycleError::StaleSaveAsScene
                    | SceneDocumentLifecycleError::StaleProjectSession { .. }
            )
        ));
        assert!(!fixture.source("stale.scene.toml").exists());
    }
    fixture.assert_no_staging();
}

#[test]
fn save_as_moves_retained_keys_without_aliasing_document_ids() {
    let fixture = Fixture::new("retained-key");
    let catalog = Catalog::new(&fixture);
    let root = fixture.project.paths().root();
    let session = fixture.lifecycle.project_session(root).unwrap();
    let target = "res://scenes/previous.scene.toml";
    let previous = fixture
        .lifecycle
        .activate_scene(session, root, target)
        .unwrap()
        .document;
    fixture
        .lifecycle
        .activate_scene(session, root, &main_uri().to_string())
        .unwrap();
    let current = fixture.identity().document();
    assert_ne!(previous, current);
    assert_eq!(
        fixture.lifecycle.retention_snapshot().scene_identity_count,
        2
    );
    let identity = fixture
        .route()
        .save_as(request(target), &fixture.world, &catalog)
        .unwrap();
    assert_eq!(identity.document(), current);
    assert_eq!(
        fixture.lifecycle.retention_snapshot().scene_identity_count,
        1
    );
    let reopened_old = fixture
        .lifecycle
        .activate_scene(session, root, &main_uri().to_string())
        .unwrap();
    assert_ne!(reopened_old.document, identity.document());
}
