//! Workbench model, snapshot projection, and view registry.

pub(crate) mod asset_content_layout;
pub mod autolayout;
pub mod debug_reflector;
pub(crate) mod document_tabs;
pub mod event;
pub mod fixture;
mod floating_window;
pub mod layout;
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

pub use floating_window::{
    FLOATING_WINDOW_DESIGN_CONTRACTS, FloatingLayer, FloatingWindow, FloatingWindowContentLayout,
    FloatingWindowDesignContract, FloatingWindowInteractionMode, FloatingWindowKind,
    FloatingWindowPlacement,
};
pub use layout_preset::{
    CenterSplitLayout, LAYOUT_PRESET_PERSISTENCE_VERSION, LayoutPreset, LayoutPresetDrawerState,
    LayoutPresetName, LayoutPresetPersistenceEntry, LayoutPresetPersistenceStore,
    LayoutPresetRestoreFallback, LayoutPresetRestoreResult, LayoutPresetScope,
    LayoutPresetSizeOverride, LayoutUserId, PersistedLayoutPreset,
};
pub use page_layout_template::PageLayoutTemplate;
