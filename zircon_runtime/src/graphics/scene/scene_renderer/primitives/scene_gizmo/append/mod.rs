mod append_camera_icon_fallback_lines;
mod append_directional_light_icon_fallback_lines;
mod append_icon_fallback_lines;
mod append_wire_shape;

pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo) use append_icon_fallback_lines::{
    append_icon_fallback_lines, icon_fallback_vertex_capacity,
};
pub(in crate::graphics::scene::scene_renderer::primitives::scene_gizmo) use append_wire_shape::{
    append_wire_shape, wire_shape_vertex_capacity,
};
