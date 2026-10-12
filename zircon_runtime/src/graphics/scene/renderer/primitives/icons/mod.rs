#[path = "build/mod.rs"]
mod build;
#[path = "size/mod.rs"]
mod size;

pub(crate) use build::build_icon_quad_vertices;
pub(in crate::graphics::scene::scene_renderer::primitives) use size::icon_world_size;
