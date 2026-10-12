#[path = "gltf_primitive_fixtures/animation.rs"]
mod animation;
#[path = "gltf_primitive_fixtures/basic.rs"]
mod basic;
#[path = "gltf_primitive_fixtures/materials.rs"]
mod materials;
#[path = "gltf_primitive_fixtures/vertex_channels.rs"]
mod vertex_channels;

pub(super) use animation::write_node_animation_gltf;
pub(super) use basic::{write_line_gltf, write_triangle_gltf};
pub(super) use materials::{write_texture_transform_triangle_gltf, write_two_primitive_gltf};
pub(super) use vertex_channels::{
    write_skinned_triangle_gltf, write_tangent_color_triangle_gltf, write_uv_channel_triangle_gltf,
};
