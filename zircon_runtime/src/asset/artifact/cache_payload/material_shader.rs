#[path = "material_shader/material.rs"]
mod material;
#[path = "material_shader/shader.rs"]
mod shader;

pub(super) use material::ArtifactCacheMaterialAsset;
pub(super) use shader::ArtifactCacheShaderAsset;
