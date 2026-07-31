use std::sync::Arc;

use crate::asset::TextureUploadSupport;
use crate::core::resource::{ResourceId, ResourceLocator};

use super::super::GpuMaterialUniformResource;
use super::super::MaterialRuntime;

pub(in crate::graphics::scene::resources) struct PreparedMaterial {
    pub(in crate::graphics::scene::resources) revision: Option<u64>,
    pub(in crate::graphics::scene::resources) texture_dependencies:
        Vec<PreparedMaterialTextureDependency>,
    pub(in crate::graphics::scene::resources) texture_support: TextureUploadSupport,
    pub(in crate::graphics::scene::resources) runtime: MaterialRuntime,
    pub(in crate::graphics::scene::resources) uniform: Arc<GpuMaterialUniformResource>,
    pub(in crate::graphics::scene::resources) standard_uniform: Arc<GpuMaterialUniformResource>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::resources) struct PreparedMaterialTextureDependency {
    pub(in crate::graphics::scene::resources) locator: ResourceLocator,
    pub(in crate::graphics::scene::resources) id: Option<ResourceId>,
    pub(in crate::graphics::scene::resources) revision: Option<u64>,
    pub(in crate::graphics::scene::resources) upload_unsupported_reason: Option<String>,
}
