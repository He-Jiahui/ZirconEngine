use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use thiserror::Error;
use zircon_runtime_interface::ui::component::{
    UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiComponentEventError,
    UiDragPayload, UiValue,
};

use super::SHOWCASE_DOCUMENT_ID;
use super::defaults::component_id_for_control;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiComponentShowcaseDemoEventInput {
    None,
    Value(UiValue),
    Toggle(bool),
    Hover(bool),
    Press(bool),
    DragDelta(f64),
    LargeDragDelta(f64),
    DropHover(bool),
    ActiveDragTarget(bool),
    OpenPopupAt {
        x: f64,
        y: f64,
    },
    SelectOption {
        option_id: String,
        selected: bool,
    },
    DropReference {
        payload: UiDragPayload,
    },
    AddElement {
        value: UiValue,
    },
    SetElement {
        index: usize,
        value: UiValue,
    },
    RemoveElement {
        index: usize,
    },
    MoveElement {
        from: usize,
        to: usize,
    },
    AddMapEntry {
        key: String,
        value: UiValue,
    },
    SetMapEntry {
        key: String,
        value: UiValue,
    },
    RenameMapEntry {
        from_key: String,
        to_key: String,
    },
    RemoveMapEntry {
        key: String,
    },
    SetVisibleRange {
        start: i64,
        count: i64,
    },
    SetPage {
        page_index: i64,
        page_size: i64,
    },
    SetWorldTransform {
        position: [f64; 3],
        rotation: [f64; 3],
        scale: [f64; 3],
    },
    SetWorldSurface {
        size: [f64; 2],
        pixels_per_meter: f64,
        billboard: bool,
        depth_test: bool,
        render_order: i64,
        camera_target: String,
    },
}

pub(crate) struct UiComponentShowcaseResolvedEvent {
    pub(crate) action: String,
    pub(crate) changed_property: Option<String>,
    pub(crate) envelope: UiComponentEventEnvelope,
}

