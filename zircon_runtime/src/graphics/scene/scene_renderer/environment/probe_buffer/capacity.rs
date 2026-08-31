pub(super) const MAX_REFLECTION_PROBES: usize = 64;
pub(super) const REFLECTION_PROBE_FACE_COUNT: u32 = 6;
pub(super) const REFLECTION_PROBE_FACE_SIZE: u32 = 128;
pub(super) const REFLECTION_PROBE_MIP_COUNT: u32 = 8;
pub(in crate::graphics::scene::scene_renderer) const PLANAR_REFLECTION_TEXTURE_SIZE: u32 = 1024;
pub(in crate::graphics::scene::scene_renderer) const PLANAR_REFLECTION_MIP_COUNT: u32 = 11;

const ENVIRONMENT_PREVIEW_PLACEHOLDER_PROBE_COUNT: usize = 1;
const ENVIRONMENT_PREVIEW_PLACEHOLDER_FACE_SIZE: u32 = 1;
const ENVIRONMENT_PREVIEW_PLACEHOLDER_MIP_COUNT: u32 = 1;
const ENVIRONMENT_PREVIEW_PLACEHOLDER_PLANAR_TEXTURE_SIZE: u32 = 1;
const ENVIRONMENT_PREVIEW_PLACEHOLDER_PLANAR_MIP_COUNT: u32 = 1;
const REFLECTION_PROBE_CAPTURE_SPARE_SLOT_COUNT: usize = 1;

#[derive(Clone, Copy)]
pub(super) struct ReflectionProbeResourceCapacity {
    pub(super) probe_count: usize,
    pub(super) cubemap_slot_count: usize,
    pub(super) cubemap_face_size: u32,
    pub(super) cubemap_mip_count: u32,
    pub(super) planar_texture_size: u32,
    pub(super) planar_mip_count: u32,
}

impl ReflectionProbeResourceCapacity {
    pub(super) const FULL: Self = Self {
        probe_count: MAX_REFLECTION_PROBES,
        cubemap_slot_count: MAX_REFLECTION_PROBES + REFLECTION_PROBE_CAPTURE_SPARE_SLOT_COUNT,
        cubemap_face_size: REFLECTION_PROBE_FACE_SIZE,
        cubemap_mip_count: REFLECTION_PROBE_MIP_COUNT,
        planar_texture_size: PLANAR_REFLECTION_TEXTURE_SIZE,
        planar_mip_count: PLANAR_REFLECTION_MIP_COUNT,
    };

    pub(super) const ENVIRONMENT_PREVIEW_PLACEHOLDER: Self = Self {
        probe_count: ENVIRONMENT_PREVIEW_PLACEHOLDER_PROBE_COUNT,
        cubemap_slot_count: ENVIRONMENT_PREVIEW_PLACEHOLDER_PROBE_COUNT
            + REFLECTION_PROBE_CAPTURE_SPARE_SLOT_COUNT,
        cubemap_face_size: ENVIRONMENT_PREVIEW_PLACEHOLDER_FACE_SIZE,
        cubemap_mip_count: ENVIRONMENT_PREVIEW_PLACEHOLDER_MIP_COUNT,
        planar_texture_size: ENVIRONMENT_PREVIEW_PLACEHOLDER_PLANAR_TEXTURE_SIZE,
        planar_mip_count: ENVIRONMENT_PREVIEW_PLACEHOLDER_PLANAR_MIP_COUNT,
    };
}
