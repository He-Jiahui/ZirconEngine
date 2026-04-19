//! Shared runtime UI implementation and data surface.

pub mod binding;
pub mod dispatch;
pub mod event_ui;
pub mod layout;
pub mod surface;
pub mod template;
pub mod tree;

pub use binding::*;
pub use event_ui::*;
pub use layout::{
    Anchor, AxisConstraint, BoxConstraints, DesiredSize, LayoutBoundary, Pivot, Position,
    ResolvedAxisConstraint, StretchMode, UiAxis, UiContainerKind, UiFrame, UiLinearBoxConfig,
    UiPoint, UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize,
    UiVirtualListConfig, UiVirtualListWindow,
};
pub use surface::{
    UiFocusState, UiNavigationEventKind, UiNavigationRoute, UiNavigationState, UiPointerButton,
    UiPointerEventKind, UiPointerRoute, UiSurface,
};
pub use template::{UiAssetDocument, UiAssetKind};
pub use tree::{UiInputPolicy, UiTree, UiTreeNode};

#[cfg(test)]
mod tests;
