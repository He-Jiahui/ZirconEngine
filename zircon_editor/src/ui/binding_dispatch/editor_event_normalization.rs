use crate::core::editor_event::{
    EditorAnimationEvent, EditorAssetEvent, EditorDraftEvent, EditorEvent, EditorInspectorEvent,
    EditorViewportEvent,
};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::binding_dispatch::{
    dispatch_animation_binding, dispatch_asset_binding, dispatch_docking_binding,
    dispatch_draft_binding, dispatch_inspector_binding, dispatch_selection_binding,
    dispatch_viewport_binding, AnimationHostEvent, AssetHostEvent, DraftHostEvent,
};
use crate::ui::workbench::event::{dispatch_editor_host_binding, EditorHostEvent};

pub(crate) fn normalize_editor_event_binding(
    binding: &EditorUiBinding,
) -> Result<EditorEvent, String> {
    match binding.payload() {
        EditorUiBindingPayload::MenuAction { .. } => {
            let EditorHostEvent::Menu(action) =
                dispatch_editor_host_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::WorkbenchMenu(action))
        }
        EditorUiBindingPayload::DockCommand(_) => Ok(EditorEvent::Layout(
            dispatch_docking_binding(binding).map_err(|error| error.to_string())?,
        )),
        EditorUiBindingPayload::SelectionCommand(_) => Ok(EditorEvent::Selection(
            dispatch_selection_binding(binding).map_err(|error| error.to_string())?,
        )),
        EditorUiBindingPayload::AssetCommand(_) => {
            let event = dispatch_asset_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::Asset(match event {
                AssetHostEvent::OpenAsset { asset_path } => {
                    EditorAssetEvent::OpenAsset { asset_path }
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
                AssetHostEvent::OpenAssetBrowser => EditorAssetEvent::OpenAssetBrowser,
                AssetHostEvent::LocateSelectedAsset => EditorAssetEvent::LocateSelectedAsset,
                AssetHostEvent::ImportModel => EditorAssetEvent::ImportModel,
            }))
        }
        EditorUiBindingPayload::DraftCommand(_) => {
            let event = dispatch_draft_binding(binding).map_err(|error| error.to_string())?;
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
            let event = dispatch_animation_binding(binding).map_err(|error| error.to_string())?;
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
                    graph_path,
                    node_id,
                    node_kind,
                } => EditorAnimationEvent::AddGraphNode {
                    graph_path,
                    node_id,
                    node_kind,
                },
                AnimationHostEvent::RemoveGraphNode {
                    graph_path,
                    node_id,
                } => EditorAnimationEvent::RemoveGraphNode {
                    graph_path,
                    node_id,
                },
                AnimationHostEvent::ConnectGraphNodes {
                    graph_path,
                    from_node_id,
                    to_node_id,
                } => EditorAnimationEvent::ConnectGraphNodes {
                    graph_path,
                    from_node_id,
                    to_node_id,
                },
                AnimationHostEvent::DisconnectGraphNodes {
                    graph_path,
                    from_node_id,
                    to_node_id,
                } => EditorAnimationEvent::DisconnectGraphNodes {
                    graph_path,
                    from_node_id,
                    to_node_id,
                },
                AnimationHostEvent::SetGraphParameter {
                    graph_path,
                    parameter_name,
                    value_literal,
                } => EditorAnimationEvent::SetGraphParameter {
                    graph_path,
                    parameter_name,
                    value_literal,
                },
                AnimationHostEvent::CreateState {
                    state_machine_path,
                    state_name,
                    graph_path,
                } => EditorAnimationEvent::CreateState {
                    state_machine_path,
                    state_name,
                    graph_path,
                },
                AnimationHostEvent::RemoveState {
                    state_machine_path,
                    state_name,
                } => EditorAnimationEvent::RemoveState {
                    state_machine_path,
                    state_name,
                },
                AnimationHostEvent::SetEntryState {
                    state_machine_path,
                    state_name,
                } => EditorAnimationEvent::SetEntryState {
                    state_machine_path,
                    state_name,
                },
                AnimationHostEvent::CreateTransition {
                    state_machine_path,
                    from_state,
                    to_state,
                    duration_frames,
                } => EditorAnimationEvent::CreateTransition {
                    state_machine_path,
                    from_state,
                    to_state,
                    duration_frames,
                },
                AnimationHostEvent::RemoveTransition {
                    state_machine_path,
                    from_state,
                    to_state,
                } => EditorAnimationEvent::RemoveTransition {
                    state_machine_path,
                    from_state,
                    to_state,
                },
                AnimationHostEvent::SetTransitionCondition {
                    state_machine_path,
                    from_state,
                    to_state,
                    parameter_name,
                    operator,
                    value_literal,
                } => EditorAnimationEvent::SetTransitionCondition {
                    state_machine_path,
                    from_state,
                    to_state,
                    parameter_name,
                    operator,
                    value_literal,
                },
            }))
        }
        EditorUiBindingPayload::InspectorFieldBatch { .. } => {
            let batch = dispatch_inspector_binding(binding).map_err(|error| error.to_string())?;
            Ok(EditorEvent::Inspector(EditorInspectorEvent {
                subject_path: batch.subject_path,
                changes: batch.changes,
            }))
        }
        EditorUiBindingPayload::ViewportCommand(_) => {
            Ok(EditorEvent::Viewport(viewport_event_from_binding(binding)?))
        }
        EditorUiBindingPayload::WelcomeCommand(_) | EditorUiBindingPayload::Custom(_) => {
            Err(format!(
                "unsupported editor event binding {}",
                binding.native_binding()
            ))
        }
    }
}

fn viewport_event_from_binding(binding: &EditorUiBinding) -> Result<EditorViewportEvent, String> {
    let command = dispatch_viewport_binding(binding).map_err(|error| error.to_string())?;
    Ok(crate::ui::slint_host::callback_dispatch::viewport_event_from_command(command))
}
