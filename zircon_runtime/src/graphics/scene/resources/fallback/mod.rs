mod create_fallback_texture;
mod fallback_shader_uri;

#[cfg(test)]
pub(in crate::graphics::scene::resources) use create_fallback_texture::{
    create_fallback_normal_texture, create_fallback_texture,
};
pub(in crate::graphics::scene::resources) use create_fallback_texture::{
    create_fallback_normal_texture_from_system, create_fallback_texture_from_system,
};
pub(crate) use fallback_shader_uri::fallback_shader_uri;
