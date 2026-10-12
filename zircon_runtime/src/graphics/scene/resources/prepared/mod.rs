#[path = "geometry_deformation.rs"]
mod prepared_geometry_deformation;
#[path = "prepared_material.rs"]
mod prepared_material;
#[path = "prepared_mesh.rs"]
mod prepared_mesh;
#[path = "prepared_mesh_sdf.rs"]
mod prepared_mesh_sdf;
#[path = "prepared_model.rs"]
mod prepared_model;
#[path = "output_target_texture.rs"]
mod prepared_output_target_texture;
#[path = "post_process_lut_texture.rs"]
mod prepared_post_process_lut_texture;
#[path = "prepared_shader.rs"]
mod prepared_shader;
#[path = "prepared_texture.rs"]
mod prepared_texture;

pub(in crate::graphics::scene::resources) use prepared_geometry_deformation::PreparedGeometryDeformation;
pub(in crate::graphics::scene::resources) use prepared_material::{
    PreparedMaterial, PreparedMaterialBundle, PreparedMaterialCandidateIdentity,
    PreparedMaterialDependency, PreparedMaterialShaderDependency, PreparedMaterialTextureBinding,
    PreparedMaterialTextureDependency, PreparedMaterialTextureResource, PreparedMaterialTextureSet,
    RejectedPreparedMaterialCandidate,
};
pub(in crate::graphics::scene::resources) use prepared_mesh::PreparedMesh;
pub(in crate::graphics::scene::resources) use prepared_mesh_sdf::mesh_sdf_seed_from_primitives;
pub(in crate::graphics::scene::resources) use prepared_model::PreparedModel;
pub(in crate::graphics::scene::resources) use prepared_output_target_texture::PreparedOutputTargetTexture;
pub(in crate::graphics::scene::resources) use prepared_post_process_lut_texture::PreparedPostProcessLutTexture;
pub(in crate::graphics::scene::resources) use prepared_shader::PreparedShader;
pub(in crate::graphics::scene::resources) use prepared_texture::PreparedTexture;