#[derive(Debug, Error, PartialEq)]
pub(crate) enum UiComponentShowcaseDemoError {
    #[error("binding does not carry a UiComponentShowcase custom payload")]
    UnsupportedPayload,
    #[error("showcase payload is missing string argument {index}")]
    MissingPayloadArgument { index: usize },
    #[error("unknown showcase control {control_id}")]
    UnknownControl { control_id: String },
    #[error("missing runtime component descriptor {component_id}")]
    MissingDescriptor { component_id: String },
    #[error("action {action} does not accept the provided event input")]
    InputMismatch { action: String },
    #[error(transparent)]
    Component(#[from] UiComponentEventError),
}

fn showcase_action(
    binding: &EditorUiBinding,
) -> Result<(&str, &str), UiComponentShowcaseDemoError> {
    let EditorUiBindingPayload::Custom(call) = binding.payload() else {
        return Err(UiComponentShowcaseDemoError::UnsupportedPayload);
    };
    if call.symbol != "UiComponentShowcase" {
        return Err(UiComponentShowcaseDemoError::UnsupportedPayload);
    }
    let action = call
        .argument(0)
        .and_then(|value| value.as_str())
        .ok_or(UiComponentShowcaseDemoError::MissingPayloadArgument { index: 0 })?;
    let control_id = call
        .argument(1)
        .and_then(|value| value.as_str())
        .ok_or(UiComponentShowcaseDemoError::MissingPayloadArgument { index: 1 })?;
    Ok((action, control_id))
}

pub(crate) fn resolve_showcase_component_event(
    binding: &EditorUiBinding,
    input: UiComponentShowcaseDemoEventInput,
) -> Result<UiComponentShowcaseResolvedEvent, UiComponentShowcaseDemoError> {
    let (action, control_id) = showcase_action(binding)?;
    let (event, changed_property, component_id) =
        if let Some(category) = action.strip_prefix("SelectCategory.") {
            (
                UiComponentEvent::Commit {
                    property: "selected_category".to_string(),
                    value: UiValue::String(category.to_string()),
                },
                Some("selected_category".to_string()),
                None,
            )
        } else {
            let component_id = component_id_for_control(control_id).ok_or_else(|| {
                UiComponentShowcaseDemoError::UnknownControl {
                    control_id: control_id.to_string(),
                }
            })?;
            let (event, changed_property) = component_event_for_action(action, input)?;
            (event, changed_property, Some(component_id))
        };

    let mut envelope = UiComponentEventEnvelope::new(
        SHOWCASE_DOCUMENT_ID,
        control_id,
        UiComponentBindingTarget::showcase(control_id),
        event,
    );
    if let Some(component_id) = component_id {
        envelope = envelope.with_component_id(component_id);
    }

    Ok(UiComponentShowcaseResolvedEvent {
        action: action.to_string(),
        changed_property,
        envelope,
    })
}

fn component_event_for_action(
    action: &str,
    input: UiComponentShowcaseDemoEventInput,
) -> Result<(UiComponentEvent, Option<String>), UiComponentShowcaseDemoError> {
    let mismatch = || UiComponentShowcaseDemoError::InputMismatch {
        action: action.to_string(),
    };
    let value_property = value_property_for_action(action);
    match action
        .split_once('.')
        .map(|(kind, _)| kind)
        .unwrap_or(action)
    {
        "Commit" => match input {
            UiComponentShowcaseDemoEventInput::Value(value) => {
                let property = value_property.to_string();
                Ok((
                    UiComponentEvent::Commit {
                        property: property.clone(),
                        value,
                    },
                    Some(property),
                ))
            }
            UiComponentShowcaseDemoEventInput::None => {
                let property = value_property.to_string();
                Ok((
                    UiComponentEvent::Commit {
                        property: property.clone(),
                        value: UiValue::Null,
                    },
                    Some(property),
                ))
            }
            _ => Err(mismatch()),
        },
        "ValueChanged" | "Change" => match input {
            UiComponentShowcaseDemoEventInput::Value(value) => Ok((
                UiComponentEvent::ValueChanged {
                    property: value_property.to_string(),
                    value,
                },
                Some(value_property.to_string()),
            )),
            UiComponentShowcaseDemoEventInput::Toggle(value) => Ok((
                UiComponentEvent::ValueChanged {
                    property: value_property.to_string(),
                    value: UiValue::Bool(value),
                },
                Some(value_property.to_string()),
            )),
            _ => Err(mismatch()),
        },
        "BeginDrag" => Ok((
            UiComponentEvent::BeginDrag {
                property: "value".to_string(),
            },
            None,
        )),
        "Hover" => match input {
            UiComponentShowcaseDemoEventInput::Hover(hovered) => {
                Ok((UiComponentEvent::Hover { hovered }, None))
            }
            _ => Err(mismatch()),
        },
        "Press" => match input {
            UiComponentShowcaseDemoEventInput::Press(pressed) => {
                Ok((UiComponentEvent::Press { pressed }, None))
            }
            _ => Err(mismatch()),
        },
        "DragDelta" => match input {
            UiComponentShowcaseDemoEventInput::DragDelta(delta) => Ok((
                UiComponentEvent::DragDelta {
                    property: "value".to_string(),
                    delta,
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "LargeDragDelta" => match input {
            UiComponentShowcaseDemoEventInput::LargeDragDelta(delta) => Ok((
                UiComponentEvent::LargeDragDelta {
                    property: "value".to_string(),
                    delta,
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "EndDrag" => Ok((
            UiComponentEvent::EndDrag {
                property: "value".to_string(),
            },
            Some("value".to_string()),
        )),
        "OpenPopup" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((UiComponentEvent::OpenPopup, None)),
            _ => Err(mismatch()),
        },
        "OpenPopupAt" => match input {
            UiComponentShowcaseDemoEventInput::OpenPopupAt { x, y } => {
                Ok((UiComponentEvent::OpenPopupAt { x, y }, None))
            }
            _ => Err(mismatch()),
        },
        "ClosePopup" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((UiComponentEvent::ClosePopup, None)),
            _ => Err(mismatch()),
        },
        "SelectOption" => match input {
            UiComponentShowcaseDemoEventInput::SelectOption {
                option_id,
                selected,
            } => {
                let option_id = if action.ends_with(".ContextActionMenu") {
                    context_action_menu_option_id(&option_id).ok_or_else(mismatch)?
                } else {
                    option_id
                };
                Ok((
                    UiComponentEvent::SelectOption {
                        property: "value".to_string(),
                        option_id,
                        selected,
                    },
                    Some("value".to_string()),
                ))
            }
            _ => Err(mismatch()),
        },
        "DropReference" => match input {
            UiComponentShowcaseDemoEventInput::DropReference { payload } => Ok((
                UiComponentEvent::DropReference {
                    property: "value".to_string(),
                    payload,
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "DropHover" => match input {
            UiComponentShowcaseDemoEventInput::DropHover(hovered) => {
                Ok((UiComponentEvent::DropHover { hovered }, None))
            }
            _ => Err(mismatch()),
        },
        "ActiveDragTarget" => match input {
            UiComponentShowcaseDemoEventInput::ActiveDragTarget(active) => {
                Ok((UiComponentEvent::ActiveDragTarget { active }, None))
            }
            _ => Err(mismatch()),
        },
        "ClearReference" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((
                UiComponentEvent::ClearReference {
                    property: "value".to_string(),
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "LocateReference" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((
                UiComponentEvent::LocateReference {
                    property: "value".to_string(),
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "OpenReference" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((
                UiComponentEvent::OpenReference {
                    property: "value".to_string(),
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "ToggleExpanded" => match input {
            UiComponentShowcaseDemoEventInput::Toggle(expanded) => Ok((
                UiComponentEvent::ToggleExpanded { expanded },
                Some("expanded".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "AddElement" => match input {
            UiComponentShowcaseDemoEventInput::AddElement { value } => Ok((
                UiComponentEvent::AddElement {
                    property: "items".to_string(),
                    value,
                },
                Some("items".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "SetElement" => match input {
            UiComponentShowcaseDemoEventInput::SetElement { index, value } => Ok((
                UiComponentEvent::SetElement {
                    property: "items".to_string(),
                    index,
                    value,
                },
                Some("items".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "RemoveElement" => match input {
            UiComponentShowcaseDemoEventInput::RemoveElement { index } => Ok((
                UiComponentEvent::RemoveElement {
                    property: "items".to_string(),
                    index,
                },
                Some("items".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "MoveElement" => match input {
            UiComponentShowcaseDemoEventInput::MoveElement { from, to } => Ok((
                UiComponentEvent::MoveElement {
                    property: "items".to_string(),
                    from,
                    to,
                },
                Some("items".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "AddMapEntry" => match input {
            UiComponentShowcaseDemoEventInput::AddMapEntry { key, value } => Ok((
                UiComponentEvent::AddMapEntry {
                    property: "entries".to_string(),
                    key,
                    value,
                },
                Some("entries".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "SetMapEntry" => match input {
            UiComponentShowcaseDemoEventInput::SetMapEntry { key, value } => Ok((
                UiComponentEvent::SetMapEntry {
                    property: "entries".to_string(),
                    key,
                    value,
                },
                Some("entries".to_string()),
            )),
            UiComponentShowcaseDemoEventInput::RenameMapEntry { from_key, to_key } => Ok((
                UiComponentEvent::RenameMapKey {
                    property: "entries".to_string(),
                    from_key,
                    to_key,
                },
                Some("entries".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "RemoveMapEntry" => match input {
            UiComponentShowcaseDemoEventInput::RemoveMapEntry { key } => Ok((
                UiComponentEvent::RemoveMapEntry {
                    property: "entries".to_string(),
                    key,
                },
                Some("entries".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "SetVisibleRange" => match input {
            UiComponentShowcaseDemoEventInput::SetVisibleRange { start, count } => Ok((
                UiComponentEvent::SetVisibleRange { start, count },
                Some("viewport_start".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "SetPage" => match input {
            UiComponentShowcaseDemoEventInput::SetPage {
                page_index,
                page_size,
            } => Ok((
                UiComponentEvent::SetPage {
                    page_index,
                    page_size,
                },
                Some("page_index".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "SetWorldTransform" => match input {
            UiComponentShowcaseDemoEventInput::SetWorldTransform {
                position,
                rotation,
                scale,
            } => Ok((
                UiComponentEvent::SetWorldTransform {
                    position,
                    rotation,
                    scale,
                },
                Some("world_position".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "SetWorldSurface" => match input {
            UiComponentShowcaseDemoEventInput::SetWorldSurface {
                size,
                pixels_per_meter,
                billboard,
                depth_test,
                render_order,
                camera_target,
            } => Ok((
                UiComponentEvent::SetWorldSurface {
                    size,
                    pixels_per_meter,
                    billboard,
                    depth_test,
                    render_order,
                    camera_target,
                },
                Some("world_size".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "Select" => Ok((
            UiComponentEvent::Focus { focused: true },
            Some("value".to_string()),
        )),
        _ => Err(mismatch()),
    }
}

fn context_action_menu_option_id(encoded: &str) -> Option<String> {
    if encoded == "---" {
        return None;
    }
    if let Some(action_segment) = encoded.strip_prefix("menu.item.") {
        return Some(
            action_segment
                .split('_')
                .filter(|segment| !segment.is_empty())
                .map(|segment| {
                    let mut chars = segment.chars();
                    let Some(first) = chars.next() else {
                        return String::new();
                    };
                    format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
                })
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    let mut parts = encoded.split('|');
    let label = parts.next()?.trim();
    let flags = parts.next().unwrap_or_default();
    if label.is_empty() || flags.split(',').any(|flag| flag.trim() == "disabled") {
        return None;
    }
    Some(label.to_string())
}

fn value_property_for_action(action: &str) -> &'static str {
    match action.rsplit_once('.').map(|(_, component)| component) {
        Some("SearchSelectQuery") => "query",
        Some("ArrayField") => "items",
        Some("MapField") => "entries",
        _ => "value",
    }
}
