use super::super::editor_error::{AnimationEditorDocumentLoadDiagnostic, EditorError};
use super::super::editor_ui_host::EditorUiHost;
use crate::core::asset::{builtin_asset_type_definition, AssetToolkitOpenRoute};
use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationAuthoringDocumentReadHandle,
};
use crate::core::editing::context::CoreEditContext;
use crate::ui::animation_editor::{
    AnimationEditorDocumentKind, AnimationEditorPanePresentation, AnimationEditorSession,
    AnimationEditorSessionError,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstance, ViewInstanceId};
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    save::{
        animation_document_autosave_source_path, capture_animation_document_autosave,
        save_animation_document,
    },
    AnimationEditorWorkspaceEntry,
};

const ANIMATION_TOOLKIT_LAYOUT_ID: &str = "editor.animation.layout";
const ANIMATION_TOOLKIT_TAB_ID: &str = "editor.animation.document";

fn document_kind_for_route(
    instance: &ViewInstance,
    route: &AssetToolkitOpenRoute,
) -> Result<AnimationEditorDocumentKind, EditorError> {
    for (resource_kind, document_kind) in [
        (
            ResourceKind::AnimationSequence,
            AnimationEditorDocumentKind::Sequence,
        ),
        (
            ResourceKind::AnimationGraph,
            AnimationEditorDocumentKind::Graph,
        ),
        (
            ResourceKind::AnimationStateMachine,
            AnimationEditorDocumentKind::StateMachine,
        ),
    ] {
        let Some(toolkit) = builtin_asset_type_definition(resource_kind)
            .and_then(|definition| definition.toolkit())
        else {
            continue;
        };
        if toolkit.open_operation() != route.open_operation() {
            continue;
        }
        if instance.descriptor_id != ViewDescriptorId::new(toolkit.view_id()) {
            return Err(EditorError::UiAsset(format!(
                "animation editor route {} requires view {} instead of {}",
                route.open_operation().as_str(),
                toolkit.view_id(),
                instance.descriptor_id.0
            )));
        }
        return Ok(document_kind);
    }

    Err(EditorError::UiAsset(format!(
        "unsupported animation editor operation {} for {}",
        route.open_operation().as_str(),
        instance.instance_id.0
    )))
}

fn animation_document_load_error(error: AnimationEditorSessionError) -> EditorError {
    match error.binary_kind_mismatch() {
        Some(mismatch) => EditorError::AnimationDocumentLoad {
            diagnostic: AnimationEditorDocumentLoadDiagnostic::binary_kind_mismatch(
                mismatch.expected(),
                mismatch.actual(),
            ),
        },
        None => EditorError::UiAsset(error.to_string()),
    }
}

