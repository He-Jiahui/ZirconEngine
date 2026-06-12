use std::sync::Arc;

use crate::graphics::scene::resources::GpuTextureResource;

#[derive(Clone)]
pub(crate) struct MaterialTextureSet {
    pub(crate) base_color: Arc<GpuTextureResource>,
    pub(crate) normal: Arc<GpuTextureResource>,
    pub(crate) metallic_roughness: Arc<GpuTextureResource>,
    pub(crate) occlusion: Arc<GpuTextureResource>,
    pub(crate) emissive: Arc<GpuTextureResource>,
}

impl MaterialTextureSet {
    pub(crate) fn new(
        base_color: Arc<GpuTextureResource>,
        normal: Arc<GpuTextureResource>,
        metallic_roughness: Arc<GpuTextureResource>,
        occlusion: Arc<GpuTextureResource>,
        emissive: Arc<GpuTextureResource>,
    ) -> Self {
        Self {
            base_color,
            normal,
            metallic_roughness,
            occlusion,
            emissive,
        }
    }
}
