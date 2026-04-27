use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{UiDragPayload, UiValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiComponentEventKind {
    ValueChanged,
    Commit,
    Focus,
    BeginDrag,
    DragDelta,
    LargeDragDelta,
    EndDrag,
    OpenPopup,
    ClosePopup,
    SelectOption,
    ToggleExpanded,
    AddElement,
    SetElement,
    RemoveElement,
    MoveElement,
    AddMapEntry,
    SetMapEntry,
    RemoveMapEntry,
    DropReference,
    ClearReference,
    LocateReference,
    OpenReference,
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
    OpenPopup,
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
}
