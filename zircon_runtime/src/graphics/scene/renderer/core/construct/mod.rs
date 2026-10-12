#[path = "construct/mod.rs"]
mod construct;
#[path = "layouts/mod.rs"]
mod layouts;
#[path = "../scene_bind_group_bundle/mod.rs"]
mod scene_bind_group_bundle;

pub(in crate::graphics::scene::scene_renderer) use layouts::material_texture_bind_group_layout_entries;
