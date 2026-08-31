use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;

use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetManager, AssetUri};
use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime::plugin::native::discovery::load_discovered_native_editor_plugins;

use crate::core::document::{
    ActiveSceneDocumentIdentity, AuthoringSceneInstaller, SceneAssetCatalog,
    SceneDocumentActivationBindingError, SceneDocumentRoute, SceneDocumentRouteError,
    SceneDocumentRouteResult, ScenePickerTicket,
};
use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::editor_message::{
    DocumentId, DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic,
    SharedEditorMessageBus,
};
use crate::core::project::{SceneCreateRequest, SceneOpenRequest};
use crate::core::recovery::{
    DocumentJournalCoordinator, DocumentJournalCoordinatorError, ProjectSessionEffect,
};
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::EditorStartupSessionDocument;

use super::editor_asset_manager::EditorAssetManager;
use super::editor_error::EditorError;
use super::editor_manager::EditorManager;
use super::project_session_close::{
    ProjectCloseCommit, ProjectCloseError, ProjectCloseOperation, ProjectCloseReceipt,
};

impl EditorManager {
    pub fn project_reference_diagnostics(
        &self,
    ) -> Result<zircon_runtime::asset::project::ProjectReferenceDiagnosticsSnapshot, EditorError>
    {
        Ok(self
            .host
            .current_project_snapshot()?
            .map(|project| project.reference_diagnostics())
            .unwrap_or_default())
    }

    pub(crate) fn active_scene_identity(
        &self,
        project_root: &Path,
    ) -> Option<ActiveSceneDocumentIdentity> {
        self.document_lifecycle.active_scene_identity(project_root)
    }

    pub(crate) fn active_scene_identity_for_session(&self) -> Option<ActiveSceneDocumentIdentity> {
        self.document_lifecycle.active_scene_identity_for_session()
    }

