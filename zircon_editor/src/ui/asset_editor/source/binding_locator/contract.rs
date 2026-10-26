use zircon_runtime_interface::ui::{
    binding::UiEventKind, component::UiComponentEventKind, template::UiBindingRef,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UiAssetSourceOwner {
    AssetRoot { root_node_id: String },
    Component { name: String, root_node_id: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiAssetBindingSourceScope {
    Current,
    LastValid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetBindingSourceSelector {
    pub node_id: String,
    pub binding_id: String,
    pub event: UiEventKind,
    pub component_event: Option<UiComponentEventKind>,
    pub owner: Option<UiAssetSourceOwner>,
}

/// Byte offsets are UTF-8; line is one-based and utf16_column is zero-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetSourcePosition {
    pub byte_offset: usize,
    pub line: usize,
    pub utf16_column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetSourceRange {
    pub start: UiAssetSourcePosition,
    pub end: UiAssetSourcePosition,
}

/// Source syntax only. A binding route is not an external handler definition.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiAssetBindingSourceDeclaration {
    pub asset_id: String,
    pub node_id: String,
    pub owner: UiAssetSourceOwner,
    pub control_component: String,
    pub source_binding_index: usize,
    pub binding: UiBindingRef,
    pub declaration: UiAssetSourceRange,
    pub id: UiAssetSourceRange,
    pub event: UiAssetSourceRange,
    pub component_event: Option<UiAssetSourceRange>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum UiAssetBindingSourceError {
    #[error("exact binding source lookup currently requires a v2 document")]
    UnsupportedSchema,
    #[error("current source is invalid or has not been validated")]
    InvalidCurrentSource,
    #[error("binding source location belongs to a retired source revision or document")]
    StaleLocation,
    #[error("last-valid source is read-only; it cannot navigate the current edit buffer")]
    LastValidReadOnly,
    #[error("cannot parse binding source: {detail}")]
    ParseSource { detail: String },
    #[error("source node {node_id} is unavailable")]
    MissingNode { node_id: String },
    #[error("source binding {binding_id} is unavailable")]
    MissingBinding { binding_id: String },
    #[error("binding id {binding_id} occurs {count} times on this source node")]
    AmbiguousBinding { binding_id: String, count: usize },
    #[error("the binding event no longer matches the requested event")]
    EventMismatch,
    #[error("source node has no declared root or component owner")]
    MissingOwner,
    #[error("source node has multiple declared owners: {owners:?}")]
    AmbiguousOwner { owners: Vec<UiAssetSourceOwner> },
    #[error("the requested source owner does not own this node")]
    OwnerMismatch,
    #[error("binding spans do not match the validated source document")]
    SourceMismatch,
    #[error("the current authoring projection cannot select this source binding")]
    SelectionUnavailable,
    #[error("source range is outside a UTF-8 character boundary")]
    InvalidRange,
}
