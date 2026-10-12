#[path = "activity_actions/mod.rs"]
mod activity_actions;
#[path = "activity_collection.rs"]
mod activity_collection;
#[path = "activity_descriptors.rs"]
mod activity_descriptors;
#[path = "animation_route.rs"]
mod animation_route;
#[path = "asset_route.rs"]
mod asset_route;
#[path = "docking_route.rs"]
mod docking_route;
#[path = "draft_route.rs"]
mod draft_route;
#[path = "drawer_slot_preference.rs"]
mod drawer_slot_preference;
#[path = "inspector_route.rs"]
mod inspector_route;
#[path = "model_build.rs"]
mod model_build;
#[path = "name_mapping.rs"]
mod name_mapping;
#[path = "route_registration/mod.rs"]
mod route_registration;
#[path = "transient_ui_state.rs"]
mod transient_ui_state;
#[path = "viewport_route/mod.rs"]
mod viewport_route;
#[path = "widget_reflector.rs"]
mod widget_reflector;

pub use activity_descriptors::activity_descriptors_from_views;
pub use model_build::build_workbench_reflection_model;
pub use route_registration::register_workbench_reflection_routes;
pub(crate) use transient_ui_state::{apply_transient_projection, EditorTransientUiState};
pub use widget_reflector::{
    WorkbenchWidgetReflectorError, WorkbenchWidgetReflectorModel, WorkbenchWidgetReflectorRow,
    WorkbenchWidgetReflectorSelection,
};
