#[path = "../../viewport_surface/viewport_surface.rs"]
mod viewport_surface;

pub(in crate::graphics::runtime::render_framework) use viewport_surface::{
    bind_viewport_surface, unbind_viewport_surface,
};
