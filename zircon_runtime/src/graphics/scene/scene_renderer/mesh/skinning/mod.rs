mod joint_palette_storage;

#[cfg(test)]
pub(in crate::graphics::scene::scene_renderer::mesh) use joint_palette_storage::SKINNED_MESH_MAX_JOINT_MATRICES;
pub(in crate::graphics::scene) use joint_palette_storage::SkinnedMeshJointPaletteStorage;
pub(in crate::graphics::scene::scene_renderer) use joint_palette_storage::{
    create_empty_skinned_joint_palette_arena_buffer, skinned_joint_palette_arena_min_binding_size,
};
