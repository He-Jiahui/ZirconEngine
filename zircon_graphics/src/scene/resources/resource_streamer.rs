use std::collections::HashMap;
use std::sync::Arc;

use zircon_asset::ProjectAssetManager;
use zircon_resource::ResourceId;

use super::gpu_texture_resource::GpuTextureResource;
use super::prepared_material::PreparedMaterial;
use super::prepared_model::PreparedModel;
use super::prepared_shader::PreparedShader;
use super::prepared_texture::PreparedTexture;

pub(crate) struct ResourceStreamer {
    pub(super) asset_manager: Arc<ProjectAssetManager>,
    pub(super) models: HashMap<ResourceId, PreparedModel>,
    pub(super) materials: HashMap<ResourceId, PreparedMaterial>,
    pub(super) textures: HashMap<ResourceId, PreparedTexture>,
    pub(super) shaders: HashMap<ResourceId, PreparedShader>,
    pub(super) fallback_texture: Arc<GpuTextureResource>,
}
