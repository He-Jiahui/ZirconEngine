//! Editor-only UI contracts, reusable widget/layout composition, Retained host runtime,
//! and workbench projection.

#[path = "activity/mod.rs"]
pub mod activity;
#[path = "animation_editor/mod.rs"]
pub mod animation_editor;
#[path = "asset_editor/mod.rs"]
pub mod asset_editor;
#[path = "binding/mod.rs"]
pub mod binding;
#[path = "binding_dispatch/mod.rs"]
pub mod binding_dispatch;
#[path = "component_registry/mod.rs"]
pub(crate) mod component_registry;
#[path = "control/mod.rs"]
pub mod control;
#[path = "curve/mod.rs"]
pub mod curve;
#[path = "graph/mod.rs"]
pub mod graph;
#[path = "host/mod.rs"]
pub mod host;
#[path = "layouts/mod.rs"]
pub(crate) mod layouts;
#[path = "material_editor/mod.rs"]
pub mod material_editor;
#[path = "operator/mod.rs"]
pub mod operator;
#[path = "preview_scene/mod.rs"]
pub mod preview_scene;
#[path = "reflection/mod.rs"]
mod reflection;
#[path = "retained/mod.rs"]
pub mod retained_host;
#[path = "sample_grid/mod.rs"]
pub(crate) mod sample_grid;
#[path = "settings/mod.rs"]
pub mod settings;
#[path = "template/mod.rs"]
pub(crate) mod template;
#[path = "runtime/mod.rs"]
pub mod template_runtime;
#[path = "timeline/mod.rs"]
pub mod timeline;
#[path = "timeline_strip/mod.rs"]
pub(crate) mod timeline_strip;
#[path = "v2_design_tokens/mod.rs"]
pub(crate) mod v2_design_tokens;
#[path = "weight_heatmap/mod.rs"]
pub(crate) mod weight_heatmap;
#[path = "widgets/mod.rs"]
pub(crate) mod widgets;
#[path = "workbench/mod.rs"]
pub mod workbench;

pub use activity::{
    ActivityDrawerSlotPreference, ActivityViewDescriptor, ActivityWindowDescriptor,
};
pub use reflection::{
    EditorActivityHost, EditorActivityKind, EditorActivityReflection, EditorDrawerReflectionModel,
    EditorFloatingWindowReflectionModel, EditorHostPageReflectionModel,
    EditorMenuItemReflectionModel, EditorUiReflectionAdapter, EditorWorkbenchReflectionModel,
};
