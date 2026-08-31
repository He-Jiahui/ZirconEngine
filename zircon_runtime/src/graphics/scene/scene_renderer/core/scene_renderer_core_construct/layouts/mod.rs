pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) mod create_material_texture_bind_group_layout;
pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) mod create_texture_bind_group_layout;

pub(super) use create_material_texture_bind_group_layout::create_material_texture_bind_group_layout;
pub(in crate::graphics::scene::scene_renderer) use create_material_texture_bind_group_layout::material_texture_bind_group_layout_entries;
pub(super) use create_texture_bind_group_layout::create_texture_bind_group_layout;
