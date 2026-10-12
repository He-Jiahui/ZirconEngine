#[path = "../../../append/camera_icon_fallback_lines.rs"]
mod append_camera_icon_fallback_lines;
#[path = "../../../../../append/directional_light_icon_fallback_lines.rs"]
mod append_directional_light_icon_fallback_lines;
#[path = "../../../append/icon_fallback_lines.rs"]
mod append_icon_fallback_lines;
#[path = "../../../append/append_wire_shape.rs"]
mod append_wire_shape;

pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo) use append_icon_fallback_lines::{
    append_icon_fallback_lines, icon_fallback_vertex_capacity,
};
pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo) use append_wire_shape::{
    append_wire_shape, wire_shape_vertex_capacity,
};
