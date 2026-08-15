use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;

use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetManager, AssetUri};
use zircon_runtime::plugin::native::load_discovered_native_editor_plugins;

use crate::core::document::{
    AuthoringSceneInstaller, SceneAssetCatalog, SceneDocumentRoute, SceneDocumentRouteError,
    SceneDocumentRouteResult, ScenePickerTicket,
};
use crate::core::editing::authoring_world::AuthoringWorldSeed;
use crate::core::editor_message::{
    DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic, SharedEditorMessageBus,
};
use crate::core::project::{SceneCreateRequest, SceneOpenRequest};
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::EditorStartupSessionDocument;

use super::editor_asset_manager::EditorAssetManager;
use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        self.open_project_document(path)
    }

    pub fn close_project(&self) -> Result<Option<std::path::PathBuf>, EditorError> {
        let closed_root = self.host.close_project()?;
        let session_release = self.release_project_session_guard();
        if closed_root.is_some() {
            self.context().logs().disable_rolling_file();
            self.context().settings().clear_project_layer();
        }
        self.clear_project_plugin_status();
        let registration_cleanup = if closed_root.is_some() {
            self.plugin_manager()
                .clear_project_registration_reports()
                .map(|_| ())
                .map_err(|error| {
                    EditorError::Project(format!(
                        "project-native editor registrations cannot be cleared after close: {error}"
                    ))
                })
        } else {
            Ok(())
        };
        publish_committed_project_close(
            self.context().bus(),
            &self.document_lifecycle,
            closed_root.as_deref(),
        );
        registration_cleanup?;
        session_release?;
        Ok(closed_root)
    }

    pub(crate) fn open_prepared_project_and_remember(
        &self,
        project: ProjectManager,
    ) -> Result<crate::ui::workbench::startup::EditorStartupSessionDocument, EditorError> {
        self.open_prepared_project_and_remember_with_session(project)
    }

    pub(crate) fn save_project(
        &self,
        path: impl AsRef<Path>,
        world: &zircon_runtime::scene::Scene,
    ) -> Result<(), EditorError> {
        let project_root = self.host.save_project(path, world)?;
        self.publish_document_messages(
            self.document_lifecycle
                .save_active_project_session(&project_root),
        );
        Ok(())
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

    pub(crate) fn publish_document_startup_session(
        &self,
        session: &EditorStartupSessionDocument,
    ) -> Result<(), EditorError> {
        let Some(document) = session.project.as_ref() else {
            return Ok(());
        };
        self.configure_project_diagnostics(&document.root_path)?;
        self.apply_project_plugin_manifest_or_close(&document.root_path, &document.manifest)?;
        let activation = self
            .document_lifecycle
            .begin_project_session(&document.root_path);
        self.publish_document_messages(activation.messages);
        Ok(())
    }

    fn apply_project_plugin_manifest_or_close(
        &self,
        project_root: &Path,
        manifest: &ProjectManifest,
    ) -> Result<(), EditorError> {
        if let Err(plugin_error) = self.apply_project_plugin_manifest(project_root, manifest) {
            self.clear_project_plugin_status();
            let cleared = self.plugin_manager().clear_project_registration_reports();
            let close_result = self.host.close_project();
            if close_result.is_ok() {
                self.context().logs().disable_rolling_file();
            }
            return match (close_result, cleared) {
                (Ok(_), Ok(_)) => Err(plugin_error),
                (Err(close_error), _) => Err(EditorError::Project(format!(
                    "project plugin manifest synchronization failed: {plugin_error}; \
                     additionally failed to roll back the opened project: {close_error}"
                ))),
                (Ok(_), Err(clear_error)) => Err(EditorError::Project(format!(
                    "project plugin manifest synchronization failed: {plugin_error}; \
                     additionally failed to clear project-native registrations: {clear_error}"
                ))),
            };
        }
        Ok(())
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
    ) -> Result<(), EditorError> {
        let native_report =
            load_discovered_native_editor_plugins(self.plugin_directory(project_root));
        let completed =
            self.complete_project_plugin_manifest_with_native_report(manifest, &native_report);
        let native_reports = self
            .selected_native_editor_plugin_registration_reports_from_load_report(
                &native_report,
                &completed.plugins,
            );
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
        self.publish_project_plugin_status(
            self.native_plugin_status_report_from_load_report(&completed, &native_report),
        );
        Ok(())
    }

    pub(super) fn publish_document_messages(
        &self,
        messages: impl IntoIterator<Item = DocumentMessage>,
    ) {
        publish_document_messages(self.context().bus(), messages);
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
        let result = route(SceneDocumentRoute::new(
            &project,
            &self.document_lifecycle,
            ticket,
        ))
        .map_err(|error| EditorError::Project(error.to_string()))?;
        if let SceneDocumentRouteResult::Activated(activation) = &result {
            self.publish_document_messages(activation.activation.messages.clone());
        }
        Ok(result)
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::core::document::DocumentLifecycleAuthority;
    use crate::core::editor_message::{
        DocumentId, DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic,
        SharedEditorMessageBus, TOPIC_DOCUMENT,
    };

    use super::{
        project_diagnostics_configuration_message, publish_committed_project_close,
        publish_document_messages,
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
}