impl EditorUiHost {
    pub fn animation_editor_pane_presentation(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<AnimationEditorPanePresentation, EditorError> {
        self.ensure_animation_editor_session(instance_id)?;
        let sessions = self.lock_animation_editor_sessions();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!(
                "missing animation editor session {}",
                instance_id.0
            ))
        })?;
        Ok(entry.session.pane_presentation())
    }

    pub(crate) fn restore_animation_editor_instance(
        &self,
        instance: &ViewInstance,
    ) -> Result<(), EditorError> {
        let route: AssetToolkitOpenRoute =
            serde_json::from_value(instance.serializable_payload.clone()).map_err(|error| {
                EditorError::UiAsset(format!(
                    "invalid animation editor route for {}: {error}",
                    instance.instance_id.0
                ))
            })?;
        let document_kind = document_kind_for_route(instance, &route)?;
        let source_path = self.resolve_asset_locator_path(route.asset_locator())?;
        let bytes =
            std::fs::read(&source_path).map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let asset = AnimationAuthoringAsset::from_bytes(document_kind, &bytes)
            .map_err(AnimationEditorSessionError::from_animation_asset_error)
            .map_err(animation_document_load_error)?;
        let original_metadata = (
            instance.title.clone(),
            instance.dirty,
            instance.serializable_payload.clone(),
        );
        let document = self.register_animation_document_toolkit(&instance.instance_id)?;
        let handle = match self.attach_animation_document(AnimationAuthoringDocument::new(
            document,
            route.asset_locator().clone(),
            asset,
        )) {
            Ok(handle) => handle,
            Err(error) => {
                let _ = self.unregister_document_toolkit(&instance.instance_id);
                return Err(error);
            }
        };
        let session =
            AnimationEditorSession::new(source_path.to_string_lossy().into_owned(), handle);
        self.lock_animation_editor_sessions().insert(
            instance.instance_id.clone(),
            AnimationEditorWorkspaceEntry {
                document,
                route,
                disk_source: bytes,
                session,
            },
        );
        if let Err(error) = self.sync_animation_editor_instance(&instance.instance_id) {
            self.rollback_animation_document_restore(&instance.instance_id, document);
            let _ = self.update_view_instance_metadata(
                &instance.instance_id,
                Some(original_metadata.0),
                Some(original_metadata.1),
                Some(original_metadata.2),
            );
            return Err(error);
        }
        Ok(())
    }

    fn attach_animation_document(
        &self,
        document: AnimationAuthoringDocument,
    ) -> Result<AnimationAuthoringDocumentReadHandle, EditorError> {
        self.transactions
            .with_context_mut::<CoreEditContext, _>(|context| {
                context.animation_documents_mut().attach(document)
            })
            .map_err(|error| EditorError::UiAsset(error.to_string()))?
            .ok_or_else(|| {
                EditorError::UiAsset("animation transaction context type mismatch".to_string())
            })?
            .map_err(|error| EditorError::UiAsset(error.to_string()))
    }

    fn rollback_animation_document_restore(
        &self,
        instance_id: &ViewInstanceId,
        document: crate::core::editor_message::DocumentId,
    ) {
        self.lock_animation_editor_sessions().remove(instance_id);
        let _ = self
            .transactions
            .with_context_mut::<CoreEditContext, _>(|context| {
                context.animation_documents_mut().detach(document)
            });
        let _ = self.unregister_document_toolkit(instance_id);
    }

    pub(super) fn ensure_animation_editor_session(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        if self
            .lock_animation_editor_sessions()
            .contains_key(instance_id)
        {
            self.register_animation_document_toolkit(instance_id)?;
            return Ok(());
        }
        let instance = self
            .lock_session()
            .open_view_instances
            .get(instance_id)
            .cloned()
            .ok_or_else(|| {
                EditorError::UiAsset(format!("missing animation editor view {}", instance_id.0))
            })?;
        self.restore_animation_editor_instance(&instance)
    }

    fn register_animation_document_toolkit(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        self.register_document_toolkit(
            instance_id,
            ANIMATION_TOOLKIT_LAYOUT_ID,
            ANIMATION_TOOLKIT_TAB_ID,
            super::save::validate_animation_document_references,
            save_animation_document,
            animation_document_autosave_source_path,
            capture_animation_document_autosave,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::asset::AssetUri;

    use super::*;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::ui::workbench::layout::MainPageId;
    use crate::ui::workbench::view::ViewHost;

    fn animation_instance(descriptor_id: &str) -> ViewInstance {
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.animation#route-test"),
            descriptor_id: ViewDescriptorId::new(descriptor_id),
            title: "Animation".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Document(MainPageId::workbench(), Vec::new()),
        }
    }

    fn animation_route(operation: &str) -> AssetToolkitOpenRoute {
        AssetToolkitOpenRoute::new(
            AssetUri::parse("res://animation/hero.zranim")
                .expect("animation fixture locator must be canonical"),
            EditorOperationPath::parse(operation)
                .expect("animation fixture operation must be valid"),
        )
    }

    #[test]
    fn route_operation_selects_the_registered_animation_document_kind() {
        for (descriptor_id, operation, expected_kind) in [
            (
                "editor.animation_sequence",
                "timeline_sequence.authoring.open",
                AnimationEditorDocumentKind::Sequence,
            ),
            (
                "editor.animation_graph",
                "animation_graph.authoring.open_graph",
                AnimationEditorDocumentKind::Graph,
            ),
            (
                "editor.animation_graph",
                "animation_graph.authoring.open_state_machine",
                AnimationEditorDocumentKind::StateMachine,
            ),
        ] {
            assert_eq!(
                document_kind_for_route(
                    &animation_instance(descriptor_id),
                    &animation_route(operation),
                )
                .expect("registered route should select its document kind"),
                expected_kind
            );
        }
    }

    #[test]
    fn route_rejects_an_operation_with_the_wrong_view_descriptor() {
        let result = document_kind_for_route(
            &animation_instance("editor.animation_graph"),
            &animation_route("timeline_sequence.authoring.open"),
        );

        assert!(result.is_err());
    }

    #[test]
    fn route_rejects_operations_outside_the_registered_animation_toolkits() {
        let result = document_kind_for_route(
            &animation_instance("editor.animation_graph"),
            &animation_route("animation_graph.authoring.compile"),
        );

        assert!(result.is_err());
    }
}
