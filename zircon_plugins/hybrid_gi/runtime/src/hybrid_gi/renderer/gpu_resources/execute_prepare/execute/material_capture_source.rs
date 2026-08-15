use zircon_runtime::core::math::{Vec3, Vec4};
use zircon_runtime::core::resource::ResourceId;

#[derive(Clone, Copy, Debug)]
pub(in crate::hybrid_gi::renderer) struct HybridGiMaterialCaptureSeed {
    pub(in crate::hybrid_gi::renderer) base_color: Vec4,
    pub(in crate::hybrid_gi::renderer) emissive: Vec3,
    pub(in crate::hybrid_gi::renderer) metallic: f32,
    pub(in crate::hybrid_gi::renderer) roughness: f32,
    pub(in crate::hybrid_gi::renderer) occlusion_strength: f32,
    pub(in crate::hybrid_gi::renderer) double_sided: bool,
    pub(in crate::hybrid_gi::renderer) alpha_blend: bool,
    pub(in crate::hybrid_gi::renderer) alpha_cutoff: Option<f32>,
    pub(in crate::hybrid_gi::renderer) cast_shadows: bool,
    pub(in crate::hybrid_gi::renderer) base_color_texture: Option<ResourceId>,
    pub(in crate::hybrid_gi::renderer) normal_texture: Option<ResourceId>,
    pub(in crate::hybrid_gi::renderer) metallic_roughness_texture: Option<ResourceId>,
    pub(in crate::hybrid_gi::renderer) occlusion_texture: Option<ResourceId>,
    pub(in crate::hybrid_gi::renderer) emissive_texture: Option<ResourceId>,
}

pub(in crate::hybrid_gi::renderer) trait HybridGiMaterialCaptureSource {
    fn material_capture_seed(&self, id: &ResourceId) -> Option<HybridGiMaterialCaptureSeed>;

    fn sample_texture_rgba(&self, id: Option<ResourceId>, uv: [f32; 2]) -> Option<Vec4>;
}
