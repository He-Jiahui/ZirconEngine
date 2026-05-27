mod arranged;
mod component_state;
mod diagnostics;
mod ecs_projection;
mod focus;
mod frame_hit_test;
mod input;
mod interaction_gate;
mod node_pool;
mod property_mutation;
mod reflection_snapshot;
mod render;
mod surface;
mod timeline;

pub use crate::ui::text::layout_text;
pub(crate) use arranged::{
    arranged_bubble_route, arranged_effective_input_policy, build_arranged_tree,
    is_arranged_child_hit_path_visible, is_arranged_render_visible,
};
pub use component_state::UiSurfaceComponentStateStore;
pub use diagnostics::{
    debug_surface_frame, debug_surface_frame_for_pick, debug_surface_frame_for_selection,
    debug_surface_frame_with_options,
};
pub use frame_hit_test::{
    debug_hit_test_surface_frame, debug_hit_test_surface_frame_with_query, hit_test_surface_frame,
    hit_test_surface_frame_with_query,
};
pub(crate) use input::text_input_constraints_for_node;
pub use input::UiSurfaceInputState;
pub(crate) use interaction_gate::{ui_surface_effective_disabled, ui_surface_node_disabled};
pub use node_pool::{UiSurfaceNodePool, UiSurfaceNodePoolReport};
pub use property_mutation::{
    UiPropertyMutationReport, UiPropertyMutationRequest, UiPropertyMutationStatus,
};
pub use reflection_snapshot::reflector_snapshot;
pub(crate) use render::measure_text;
pub use render::{extract_ui_render_tree, extract_ui_render_tree_from_arranged};
pub use surface::{UiSurface, UiSurfaceRebuildReport};
pub use timeline::UiDebugTimelineStore;
