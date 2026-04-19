use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::resource::ResourceId;

use super::super::prepared::{PreparedMaterial, PreparedModel, PreparedShader, PreparedTexture};
use super::super::GpuTextureResource;

pub(crate) struct ResourceStreamer {
    pub(super) asset_manager: Arc<ProjectAssetManager>,
    pub(super) models: HashMap<ResourceId, PreparedModel>,
    pub(super) materials: HashMap<ResourceId, PreparedMaterial>,
    pub(super) textures: HashMap<ResourceId, PreparedTexture>,
    pub(super) shaders: HashMap<ResourceId, PreparedShader>,
    pub(super) fallback_texture: Arc<GpuTextureResource>,
}
