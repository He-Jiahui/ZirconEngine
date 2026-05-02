use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{UiDragPayload, UiValue, UiValueKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiComponentEventKind {
    ValueChanged,
    Commit,
    Focus,
    Hover,
    Press,
    BeginDrag,
    DragDelta,
    LargeDragDelta,
    EndDrag,
    DropHover,
    ActiveDragTarget,
    OpenPopup,
    OpenPopupAt,
    ClosePopup,
    SelectOption,
    ToggleExpanded,
    AddElement,
    SetElement,
    RemoveElement,
    MoveElement,
    AddMapEntry,
    SetMapEntry,
    RenameMapKey,
    RemoveMapEntry,
    DropReference,
    ClearReference,
    LocateReference,
    OpenReference,
    SetVisibleRange,
    SetPage,
    SetWorldTransform,
    SetWorldSurface,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiComponentEvent {
    ValueChanged {
        property: String,
        value: UiValue,
    },
    Commit {
        property: String,
        value: UiValue,
    },
    Focus {
        focused: bool,
    },
    Hover {
        hovered: bool,
    },
    Press {
        pressed: bool,
    },
    BeginDrag {
        property: String,
    },
    DragDelta {
        property: String,
        delta: f64,
    },
    LargeDragDelta {
        property: String,
        delta: f64,
    },
    EndDrag {
        property: String,
    },
    DropHover {
        hovered: bool,
    },
    ActiveDragTarget {
        active: bool,
    },
    OpenPopup,
    OpenPopupAt {
        x: f64,
        y: f64,
    },
    ClosePopup,
    SelectOption {
        property: String,
        option_id: String,
        selected: bool,
    },
    ToggleExpanded {
        expanded: bool,
    },
    AddElement {
        property: String,
        value: UiValue,
    },
    SetElement {
        property: String,
        index: usize,
        value: UiValue,
    },
    RemoveElement {
        property: String,
        index: usize,
    },
    MoveElement {
        property: String,
        from: usize,
        to: usize,
    },
    AddMapEntry {
        property: String,
        key: String,
        value: UiValue,
    },
    SetMapEntry {
        property: String,
        key: String,
        value: UiValue,
    },
    RenameMapKey {
        property: String,
        from_key: String,
        to_key: String,
    },
    RemoveMapEntry {
        property: String,
        key: String,
    },
    DropReference {
        property: String,
        payload: UiDragPayload,
    },
    ClearReference {
        property: String,
    },
    LocateReference {
        property: String,
    },
    OpenReference {
        property: String,
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

impl UiComponentEvent {
    pub fn kind(&self) -> UiComponentEventKind {
        match self {
            Self::ValueChanged { .. } => UiComponentEventKind::ValueChanged,
            Self::Commit { .. } => UiComponentEventKind::Commit,
            Self::Focus { .. } => UiComponentEventKind::Focus,
            Self::Hover { .. } => UiComponentEventKind::Hover,
            Self::Press { .. } => UiComponentEventKind::Press,
            Self::BeginDrag { .. } => UiComponentEventKind::BeginDrag,
            Self::DragDelta { .. } => UiComponentEventKind::DragDelta,
            Self::LargeDragDelta { .. } => UiComponentEventKind::LargeDragDelta,
            Self::EndDrag { .. } => UiComponentEventKind::EndDrag,
            Self::DropHover { .. } => UiComponentEventKind::DropHover,
            Self::ActiveDragTarget { .. } => UiComponentEventKind::ActiveDragTarget,
            Self::OpenPopup => UiComponentEventKind::OpenPopup,
            Self::OpenPopupAt { .. } => UiComponentEventKind::OpenPopupAt,
            Self::ClosePopup => UiComponentEventKind::ClosePopup,
            Self::SelectOption { .. } => UiComponentEventKind::SelectOption,
            Self::ToggleExpanded { .. } => UiComponentEventKind::ToggleExpanded,
            Self::AddElement { .. } => UiComponentEventKind::AddElement,
            Self::SetElement { .. } => UiComponentEventKind::SetElement,
            Self::RemoveElement { .. } => UiComponentEventKind::RemoveElement,
            Self::MoveElement { .. } => UiComponentEventKind::MoveElement,
            Self::AddMapEntry { .. } => UiComponentEventKind::AddMapEntry,
            Self::SetMapEntry { .. } => UiComponentEventKind::SetMapEntry,
            Self::RenameMapKey { .. } => UiComponentEventKind::RenameMapKey,
            Self::RemoveMapEntry { .. } => UiComponentEventKind::RemoveMapEntry,
            Self::DropReference { .. } => UiComponentEventKind::DropReference,
            Self::ClearReference { .. } => UiComponentEventKind::ClearReference,
            Self::LocateReference { .. } => UiComponentEventKind::LocateReference,
            Self::OpenReference { .. } => UiComponentEventKind::OpenReference,
            Self::SetVisibleRange { .. } => UiComponentEventKind::SetVisibleRange,
            Self::SetPage { .. } => UiComponentEventKind::SetPage,
            Self::SetWorldTransform { .. } => UiComponentEventKind::SetWorldTransform,
            Self::SetWorldSurface { .. } => UiComponentEventKind::SetWorldSurface,
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiComponentEventError {
    #[error("component {component_id} does not support event {event_kind:?}")]
    UnsupportedEvent {
        component_id: String,
        event_kind: UiComponentEventKind,
    },
    #[error("property {property} is not numeric")]
    NonNumericProperty { property: String },
    #[error("invalid value `{value}` for numeric property {property}")]
    InvalidNumericValue { property: String, value: String },
    #[error("invalid value kind {actual:?} for property {property}; expected {expected:?}")]
    InvalidValueKind {
        property: String,
        expected: UiValueKind,
        actual: UiValueKind,
    },
    #[error("array property {property} has no element at index {index}")]
    ArrayIndexOutOfBounds { property: String, index: usize },
    #[error("map property {property} already contains key {key}")]
    DuplicateMapKey { property: String, key: String },
    #[error("map property {property} does not contain key {key}")]
    MissingMapKey { property: String, key: String },
    #[error("component {component_id} cannot select disabled option {option_id}")]
    DisabledOption {
        component_id: String,
        option_id: String,
    },
    #[error("drop payload kind {payload_kind} is not accepted by component {component_id}")]
    RejectedDrop {
        component_id: String,
        payload_kind: String,
    },
    #[error("reference property {property} does not contain a value")]
    MissingReference { property: String },
    #[error("event {event_kind:?} requires component {expected_component_id}")]
    UnsupportedComponentForEvent {
        expected_component_id: String,
        event_kind: UiComponentEventKind,
    },
    #[error("invalid complex component value {property}={value}")]
    InvalidComplexValue { property: String, value: String },
}
