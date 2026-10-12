//! Workbench model, snapshot projection, and view registry.

#[path = "activity_log_console_projection.rs"]
mod activity_log_console_projection;
#[path = "asset_content_layout/mod.rs"]
pub(crate) mod asset_content_layout;
#[path = "autolayout/mod.rs"]
pub mod autolayout;
#[path = "debug_reflector/mod.rs"]
pub mod debug_reflector;
#[path = "document_tabs/mod.rs"]
pub(crate) mod document_tabs;
#[path = "event/mod.rs"]
pub mod event;
#[cfg(any(test, feature = "integration-contracts"))]
#[path = "fixture/mod.rs"]
pub mod fixture;
#[cfg(any(test, feature = "integration-contracts"))]
#[path = "floating_window.rs"]
mod floating_window;
#[path = "layout/mod.rs"]
pub mod layout;
#[path = "layout_persistence_document.rs"]
pub(crate) mod layout_persistence_document;
#[path = "layout_preset.rs"]
mod layout_preset;
#[path = "menu_bar/mod.rs"]
pub(crate) mod menu_bar;
#[path = "model/mod.rs"]
pub mod model;
#[path = "page_layout_template.rs"]
mod page_layout_template;
#[path = "page_tabs/mod.rs"]
pub(crate) mod page_tabs;
#[path = "preset/mod.rs"]
pub mod preset;
#[path = "project/mod.rs"]
pub mod project;
#[path = "reference/mod.rs"]
pub mod reference;
#[path = "reflection/mod.rs"]
pub mod reflection;
#[path = "shell_state.rs"]
pub(crate) mod shell_state;
#[path = "snapshot/mod.rs"]
pub mod snapshot;
#[path = "startup/mod.rs"]
pub mod startup;
#[path = "state/mod.rs"]
pub mod state;
#[path = "view/mod.rs"]
pub mod view;
#[path = "window_registry/mod.rs"]
pub mod window_registry;

pub(crate) use activity_log_console_projection::ActivityLogConsoleProjection;

#[cfg(any(test, feature = "integration-contracts"))]
pub use floating_window::{
    FloatingLayer, FloatingWindow, FloatingWindowContentLayout, FloatingWindowDesignContract,
    FloatingWindowInteractionMode, FloatingWindowKind, FloatingWindowPlacement,
    FLOATING_WINDOW_DESIGN_CONTRACTS,
};
pub use layout_preset::{
    CenterSplitLayout, CenterSplitNode, LayoutPreset, LayoutPresetDrawerState, LayoutPresetName,
    LayoutPresetPersistenceEntry, LayoutPresetPersistenceStore, LayoutPresetRestoreFallback,
    LayoutPresetRestoreResult, LayoutPresetScope, LayoutPresetSizeOverride, LayoutUserId,
};
pub use page_layout_template::PageLayoutTemplate;
