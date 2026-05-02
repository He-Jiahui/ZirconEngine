use zircon_runtime::core::math::{Vec3, Vec4};
use zircon_runtime::core::resource::ResourceId;

#[derive(Clone, Copy, Debug)]
pub(super) struct HybridGiMaterialCaptureSeed {
    pub(super) base_color: Vec4,
    pub(super) emissive: Vec3,
    pub(super) metallic: f32,
    pub(super) roughness: f32,
    pub(super) double_sided: bool,
    pub(super) alpha_blend: bool,
    pub(super) alpha_cutoff: Option<f32>,
    pub(super) base_color_texture: Option<ResourceId>,
    pub(super) normal_texture: Option<ResourceId>,
    pub(super) metallic_roughness_texture: Option<ResourceId>,
    pub(super) occlusion_texture: Option<ResourceId>,
    pub(super) emissive_texture: Option<ResourceId>,
}

pub(super) trait HybridGiMaterialCaptureSource {
    fn material_capture_seed(&self, id: &ResourceId) -> Option<HybridGiMaterialCaptureSeed>;

    fn sample_texture_rgba(&self, id: Option<ResourceId>, uv: [f32; 2]) -> Option<Vec4>;
}
