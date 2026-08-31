use thiserror::Error;

use crate::core::commands::{CommandEvalCtx, EditorCommandDispatchError, EditorCommandRegistry};
use crate::core::editor_event::{
    EditorAnimationEvent, EditorAssetEvent, EditorDraftEvent, EditorEvent, EditorInspectorEvent,
    EditorViewportEvent,
};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::binding_dispatch::{
    dispatch_animation_binding, dispatch_asset_binding, dispatch_docking_binding,
    dispatch_draft_binding, dispatch_inspector_binding, dispatch_selection_binding,
    dispatch_viewport_binding, AnimationHostEvent, AssetHostEvent, DraftHostEvent,
    EditorBindingDispatchError,
};
use crate::ui::workbench::event::{
    dispatch_editor_host_binding, EditorHostEvent, EditorHostEventError,
};

#[derive(Debug, Error, PartialEq)]
pub enum EditorEventNormalizationError {
    #[error(transparent)]
    HostEvent(#[from] EditorHostEventError),
    #[error(transparent)]
    Command(#[from] EditorCommandDispatchError),
    #[error(transparent)]
    Binding(#[from] EditorBindingDispatchError),
    #[error("unsupported editor event binding {native_binding}")]
    UnsupportedBinding { native_binding: String },
}

pub(crate) fn normalize_editor_event_binding(
    binding: &EditorUiBinding,
    commands: &EditorCommandRegistry,
    context: &CommandEvalCtx,
) -> Result<EditorEvent, EditorEventNormalizationError> {
    match binding.payload() {
        EditorUiBindingPayload::MenuAction { .. } => {
            let EditorHostEvent::Menu(action) = dispatch_editor_host_binding(binding)?;
            Ok(EditorEvent::WorkbenchMenu(action))
        }
        EditorUiBindingPayload::EditorCommand { command_id } => commands
            .event_for_command(command_id, context)
            .map_err(EditorEventNormalizationError::Command),
        EditorUiBindingPayload::DockCommand(_) => {
            Ok(EditorEvent::Layout(dispatch_docking_binding(binding)?))
        }
        EditorUiBindingPayload::SelectionCommand(_) => {
            Ok(EditorEvent::Selection(dispatch_selection_binding(binding)?))
        }
        EditorUiBindingPayload::AssetCommand(_) => {
            let event = dispatch_asset_binding(binding)?;
            Ok(EditorEvent::Asset(match event {
                AssetHostEvent::OpenAsset { asset_locator } => {
                    EditorAssetEvent::OpenAsset { asset_locator }
                }
                AssetHostEvent::SelectFolder { folder_id } => {
                    EditorAssetEvent::SelectFolder { folder_id }
                }
                AssetHostEvent::SelectItem { asset_uuid } => {
                    EditorAssetEvent::SelectItem { asset_uuid }
                }
                AssetHostEvent::ActivateReference { asset_uuid } => {
                    EditorAssetEvent::ActivateReference { asset_uuid }
                }
                AssetHostEvent::SetSearchQuery { query } => {
                    EditorAssetEvent::SetSearchQuery { query }
                }
                AssetHostEvent::SetKindFilter { kind } => EditorAssetEvent::SetKindFilter { kind },
                AssetHostEvent::SetViewMode { surface, view_mode } => {
                    EditorAssetEvent::SetViewMode { surface, view_mode }
                }
                AssetHostEvent::SetUtilityTab { surface, tab } => {
                    EditorAssetEvent::SetUtilityTab { surface, tab }
                }
                AssetHostEvent::RelocateAsset {
                    asset_uuid,
                    target_locator,
                } => EditorAssetEvent::RelocateAsset {
                    asset_uuid,
                    target_locator,
                },
                AssetHostEvent::DeleteAsset { asset_uuid } => {
                    EditorAssetEvent::DeleteAsset { asset_uuid }
                }
                AssetHostEvent::OpenAssetBrowser => EditorAssetEvent::OpenAssetBrowser,
                AssetHostEvent::LocateSelectedAsset => EditorAssetEvent::LocateSelectedAsset,
                AssetHostEvent::ImportModel => EditorAssetEvent::ImportModel,
            }))
        }
        EditorUiBindingPayload::DraftCommand(_) => {
            let event = dispatch_draft_binding(binding)?;
            Ok(EditorEvent::Draft(match event {
                DraftHostEvent::SetInspectorField {
                    subject_path,
                    field_id,
                    value,
                } => EditorDraftEvent::SetInspectorField {
                    subject_path,
                    field_id,
                    value,
                },
                DraftHostEvent::SetMeshImportPath { value } => {
                    EditorDraftEvent::SetMeshImportPath { value }
                }
            }))
        }
        EditorUiBindingPayload::AnimationCommand(_) => {
            let event = dispatch_animation_binding(binding)?;
            Ok(EditorEvent::Animation(match event {
                AnimationHostEvent::AddKey { track_path, frame } => {
                    EditorAnimationEvent::AddKey { track_path, frame }
                }
                AnimationHostEvent::RemoveKey { track_path, frame } => {
                    EditorAnimationEvent::RemoveKey { track_path, frame }
                }
                AnimationHostEvent::CreateTrack { track_path } => {
                    EditorAnimationEvent::CreateTrack { track_path }
                }
                AnimationHostEvent::RemoveTrack { track_path } => {
                    EditorAnimationEvent::RemoveTrack { track_path }
                }
                AnimationHostEvent::RebindTrack {
                    from_track_path,
                    to_track_path,
                } => EditorAnimationEvent::RebindTrack {
                    from_track_path,
                    to_track_path,
                },
                AnimationHostEvent::ScrubTimeline { frame } => {
                    EditorAnimationEvent::ScrubTimeline { frame }
                }
                AnimationHostEvent::SetTimelineRange {
                    start_frame,
                    end_frame,
                } => EditorAnimationEvent::SetTimelineRange {
                    start_frame,
                    end_frame,
                },
                AnimationHostEvent::SelectTimelineSpan {
                    track_path,
                    start_frame,
                    end_frame,
                } => EditorAnimationEvent::SelectTimelineSpan {
                    track_path,
                    start_frame,
                    end_frame,
                },
                AnimationHostEvent::SetPlayback {
                    playing,
                    looping,
                    speed,
                } => EditorAnimationEvent::SetPlayback {
                    playing,
                    looping,
                    speed,
                },
                AnimationHostEvent::AddGraphNode {
                    graph_locator,
                    node_id,
                    node_kind,
                } => EditorAnimationEvent::AddGraphNode {
                    graph_locator,
                    node_id,
                    node_kind,
                },
                AnimationHostEvent::RemoveGraphNode {
                    graph_locator,
                    node_id,
                } => EditorAnimationEvent::RemoveGraphNode {
                    graph_locator,
                    node_id,
                },
                AnimationHostEvent::ConnectGraphNodes {
                    graph_locator,
                    from_node_id,
                    to_node_id,
                } => EditorAnimationEvent::ConnectGraphNodes {
                    graph_locator,
                    from_node_id,
                    to_node_id,
                },
                AnimationHostEvent::DisconnectGraphNodes {
                    graph_locator,
                    from_node_id,
                    to_node_id,
                } => EditorAnimationEvent::DisconnectGraphNodes {
                    graph_locator,
                    from_node_id,
                    to_node_id,
                },
                AnimationHostEvent::SetGraphParameter {
                    graph_locator,
                    parameter_name,
                    value_literal,
                } => EditorAnimationEvent::SetGraphParameter {
                    graph_locator,
                    parameter_name,
                    value_literal,
                },
                AnimationHostEvent::CreateState {
                    state_machine_locator,
                    state_name,
                    graph_locator,
                } => EditorAnimationEvent::CreateState {
                    state_machine_locator,
                    state_name,
                    graph_locator,
                },
                AnimationHostEvent::RemoveState {
                    state_machine_locator,
                    state_name,
                } => EditorAnimationEvent::RemoveState {
                    state_machine_locator,
                    state_name,
                },
                AnimationHostEvent::SetEntryState {
                    state_machine_locator,
                    state_name,
                } => EditorAnimationEvent::SetEntryState {
                    state_machine_locator,
                    state_name,
                },
                AnimationHostEvent::CreateTransition {
                    state_machine_locator,
                    from_state,
                    to_state,
                    duration_frames,
                } => EditorAnimationEvent::CreateTransition {
                    state_machine_locator,
                    from_state,
                    to_state,
                    duration_frames,
                },
                AnimationHostEvent::RemoveTransition {
                    state_machine_locator,
                    from_state,
                    to_state,
                } => EditorAnimationEvent::RemoveTransition {
                    state_machine_locator,
                    from_state,
                    to_state,
                },
                AnimationHostEvent::SetTransitionCondition {
                    state_machine_locator,
                    from_state,
                    to_state,
                    parameter_name,
                    operator,
                    value_literal,
                } => EditorAnimationEvent::SetTransitionCondition {
                    state_machine_locator,
                    from_state,
                    to_state,
                    parameter_name,
                    operator,
                    value_literal,
                },
            }))
        }
        EditorUiBindingPayload::InspectorFieldBatch { .. } => {
            let batch = dispatch_inspector_binding(binding)?;
            Ok(EditorEvent::Inspector(EditorInspectorEvent {
                subject_path: batch.subject_path,
                changes: batch.changes,
            }))
        }
        EditorUiBindingPayload::ViewportCommand(_) => {
            Ok(EditorEvent::Viewport(viewport_event_from_binding(binding)?))
        }
        EditorUiBindingPayload::WelcomeCommand(_)
        | EditorUiBindingPayload::EditorOperation { .. }
        | EditorUiBindingPayload::Custom(_) => {
            Err(EditorEventNormalizationError::UnsupportedBinding {
                native_binding: binding.native_binding(),
            })
        }
    }
}

fn viewport_event_from_binding(
    binding: &EditorUiBinding,
) -> Result<EditorViewportEvent, EditorEventNormalizationError> {
    let command = dispatch_viewport_binding(binding)?;
    Ok(crate::ui::retained_host::callback_dispatch::viewport_event_from_command(command))
}

#[cfg(test)]
mod tests {
    use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};
    use crate::ui::binding::{
        EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, WelcomeCommand,
    };

    use super::{normalize_editor_event_binding, EditorEventNormalizationError};

    #[test]
    fn normalization_preserves_unsupported_binding_as_a_typed_error() {
        let binding = EditorUiBinding::new(
            "WelcomeView",
            "CreateProjectButton",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::welcome_command(WelcomeCommand::CreateProject),
        );

        let error = normalize_editor_event_binding(
            &binding,
            &EditorCommandRegistry::default(),
            &CommandEvalCtx::default(),
        )
        .expect_err("welcome payload must not normalize as an editor event");

        assert!(matches!(
            error,
            EditorEventNormalizationError::UnsupportedBinding { ref native_binding }
                if native_binding.contains("WelcomeCommand.CreateProject")
        ));
    }

    #[test]
    fn normalization_preserves_asset_relocation_identity() {
        let binding = EditorUiBinding::new(
            "AssetTree",
            "RelocateAsset",
            EditorUiEventKind::Drop,
            EditorUiBindingPayload::asset_command(crate::ui::binding::AssetCommand::RelocateAsset {
                asset_uuid: "00112233-4455-6677-8899-aabbccddeeff".to_owned(),
                target_locator: "res://environment/cube.zmodel".to_owned(),
            }),
        );

        let event = normalize_editor_event_binding(
            &binding,
            &EditorCommandRegistry::default(),
            &CommandEvalCtx::default(),
        )
        .unwrap();

        assert!(matches!(
            event,
            crate::core::editor_event::EditorEvent::Asset(
                crate::core::editor_event::EditorAssetEvent::RelocateAsset {
                    asset_uuid,
                    target_locator,
                }
            ) if asset_uuid == "00112233-4455-6677-8899-aabbccddeeff"
                && target_locator == "res://environment/cube.zmodel"
        ));
    }
}
