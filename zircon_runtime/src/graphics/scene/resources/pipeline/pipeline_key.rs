use crate::core::resource::ResourceId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PipelineKey {
    pub(crate) shader_id: ResourceId,
    pub(crate) shader_revision: u64,
    pub(crate) double_sided: bool,
    pub(crate) alpha_blend: bool,
    pub(crate) alpha_mask: bool,
    pub(crate) alpha_cutoff_bits: Option<u32>,
    pub(crate) unlit: bool,
    pub(crate) has_base_color_texture: bool,
    pub(crate) has_normal_texture: bool,
    pub(crate) has_metallic_roughness_texture: bool,
    pub(crate) has_occlusion_texture: bool,
    pub(crate) has_emissive_texture: bool,
}

impl PipelineKey {
    pub(crate) fn is_transparent(&self) -> bool {
        self.alpha_blend
    }
}
