use std::path::Path;

use zircon_runtime::asset::project::{ProjectManager, ProjectManifest};
use zircon_runtime::plugin::native::NativePluginLoader;

use crate::core::editor_message::{
    DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic, SharedEditorMessageBus,
};
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::EditorStartupSessionDocument;

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        let document = self.host.open_project(path)?;
        self.apply_project_plugin_manifest_or_close(&document.root_path, &document.manifest)?;
        self.publish_document_messages(self.document_lifecycle.activate(&document.root_path));
        Ok(document)
    }

    pub fn close_project(&self) -> Result<Option<std::path::PathBuf>, EditorError> {
        let closed_root = self.host.close_project()?;
        self.clear_project_plugin_status();
        let registration_cleanup = if closed_root.is_some() {
            self.plugin_manager
                .clear_project_registration_reports()
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
        Ok(closed_root)
    }

    pub(crate) fn open_prepared_project_and_remember(
        &self,
        project: ProjectManager,
    ) -> Result<crate::ui::workbench::startup::EditorStartupSessionDocument, EditorError> {
        let session = self.host.open_prepared_project_and_remember(project)?;
        self.publish_document_startup_session(&session)?;
        Ok(session)
    }

    pub fn save_project(
        &self,
        path: impl AsRef<Path>,
        world: &zircon_runtime::scene::Scene,
    ) -> Result<(), EditorError> {
        let project_root = self.host.save_project(path, world)?;
        self.publish_document_messages(self.document_lifecycle.save(&project_root));
        Ok(())
    }

    pub fn create_runtime_level(
        &self,
        scene: zircon_runtime::scene::Scene,
    ) -> Result<zircon_runtime::scene::LevelSystem, EditorError> {
        self.host.create_runtime_level(scene)
    }

    pub(crate) fn publish_document_startup_session(
        &self,
        session: &EditorStartupSessionDocument,
    ) -> Result<(), EditorError> {
        let Some(document) = session.project.as_ref() else {
            return Ok(());
        };
        self.apply_project_plugin_manifest_or_close(&document.root_path, &document.manifest)?;
        self.publish_document_messages(self.document_lifecycle.activate(&document.root_path));
        Ok(())
    }

    fn apply_project_plugin_manifest_or_close(
        &self,
        project_root: &Path,
        manifest: &ProjectManifest,
    ) -> Result<(), EditorError> {
        if let Err(plugin_error) = self.apply_project_plugin_manifest(project_root, manifest) {
            self.clear_project_plugin_status();
            let cleared = self.plugin_manager.clear_project_registration_reports();
            return match (self.host.close_project(), cleared) {
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

    fn apply_project_plugin_manifest(
        &self,
        project_root: &Path,
        manifest: &ProjectManifest,
    ) -> Result<(), EditorError> {
        let native_report =
            NativePluginLoader.load_discovered_editor(self.plugin_directory(project_root));
        let completed =
            self.complete_project_plugin_manifest_with_native_report(manifest, &native_report);
        let native_reports = self
            .selected_native_editor_plugin_registration_reports_from_load_report(
                &native_report,
                &completed.plugins,
            );
        self.plugin_manager
            .publish_project_registration_reports(native_reports)
            .map_err(|error| {
                EditorError::Project(format!(
                    "project-native editor registrations cannot be published: {error}"
                ))
            })?;
        self.plugin_manager
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

    fn publish_document_messages(&self, messages: impl IntoIterator<Item = DocumentMessage>) {
        publish_document_messages(self.context().bus(), messages);
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
    publish_document_messages(bus, lifecycle.close(closed_root));
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::core::document::DocumentLifecycleAuthority;
    use crate::core::editor_message::{
        DocumentId, DocumentMessage, EditorMessage, EditorMessagePayload, EditorTopic,
        SharedEditorMessageBus, TOPIC_DOCUMENT,
    };

    use super::{publish_committed_project_close, publish_document_messages};

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
}
