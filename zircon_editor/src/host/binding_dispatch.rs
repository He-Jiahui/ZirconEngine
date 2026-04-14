//! Headless editor UI binding dispatch into concrete editor/layout/viewport actions.

use thiserror::Error;
use serde::{Deserialize, Serialize};
use zircon_editor_ui::{
    AssetCommand, DockCommand, EditorUiBinding, EditorUiBindingPayload, InspectorFieldChange,
    SelectionCommand, ViewportCommand,
};
use zircon_graphics::{ViewportFeedback, ViewportInput};
use zircon_math::{UVec2, Vec2};
use zircon_scene::NodeId;
use zircon_ui::UiBindingValue;

use crate::{
    ActivityDrawerMode, ActivityDrawerSlot, EditorIntent, EditorState, LayoutCommand, MainPageId,
    ViewInstanceId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationHostEvent {
    AddFrame { track_path: String, frame: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionHostEvent {
    SelectSceneNode { node_id: NodeId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetHostEvent {
    OpenAsset { asset_path: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InspectorBindingBatch {
    pub subject_path: String,
    pub changes: Vec<InspectorFieldChange>,
}

#[derive(Debug, Error, PartialEq)]
pub enum EditorBindingDispatchError {
    #[error("unsupported editor binding payload")]
    UnsupportedPayload,
    #[error("invalid subject path {0}")]
    InvalidSubjectPath(String),
    #[error("unsupported inspector field {0}")]
    UnsupportedInspectorField(String),
    #[error("invalid inspector field value for {field_id}")]
    InvalidInspectorFieldValue { field_id: String },
    #[error("unknown drawer slot {0}")]
    UnknownDrawerSlot(String),
    #[error("unknown drawer mode {0}")]
    UnknownDrawerMode(String),
    #[error("state mutation failed: {0}")]
    StateMutation(String),
}

pub fn dispatch_animation_binding(
    binding: &EditorUiBinding,
) -> Result<AnimationHostEvent, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::PositionOfTrackAndFrame { track_path, frame } => {
            Ok(AnimationHostEvent::AddFrame {
                track_path: track_path.clone(),
                frame: *frame,
            })
        }
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}

pub fn dispatch_selection_binding(
    binding: &EditorUiBinding,
) -> Result<SelectionHostEvent, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::SelectionCommand(SelectionCommand::SelectSceneNode { node_id }) => {
            Ok(SelectionHostEvent::SelectSceneNode { node_id: *node_id })
        }
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}

pub fn apply_selection_binding(
    state: &mut EditorState,
    binding: &EditorUiBinding,
) -> Result<bool, EditorBindingDispatchError> {
    match dispatch_selection_binding(binding)? {
        SelectionHostEvent::SelectSceneNode { node_id } => state
            .apply_intent(EditorIntent::SelectNode(node_id))
            .map_err(EditorBindingDispatchError::StateMutation),
    }
}

pub fn dispatch_asset_binding(
    binding: &EditorUiBinding,
) -> Result<AssetHostEvent, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::AssetCommand(AssetCommand::OpenAsset { asset_path }) => {
            Ok(AssetHostEvent::OpenAsset {
                asset_path: asset_path.clone(),
            })
        }
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}

pub fn dispatch_inspector_binding(
    binding: &EditorUiBinding,
) -> Result<InspectorBindingBatch, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::InspectorFieldBatch {
            subject_path,
            changes,
        } => Ok(InspectorBindingBatch {
            subject_path: subject_path.clone(),
            changes: changes.clone(),
        }),
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}

pub fn apply_inspector_binding(
    state: &mut EditorState,
    binding: &EditorUiBinding,
) -> Result<bool, EditorBindingDispatchError> {
    let batch = dispatch_inspector_binding(binding)?;
    let node_id = resolve_subject_path(state, &batch.subject_path)?;
    if state.world.with_world(|scene| scene.selected_node()) != Some(node_id) {
        state
            .apply_intent(EditorIntent::SelectNode(node_id))
            .map_err(EditorBindingDispatchError::StateMutation)?;
    }

    for change in &batch.changes {
        match change.field_id.as_str() {
            "name" => {
                state.update_name_field(binding_value_to_string(&change.value, &change.field_id)?)
            }
            "parent" => state.update_parent_field(parent_binding_value_to_string(
                &change.value,
                &change.field_id,
            )?),
            "transform.translation.x" => {
                state.update_translation_field(
                    0,
                    binding_value_to_string(&change.value, &change.field_id)?,
                );
            }
            "transform.translation.y" => {
                state.update_translation_field(
                    1,
                    binding_value_to_string(&change.value, &change.field_id)?,
                );
            }
            "transform.translation.z" => {
                state.update_translation_field(
                    2,
                    binding_value_to_string(&change.value, &change.field_id)?,
                );
            }
            other => {
                return Err(EditorBindingDispatchError::UnsupportedInspectorField(
                    other.to_string(),
                ))
            }
        }
    }

    state
        .apply_intent(EditorIntent::ApplyInspectorChanges)
        .map_err(EditorBindingDispatchError::StateMutation)
}

pub fn dispatch_docking_binding(
    binding: &EditorUiBinding,
) -> Result<LayoutCommand, EditorBindingDispatchError> {
    let EditorUiBindingPayload::DockCommand(command) = binding.payload() else {
        return Err(EditorBindingDispatchError::UnsupportedPayload);
    };

    match command {
        DockCommand::FocusView { instance_id } => Ok(LayoutCommand::FocusView {
            instance_id: ViewInstanceId::new(instance_id),
        }),
        DockCommand::CloseView { instance_id } => Ok(LayoutCommand::CloseView {
            instance_id: ViewInstanceId::new(instance_id),
        }),
        DockCommand::AttachViewToDrawer { instance_id, slot } => Ok(LayoutCommand::AttachView {
            instance_id: ViewInstanceId::new(instance_id),
            target: crate::ViewHost::Drawer(parse_drawer_slot(slot)?),
            anchor: None,
        }),
        DockCommand::AttachViewToDocument {
            instance_id,
            page_id,
        } => Ok(LayoutCommand::AttachView {
            instance_id: ViewInstanceId::new(instance_id),
            target: crate::ViewHost::Document(MainPageId::new(page_id), Vec::new()),
            anchor: None,
        }),
        DockCommand::DetachViewToWindow {
            instance_id,
            window_id,
        } => Ok(LayoutCommand::DetachViewToWindow {
            instance_id: ViewInstanceId::new(instance_id),
            new_window: MainPageId::new(window_id),
        }),
        DockCommand::ActivateDrawerTab { slot, instance_id } => {
            Ok(LayoutCommand::ActivateDrawerTab {
                slot: parse_drawer_slot(slot)?,
                instance_id: ViewInstanceId::new(instance_id),
            })
        }
        DockCommand::ActivateMainPage { page_id } => Ok(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new(page_id),
        }),
        DockCommand::SetDrawerMode { slot, mode } => Ok(LayoutCommand::SetDrawerMode {
            slot: parse_drawer_slot(slot)?,
            mode: parse_drawer_mode(mode)?,
        }),
        DockCommand::SetDrawerExtent { slot, extent } => Ok(LayoutCommand::SetDrawerExtent {
            slot: parse_drawer_slot(slot)?,
            extent: *extent,
        }),
        DockCommand::SavePreset { name } => Ok(LayoutCommand::SavePreset { name: name.clone() }),
        DockCommand::LoadPreset { name } => Ok(LayoutCommand::LoadPreset { name: name.clone() }),
        DockCommand::ResetToDefault => Ok(LayoutCommand::ResetToDefault),
    }
}

pub fn dispatch_viewport_binding(
    binding: &EditorUiBinding,
) -> Result<ViewportInput, EditorBindingDispatchError> {
    let EditorUiBindingPayload::ViewportCommand(command) = binding.payload() else {
        return Err(EditorBindingDispatchError::UnsupportedPayload);
    };

    Ok(match command {
        ViewportCommand::PointerMoved { x, y } => ViewportInput::PointerMoved(Vec2::new(*x, *y)),
        ViewportCommand::LeftPressed { x, y } => ViewportInput::LeftPressed(Vec2::new(*x, *y)),
        ViewportCommand::LeftReleased => ViewportInput::LeftReleased,
        ViewportCommand::RightPressed { x, y } => ViewportInput::RightPressed(Vec2::new(*x, *y)),
        ViewportCommand::RightReleased => ViewportInput::RightReleased,
        ViewportCommand::MiddlePressed { x, y } => ViewportInput::MiddlePressed(Vec2::new(*x, *y)),
        ViewportCommand::MiddleReleased => ViewportInput::MiddleReleased,
        ViewportCommand::Scrolled { delta } => ViewportInput::Scrolled(*delta),
        ViewportCommand::Resized { width, height } => {
            ViewportInput::Resized(UVec2::new(*width, *height))
        }
    })
}

pub fn apply_viewport_binding(
    state: &mut EditorState,
    binding: &EditorUiBinding,
) -> Result<ViewportFeedback, EditorBindingDispatchError> {
    let input = dispatch_viewport_binding(binding)?;
    Ok(state.handle_viewport_input(input))
}

fn resolve_subject_path(
    state: &EditorState,
    subject_path: &str,
) -> Result<NodeId, EditorBindingDispatchError> {
    if subject_path == "entity://selected" {
        return state
            .world
            .with_world(|scene| scene.selected_node())
            .ok_or_else(|| {
                EditorBindingDispatchError::InvalidSubjectPath(subject_path.to_string())
            });
    }

    if let Some(raw) = subject_path.strip_prefix("node://") {
        let node_id = raw.parse::<NodeId>().map_err(|_| {
            EditorBindingDispatchError::InvalidSubjectPath(subject_path.to_string())
        })?;
        let exists = state
            .world
            .with_world(|scene| scene.find_node(node_id).is_some());
        if exists {
            return Ok(node_id);
        }
    }

    Err(EditorBindingDispatchError::InvalidSubjectPath(
        subject_path.to_string(),
    ))
}

fn binding_value_to_string(
    value: &UiBindingValue,
    field_id: &str,
) -> Result<String, EditorBindingDispatchError> {
    match value {
        UiBindingValue::String(value) => Ok(value.clone()),
        UiBindingValue::Unsigned(value) => Ok(value.to_string()),
        UiBindingValue::Signed(value) => Ok(value.to_string()),
        UiBindingValue::Float(value) => Ok(value.to_string()),
        UiBindingValue::Bool(value) => Ok(value.to_string()),
        UiBindingValue::Null => Ok(String::new()),
        UiBindingValue::Array(_) => Err(EditorBindingDispatchError::InvalidInspectorFieldValue {
            field_id: field_id.to_string(),
        }),
    }
}

fn parent_binding_value_to_string(
    value: &UiBindingValue,
    field_id: &str,
) -> Result<String, EditorBindingDispatchError> {
    match value {
        UiBindingValue::Null => Ok(String::new()),
        _ => binding_value_to_string(value, field_id),
    }
}

fn parse_drawer_slot(slot: &str) -> Result<ActivityDrawerSlot, EditorBindingDispatchError> {
    match slot {
        "left_top" => Ok(ActivityDrawerSlot::LeftTop),
        "left_bottom" => Ok(ActivityDrawerSlot::LeftBottom),
        "right_top" => Ok(ActivityDrawerSlot::RightTop),
        "right_bottom" => Ok(ActivityDrawerSlot::RightBottom),
        "bottom_left" => Ok(ActivityDrawerSlot::BottomLeft),
        "bottom_right" => Ok(ActivityDrawerSlot::BottomRight),
        _ => Err(EditorBindingDispatchError::UnknownDrawerSlot(
            slot.to_string(),
        )),
    }
}

fn parse_drawer_mode(mode: &str) -> Result<ActivityDrawerMode, EditorBindingDispatchError> {
    match mode {
        "Pinned" => Ok(ActivityDrawerMode::Pinned),
        "AutoHide" => Ok(ActivityDrawerMode::AutoHide),
        "Collapsed" => Ok(ActivityDrawerMode::Collapsed),
        _ => Err(EditorBindingDispatchError::UnknownDrawerMode(
            mode.to_string(),
        )),
    }
}
