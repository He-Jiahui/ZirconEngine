//! Workbench model, snapshot projection, and view registry.

pub mod autolayout;
pub mod debug_reflector;
pub mod event;
pub mod fixture;
mod floating_window;
pub mod layout;
mod layout_preset;
pub mod model;
mod page_layout_template;
pub mod preset;
pub mod project;
pub mod reference;
pub mod reflection;
pub mod snapshot;
pub mod startup;
pub mod state;
pub mod view;
pub mod window_registry;

pub use floating_window::{
    FloatingLayer, FloatingWindow, FloatingWindowContentLayout, FloatingWindowDesignContract,
    FloatingWindowInteractionMode, FloatingWindowKind, FloatingWindowPlacement,
    FLOATING_WINDOW_DESIGN_CONTRACTS,
};
pub use layout_preset::{
    CenterSplitLayout, LayoutPreset, LayoutPresetDrawerState, LayoutPresetName,
    LayoutPresetPersistenceEntry, LayoutPresetPersistenceStore, LayoutPresetRestoreFallback,
    LayoutPresetRestoreResult, LayoutPresetScope, LayoutPresetSizeOverride, LayoutUserId,
    PersistedLayoutPreset, LAYOUT_PRESET_PERSISTENCE_VERSION,
};
pub use page_layout_template::PageLayoutTemplate;
