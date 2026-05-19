//! Editor-only UI contracts, reusable widget/layout composition, Retained host runtime,
//! and workbench projection.

pub mod activity;
pub mod animation_editor;
pub mod asset_editor;
pub mod binding;
pub mod binding_dispatch;
pub mod control;
pub mod host;
pub(crate) mod layouts;
pub mod material_editor;
mod reflection;
pub mod retained_host;
pub(crate) mod template;
pub mod template_runtime;
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
