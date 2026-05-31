use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::resource::ResourceId;

use super::super::prepared::{
    PreparedMaterial, PreparedMesh, PreparedModel, PreparedShader, PreparedTexture,
};
use super::super::{GpuMaterialUniformResource, GpuTextureResource};

pub(crate) struct ResourceStreamer {
    pub(super) asset_manager: Arc<ProjectAssetManager>,
    pub(super) material_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) models: HashMap<ResourceId, PreparedModel>,
    pub(super) meshes: HashMap<ResourceId, PreparedMesh>,
    pub(super) materials: HashMap<ResourceId, PreparedMaterial>,
    pub(super) textures: HashMap<ResourceId, PreparedTexture>,
    pub(super) shaders: HashMap<ResourceId, PreparedShader>,
    pub(super) fallback_texture: Arc<GpuTextureResource>,
    pub(super) fallback_material_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) last_material_count: usize,
    pub(super) last_material_ready_count: usize,
    pub(super) last_material_fallback_count: usize,
    pub(super) last_material_validation_error_count: usize,
    pub(super) last_material_diagnostic_count: usize,
    pub(super) last_sprite_count: usize,
    pub(super) last_sprite_ready_count: usize,
    pub(super) last_sprite_texture_fallback_count: usize,
}
