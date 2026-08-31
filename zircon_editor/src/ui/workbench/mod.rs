//! Workbench model, snapshot projection, and view registry.

mod activity_log_console_projection;
pub(crate) mod asset_content_layout;
pub mod autolayout;
pub mod debug_reflector;
pub(crate) mod document_tabs;
pub mod event;
#[cfg(any(test, feature = "integration-contracts"))]
pub mod fixture;
#[cfg(any(test, feature = "integration-contracts"))]
mod floating_window;
pub mod layout;
pub(crate) mod layout_persistence_document;
mod layout_preset;
pub(crate) mod menu_bar;
pub mod model;
mod page_layout_template;
pub(crate) mod page_tabs;
pub mod preset;
pub mod project;
pub mod reference;
pub mod reflection;
pub(crate) mod shell_state;
pub mod snapshot;
pub mod startup;
pub mod state;
pub mod view;
pub mod window_registry;

pub(crate) use activity_log_console_projection::ActivityLogConsoleProjection;

#[cfg(any(test, feature = "integration-contracts"))]
pub use floating_window::{
    FloatingLayer, FloatingWindow, FloatingWindowContentLayout, FloatingWindowDesignContract,
    FloatingWindowInteractionMode, FloatingWindowKind, FloatingWindowPlacement,
    FLOATING_WINDOW_DESIGN_CONTRACTS,
};
pub use layout_preset::{
    CenterSplitLayout, LayoutPreset, LayoutPresetDrawerState, LayoutPresetName,
    LayoutPresetPersistenceEntry, LayoutPresetPersistenceStore, LayoutPresetRestoreFallback,
    LayoutPresetRestoreResult, LayoutPresetScope, LayoutPresetSizeOverride, LayoutUserId,
};
pub use page_layout_template::PageLayoutTemplate;
