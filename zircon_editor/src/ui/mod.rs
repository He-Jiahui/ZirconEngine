//! Editor-only UI contracts, reusable widget/layout composition, Retained host runtime,
//! and workbench projection.

pub mod activity;
pub mod animation_editor;
pub mod asset_editor;
pub mod binding;
pub mod binding_dispatch;
pub(crate) mod component_registry;
pub mod control;
pub mod curve;
pub mod graph;
pub mod host;
pub(crate) mod layouts;
pub mod material_editor;
pub mod preview_scene;
mod reflection;
pub mod retained_host;
pub(crate) mod sample_grid;
pub mod settings;
pub(crate) mod template;
pub mod template_runtime;
pub mod timeline;
pub(crate) mod timeline_strip;
pub(crate) mod v2_design_tokens;
pub(crate) mod weight_heatmap;
pub(crate) mod widgets;
pub mod workbench;

pub use activity::{
    ActivityDrawerSlotPreference, ActivityViewDescriptor, ActivityWindowDescriptor,
};
pub use reflection::{
    EditorActivityHost, EditorActivityKind, EditorActivityReflection, EditorDrawerReflectionModel,
    EditorFloatingWindowReflectionModel, EditorHostPageReflectionModel,
    EditorMenuItemReflectionModel, EditorUiReflectionAdapter, EditorWorkbenchReflectionModel,
};