    pub fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        let intent = self.local_open_project_intent(path.as_ref())?;
        let preflight = self.preflight_existing_project_launch(&intent, path.as_ref())?;
        let admission = self.session_admission_request(&intent)?;
        self.open_project_document_with_admission(preflight, &admission)
    }

    pub(crate) fn begin_project_close(&self) -> Result<Option<ProjectCloseOperation>, EditorError> {
        let _transition = self.begin_project_session_transition()?;
        self.ensure_project_recovery_is_settled()?;
        self.begin_project_close_operation()
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(crate) fn commit_project_close(
        &self,
        operation: &ProjectCloseOperation,
    ) -> Result<ProjectCloseCommit, ProjectCloseError> {
        let _transition = self.begin_project_session_transition().map_err(|error| {
            self.require_project_close_recovery(
                operation,
                ProjectSessionEffect::ProjectPlugins,
                error.to_string(),
            )
        })?;

        self.prepare_project_close_effect(operation, ProjectSessionEffect::ProjectPlugins)?;
        self.clear_project_plugin_status();
        let plugin_receipt = self
            .plugin_manager()
            .clear_project_registration_reports()
            .map_err(|error| {
                self.require_project_close_recovery(
                    operation,
                    ProjectSessionEffect::ProjectPlugins,
                    format!(
                        "project-native editor registrations cannot be cleared during close: {error}"
                    ),
                )
            })?;
        if !plugin_receipt.is_terminal() {
            return Err(self.require_project_close_recovery(
                operation,
                ProjectSessionEffect::ProjectPlugins,
                format!(
                    "project-native editor registrations remain after manager generation {} / catalog generation {}: {:?}",
                    plugin_receipt.manager_generation(),
                    plugin_receipt.catalog_generation(),
                    plugin_receipt.remaining_project_package_ids(),
                ),
            ));
        }
        self.commit_project_close_effect(operation, ProjectSessionEffect::ProjectPlugins)?;

        self.prepare_project_close_effect(operation, ProjectSessionEffect::Runtime)?;
        let runtime_receipt =
            self.host
                .close_project(operation.project_root())
                .map_err(|error| {
                    self.require_project_close_recovery(
                        operation,
                        ProjectSessionEffect::Runtime,
                        format!("runtime project close failed: {error}"),
                    )
                })?;
        self.commit_project_close_effect(operation, ProjectSessionEffect::Runtime)?;
        let closed_root = runtime_receipt.into_closed_root();

        let terminal_root =
            project_close_terminal_root(closed_root.as_deref(), Some(operation.project_root()));
        self.prepare_project_close_effect(operation, ProjectSessionEffect::Documents)?;
        self.clear_document_journal();
        publish_committed_project_close(
            self.context().bus(),
            &self.document_lifecycle,
            terminal_root,
        );
        self.commit_project_close_effect(operation, ProjectSessionEffect::Documents)?;

        self.prepare_project_close_effect(operation, ProjectSessionEffect::Diagnostics)?;
        self.context().logs().disable_rolling_file();
        self.context().settings().clear_project_layer();
        let receipt =
            self.commit_project_close_effect(operation, ProjectSessionEffect::Diagnostics)?;
        Ok(ProjectCloseCommit::new(closed_root, receipt))
    }

    pub(crate) fn finalize_project_close(
        &self,
        operation: &ProjectCloseOperation,
    ) -> Result<ProjectCloseReceipt, ProjectCloseError> {
        let _transition = self.begin_project_session_transition().map_err(|error| {
            self.require_project_close_recovery(
                operation,
                ProjectSessionEffect::Session,
                error.to_string(),
            )
        })?;
        let receipt = self.finish_project_close_ledger(operation)?;
        self.release_project_close_guard(operation)?;
        let _ = self.cleanup_closed_project_session_ledger(operation);
        Ok(receipt)
    }

    pub(crate) fn save_active_scene(
        &self,
        path: impl AsRef<Path>,
        world: &zircon_runtime::scene::Scene,
    ) -> Result<(), EditorError> {
        let project_root = ProjectAuthority::default().resolve_existing_project_root(&path)?;
        let active_scene = self
            .document_lifecycle
            .active_scene_identity(&project_root)
            .ok_or_else(|| {
                EditorError::Project(
                    "cannot save without an active project scene document".to_string(),
                )
            })?;
        let scene_uri = AssetUri::parse(active_scene.scene_uri()).map_err(|error| {
            EditorError::Project(format!(
                "active scene document {} has an invalid source URI: {error}",
                active_scene.document().value()
            ))
        })?;
        self.host
            .save_active_scene(&project_root, &scene_uri, world)?;
        self.publish_document_messages(
            self.document_lifecycle
                .save_scene_identity_if_active(&active_scene),
        );
        Ok(())
    }

    /// Publishes the manifest-selected startup scene as the first document of an already-open
    /// project session. Later picker routes replace this identity through the same lifecycle.
    pub(crate) fn activate_startup_scene_document(
        &self,
        project_root: &Path,
        scene_uri: &AssetUri,
    ) -> Result<DocumentId, EditorError> {
        let project = self.host.current_project_snapshot()?.ok_or_else(|| {
            EditorError::Project(
                "cannot activate a startup scene without an active project generation".to_string(),
            )
        })?;
        if project.paths().root() != project_root {
            return Err(EditorError::Project(
                "startup scene belongs to a project generation that is no longer active"
                    .to_string(),
            ));
        }
        let session = self
            .document_lifecycle
            .project_session(project_root)
            .ok_or_else(|| {
                EditorError::Project(
                    "cannot activate a startup scene without an active project session".to_string(),
                )
            })?;
        let source_path = project.source_path_for_uri(scene_uri).map_err(|error| {
            EditorError::Project(format!(
                "cannot resolve startup scene journal source {scene_uri}: {error}"
            ))
        })?;
        let journal = self.document_journal()?;
        let activation = self
            .document_lifecycle
            .activate_scene_with_binding(
                session,
                project_root,
                &scene_uri.to_string(),
                |document| journal.bind_project_document(document, &source_path),
            )
            .map_err(|error| match error {
                SceneDocumentActivationBindingError::Lifecycle(error) => {
                    EditorError::Project(error.to_string())
                }
                SceneDocumentActivationBindingError::Binding(error) => {
                    EditorError::DocumentJournal { source: error }
                }
            })?;
        let document = activation.document;
        self.release_closed_document_journals(&activation.messages, journal.as_ref());
        self.publish_document_messages(activation.messages);
        Ok(document)
    }

    /// Submits a picker-selected project scene through the authority-owned document route.
    ///
    /// The caller supplies the host-only authoring-world installer. No scene fact is published
    /// until the installer and lifecycle transition have both committed.
    pub fn open_scene_document<Installer>(
        &self,
        ticket: ScenePickerTicket,
        request: SceneOpenRequest,
        installer: &mut Installer,
    ) -> Result<SceneDocumentRouteResult, EditorError>
    where
        Installer: AuthoringSceneInstaller,
        Installer::Error: Display,
    {
        self.route_project_scene::<Installer, _>(ticket, |route| route.open(request, installer))
    }

    /// Creates and opens a picker-confirmed project scene through the same document route.
    pub fn create_scene_document<Installer>(
        &self,
        ticket: ScenePickerTicket,
        request: SceneCreateRequest,
        installer: &mut Installer,
    ) -> Result<SceneDocumentRouteResult, EditorError>
    where
        Installer: AuthoringSceneInstaller,
        Installer::Error: Display,
    {
        let catalog = RuntimeSceneAssetCatalog::new(
            self.host.asset_manager()?,
            self.host.editor_asset_manager()?,
        );
        self.route_project_scene::<Installer, _>(ticket, |route| {
            route.create(request, installer, &catalog)
        })
    }

    /// Issues the project-session capability that a scene picker must preserve until submit.
    pub fn scene_picker_ticket(&self) -> Result<ScenePickerTicket, EditorError> {
        let project = self.host.current_project_snapshot()?.ok_or_else(|| {
            EditorError::Project("cannot begin scene picking without an active project".to_string())
        })?;
        self.document_lifecycle
            .issue_scene_picker_ticket(project.paths().root())
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(crate) fn prepare_authoring_world(
        &self,
        scene: zircon_runtime::scene::Scene,
    ) -> Result<AuthoringWorldSeed, EditorError> {
        self.host.prepare_authoring_world(scene)
    }

    pub(super) fn configure_project_diagnostics(
        &self,
        project_root: &Path,
    ) -> Result<(), EditorError> {
        self.context()
            .logs()
            .configure_workspace_diagnostics(project_root)
            .map_err(|error| {
                EditorError::Project(project_diagnostics_configuration_message(
                    project_root,
                    error,
                ))
            })
    }

    pub(super) fn apply_project_plugin_manifest(
        &self,
        project_root: &Path,
        manifest: &ProjectManifest,
        approved_project_plugins: &ProjectPluginManifest,
        allows_native_extensions: bool,
    ) -> Result<(), EditorError> {
        let mut approved_manifest = manifest.clone();
        approved_manifest.plugins = approved_project_plugins.clone();
        let (completed, native_reports) = if allows_native_extensions {
            let native_report =
                load_discovered_native_editor_plugins(self.plugin_directory(project_root));
            let completed = self.complete_project_plugin_manifest_with_native_report(
                &approved_manifest,
                &native_report,
            );
            let native_reports = self
                .selected_native_editor_plugin_registration_reports_from_load_report(
                    &native_report,
                    &completed.plugins,
                );
            (completed, native_reports)
        } else {
            (
                self.complete_project_plugin_manifest(&approved_manifest),
                Vec::new(),
            )
        };
        self.plugin_manager()
            .publish_project_registration_reports(native_reports)
            .map_err(|error| {
                EditorError::Project(format!(
                    "project-native editor registrations cannot be published: {error}"
                ))
            })?;
        self.plugin_manager()
            .apply_project_manifest(&completed.plugins)
            .map_err(|error| {
                EditorError::Project(format!(
                    "project plugin manifest cannot be applied to the editor plugin manager: {error}"
                ))
            })?;
        self.publish_project_plugin_status(self.plugin_status_report(&completed));
        Ok(())
    }

    pub(super) fn publish_document_messages(
        &self,
        messages: impl IntoIterator<Item = DocumentMessage>,
    ) {
        publish_document_messages(self.context().bus(), messages);
    }

    pub(super) fn initialize_document_journal(
        &self,
        project_root: &Path,
    ) -> Result<(), EditorError> {
        let mut slot = self
            .document_journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match slot.as_ref() {
            None => {
                *slot = Some(Arc::new(DocumentJournalCoordinator::new(project_root)));
                Ok(())
            }
            Some(existing) if existing.project_root() == project_root => Ok(()),
            Some(existing) => Err(DocumentJournalCoordinatorError::ProjectRootConflict {
                existing_root: existing.project_root().to_path_buf(),
                requested_root: project_root.to_path_buf(),
            }
            .into()),
        }
    }

    pub(super) fn clear_document_journal(&self) {
        self.document_journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
    }

    fn document_journal(&self) -> Result<Arc<DocumentJournalCoordinator>, EditorError> {
        self.document_journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .ok_or_else(|| {
                EditorError::Project(
                    "scene document routing requires an active project journal authority"
                        .to_string(),
                )
            })
    }

    fn release_closed_document_journals(
        &self,
        messages: &[DocumentMessage],
        journal: &DocumentJournalCoordinator,
    ) {
        for message in messages {
            if let DocumentMessage::Closed { doc } = message {
                journal.unbind_document(*doc);
            }
        }
    }

    fn route_project_scene<Installer, Route>(
        &self,
        ticket: ScenePickerTicket,
        route: Route,
    ) -> Result<SceneDocumentRouteResult, EditorError>
    where
        Installer: AuthoringSceneInstaller,
        Installer::Error: Display,
        Route:
            FnOnce(
                SceneDocumentRoute<'_>,
            )
                -> Result<SceneDocumentRouteResult, SceneDocumentRouteError<Installer::Error>>,
    {
        let project = self.host.current_project_snapshot()?.ok_or_else(|| {
            EditorError::Project(
                "cannot route a scene request without an active project".to_string(),
            )
        })?;
        if project.paths().root() != ticket.project_root() {
            return Err(EditorError::Project(
                "scene picker result belongs to a project session that is no longer active"
                    .to_string(),
            ));
        }
        let journal = self.document_journal()?;
        let result = route(SceneDocumentRoute::new(
            &project,
            &self.document_lifecycle,
            journal.as_ref(),
            ticket,
        ))
        .map_err(|error| EditorError::Project(error.to_string()))?;
        if let SceneDocumentRouteResult::Activated(activation) = &result {
            self.publish_document_messages(activation.activation.messages.clone());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod recovery_close_contract_tests {
    #[test]
    fn close_refuses_to_clear_a_session_while_recovery_work_is_active() {
        let source = include_str!("editor_manager_project.rs");
        let close = source
            .find("pub(crate) fn begin_project_close")
            .expect("project close owner should exist");
        let recovery_gate = source[close..]
            .find("self.ensure_project_recovery_is_settled()?;")
            .map(|offset| close + offset)
            .expect("project close should check recovery state");
        let begin_close = source[close..]
            .find("self.begin_project_close_operation()")
            .map(|offset| close + offset)
            .expect("project close should begin the durable close phase");

        assert!(recovery_gate < begin_close);
    }
}

fn project_diagnostics_configuration_message(project_root: &Path, error: impl Display) -> String {
    format!(
        "editor diagnostics cannot be configured for `{}`: {error}",
        ProjectPaths::display_path(project_root).display()
    )
}

struct RuntimeSceneAssetCatalog {
    asset_manager: Arc<dyn AssetManager>,
    editor_asset_manager: Arc<dyn EditorAssetManager>,
}

impl RuntimeSceneAssetCatalog {
    fn new(
        asset_manager: Arc<dyn AssetManager>,
        editor_asset_manager: Arc<dyn EditorAssetManager>,
    ) -> Self {
        Self {
            asset_manager,
            editor_asset_manager,
        }
    }
}

impl SceneAssetCatalog for RuntimeSceneAssetCatalog {
    fn import_scene(
        &self,
        scene_uri: &AssetUri,
    ) -> Result<(), crate::core::project::ProjectAuthorityError> {
        let status = self
            .asset_manager
            .import_asset(&scene_uri.to_string())
            .map_err(
                |source| crate::core::project::ProjectAuthorityError::SceneCatalogRuntime {
                    operation: "importing created scene",
                    source,
                },
            )?;
        if status.is_none() {
            return Err(crate::core::project::ProjectAuthorityError::SceneTarget {
                uri: scene_uri.to_string(),
                reason: "runtime asset catalog has no active project for the created scene",
            });
        }
        self.editor_asset_manager
            .refresh_from_runtime_project()
            .map_err(
                |source| crate::core::project::ProjectAuthorityError::SceneCatalogRuntime {
                    operation: "refreshing editor asset catalog for created scene",
                    source,
                },
            )?;
        Ok(())
    }

    fn remove_scene(
        &self,
        scene_uri: &AssetUri,
    ) -> Result<(), crate::core::project::ProjectAuthorityError> {
        self.asset_manager.reimport_all().map_err(|source| {
            crate::core::project::ProjectAuthorityError::SceneCatalogRuntime {
                operation: "removing rolled-back scene",
                source,
            }
        })?;
        if self
            .asset_manager
            .asset_status(&scene_uri.to_string())
            .is_some()
        {
            return Err(crate::core::project::ProjectAuthorityError::SceneTarget {
                uri: scene_uri.to_string(),
                reason: "runtime asset catalog retained a rolled-back scene",
            });
        }
        self.editor_asset_manager
            .refresh_from_runtime_project()
            .map_err(
                |source| crate::core::project::ProjectAuthorityError::SceneCatalogRuntime {
                    operation: "refreshing editor asset catalog after scene rollback",
                    source,
                },
            )?;
        Ok(())
    }
}

fn publish_document_messages(
    bus: &SharedEditorMessageBus,
    messages: impl IntoIterator<Item = DocumentMessage>,
) {
    let topic = EditorTopic::document();
    for message in messages {
        bus.publish(
            topic.clone(),
            EditorMessage::new(EditorMessagePayload::Document(message)),
        );
    }
}

fn publish_committed_project_close(
    bus: &SharedEditorMessageBus,
    lifecycle: &crate::core::document::DocumentLifecycleAuthority,
    closed_root: Option<&Path>,
) {
    let Some(closed_root) = closed_root else {
        return;
    };
    let session_messages = lifecycle.end_project_session(closed_root);
    if session_messages.is_empty() {
        publish_document_messages(bus, lifecycle.close(closed_root));
    } else {
        publish_document_messages(bus, session_messages);
    }
}

fn project_close_terminal_root<'a>(
    closed_root: Option<&'a Path>,
    retained_guard_root: Option<&'a Path>,
) -> Option<&'a Path> {
    closed_root.or(retained_guard_root)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::editor_error::EditorError;
    use crate::core::document::DocumentLifecycleAuthority;
    use crate::core::editor_message::{
        DocumentId, DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic,
        SharedEditorMessageBus, TOPIC_DOCUMENT,
    };

    use super::{
        project_close_terminal_root, project_diagnostics_configuration_message,
        publish_committed_project_close, publish_document_messages,
    };

    #[cfg(windows)]
    #[test]
    fn project_diagnostics_configuration_message_hides_windows_verbatim_operation_roots() {
        assert_eq!(
            project_diagnostics_configuration_message(
                Path::new(r"\\?\C:\projects\forest"),
                "access denied"
            ),
            r"editor diagnostics cannot be configured for `C:\projects\forest`: access denied"
        );
    }

    #[test]
    fn document_events_are_published_to_the_canonical_topic_in_lifecycle_order() {
        let bus = SharedEditorMessageBus::default();
        let topic = EditorTopic::parse(TOPIC_DOCUMENT).unwrap();
        let subscriber = bus.register_subscriber([topic]).unwrap();
        let document = DocumentId::new(42);

        publish_document_messages(
            &bus,
            [
                DocumentMessage::Opened { doc: document },
                DocumentMessage::Saved { doc: document },
                DocumentMessage::Closed { doc: document },
            ],
        );

        let delivered = bus.drain_deliveries(subscriber);
        assert_eq!(delivered.len(), 3);
        assert_eq!(
            delivered
                .iter()
                .map(|delivery| (delivery.topic().as_str(), delivery.message().clone()))
                .collect::<Vec<_>>(),
            vec![
                (
                    TOPIC_DOCUMENT,
                    EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Opened {
                        doc: document,
                    })),
                ),
                (
                    TOPIC_DOCUMENT,
                    EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Saved {
                        doc: document,
                    })),
                ),
                (
                    TOPIC_DOCUMENT,
                    EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Closed {
                        doc: document,
                    })),
                ),
            ]
        );
    }

    #[test]
    fn committed_project_close_publishes_one_closed_document_message_and_never_fabricates_one() {
        let bus = SharedEditorMessageBus::default();
        let topic = EditorTopic::parse(TOPIC_DOCUMENT).unwrap();
        let subscriber = bus.register_subscriber([topic]).unwrap();
        let lifecycle = DocumentLifecycleAuthority::default();
        let root = Path::new("C:/projects/close-producer");
        let document = match lifecycle.activate(root).as_slice() {
            [DocumentMessage::Opened { doc }] => *doc,
            actual => panic!("expected opened document, got {actual:?}"),
        };

        publish_committed_project_close(&bus, &lifecycle, None);
        assert!(bus.drain_deliveries(subscriber).is_empty());

        publish_committed_project_close(&bus, &lifecycle, Some(root));
        assert_eq!(
            bus.drain_deliveries(subscriber)
                .into_iter()
                .map(|delivery| delivery.message().clone())
                .collect::<Vec<_>>(),
            vec![EditorMessage::new(EditorMessagePayload::Document(
                DocumentMessage::Closed { doc: document },
            ))]
        );

        publish_committed_project_close(&bus, &lifecycle, Some(root));
        assert!(bus.drain_deliveries(subscriber).is_empty());
    }

    #[test]
    fn committed_project_close_closes_the_active_scene_document_for_a_project_session() {
        let bus = SharedEditorMessageBus::default();
        let topic = EditorTopic::parse(TOPIC_DOCUMENT).unwrap();
        let subscriber = bus.register_subscriber([topic]).unwrap();
        let lifecycle = DocumentLifecycleAuthority::default();
        let root = Path::new("C:/projects/close-active-scene");
        let session = lifecycle.begin_project_session(root).session;
        let scene = lifecycle
            .activate_scene(session, root, "res://scenes/main.scene.toml")
            .unwrap();

        publish_committed_project_close(&bus, &lifecycle, Some(root));

        assert_eq!(
            bus.drain_deliveries(subscriber)
                .into_iter()
                .map(|delivery| delivery.message().clone())
                .collect::<Vec<_>>(),
            vec![EditorMessage::new(EditorMessagePayload::Document(
                DocumentMessage::Closed {
                    doc: scene.document
                }
            ))]
        );
    }

    #[test]
    fn project_close_consumes_a_capability_and_quiesces_plugins_before_runtime() {
        let source = include_str!("editor_manager_project.rs");
        let close_start = source
            .find("pub(crate) fn commit_project_close")
            .expect("project close entry point");
        let close_end = source[close_start..]
            .find("pub(crate) fn save_active_scene")
            .map(|offset| close_start + offset)
            .expect("project close boundary");
        let close = &source[close_start..close_end];
        let plugin_close = close
            .find("clear_project_registration_reports()")
            .expect("plugin teardown");
        let runtime_close = close
            .find(".close_project(operation.project_root())")
            .expect("runtime project teardown");

        assert!(close.contains("operation: &ProjectCloseOperation"));
        assert!(plugin_close < runtime_close);
        assert!(close.contains("require_project_close_recovery"));
        assert!(!close.contains("release_project_close_guard"));

        let finalize = source
            .find("pub(crate) fn finalize_project_close")
            .expect("final close owner");
        assert!(source[finalize..].contains("self.release_project_close_guard(operation)?"));
    }

    #[test]
    fn project_close_retry_uses_the_retained_guard_root_after_host_close_has_committed() {
        let retained_root = Path::new("C:/projects/retained-close");

        assert_eq!(
            project_close_terminal_root(None, Some(retained_root)),
            Some(retained_root)
        );
    }

    #[test]
    fn active_scene_save_routing_uses_lifecycle_identity_without_a_manifest_default_fallback() {
        let source = include_str!("editor_manager_project.rs");
        let save_start = source
            .find("pub(crate) fn save_active_scene(")
            .expect("active-scene save entry point");
        let save_end = source[save_start..]
            .find("/// Publishes the manifest-selected startup scene")
            .map(|offset| save_start + offset)
            .expect("startup-scene boundary after active-scene save entry point");
        let save = &source[save_start..save_end];

        assert!(save.contains(".active_scene_identity(&project_root)"));
        assert!(save.contains(".save_active_scene(&project_root, &scene_uri, world)?"));
        assert!(save.contains(".save_scene_identity_if_active(&active_scene)"));
        assert!(!save.contains("manifest().default_scene"));
    }
}
