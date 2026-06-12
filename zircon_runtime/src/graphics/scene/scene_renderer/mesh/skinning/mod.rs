mod joint_palette_uniform;

pub(in crate::graphics::scene::scene_renderer) use joint_palette_uniform::{
    create_empty_skinned_joint_palette_buffer, skinned_joint_palette_uniform_min_binding_size,
};
pub(in crate::graphics::scene::scene_renderer::mesh) use joint_palette_uniform::{
    SkinnedMeshJointPaletteUniform, SKINNED_MESH_MAX_JOINT_MATRICES,
};
