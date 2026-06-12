use bytemuck::{Pod, Zeroable};

pub(crate) const GPU_PRIMITIVE_DATA_STRIDE: usize = 80;
pub(crate) const GPU_PRIMITIVE_DATA_BOUNDS_CENTER_OFFSET: usize = 0;
pub(crate) const GPU_PRIMITIVE_DATA_BOUNDS_RADIUS_OFFSET: usize = 12;
pub(crate) const GPU_PRIMITIVE_DATA_TINT_OFFSET: usize = 16;
pub(crate) const GPU_PRIMITIVE_DATA_SHADOW_PARAMS_OFFSET: usize = 32;
pub(crate) const GPU_PRIMITIVE_DATA_MOTION_PARAMS_OFFSET: usize = 48;
pub(crate) const GPU_PRIMITIVE_DATA_FLAGS_OFFSET: usize = 64;
pub(crate) const GPU_PRIMITIVE_DATA_FIRST_INSTANCE_INDEX_OFFSET: usize = 68;
pub(crate) const GPU_PRIMITIVE_DATA_INSTANCE_COUNT_OFFSET: usize = 72;
pub(crate) const GPU_PRIMITIVE_DATA_PAYLOAD_SLOT_OFFSET: usize = 76;

pub(crate) const GPU_INSTANCE_DATA_STRIDE: usize = 144;
pub(crate) const GPU_INSTANCE_DATA_WORLD_FROM_LOCAL_OFFSET: usize = 0;
pub(crate) const GPU_INSTANCE_DATA_PREV_WORLD_FROM_LOCAL_OFFSET: usize = 64;
pub(crate) const GPU_INSTANCE_DATA_PRIMITIVE_INDEX_OFFSET: usize = 128;
pub(crate) const GPU_INSTANCE_DATA_FLAGS_OFFSET: usize = 132;
pub(crate) const GPU_INSTANCE_DATA_PAYLOAD_SLOT_OFFSET: usize = 136;
pub(crate) const GPU_INSTANCE_DATA_PAD0_OFFSET: usize = 140;

pub(crate) const GPU_SCENE_INVALID_PAYLOAD_SLOT: u32 = u32::MAX;
pub(crate) const GPU_PRIMITIVE_FLAG_VISIBLE: u32 = 1 << 0;
pub(crate) const GPU_PRIMITIVE_FLAG_CAST_SHADOWS: u32 = 1 << 1;
pub(crate) const GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM: u32 = 1 << 2;

/// Rust mirror for the WGSL `GpuPrimitiveData` storage-buffer element.
///
/// The explicit 16-byte stride is the shader ABI: instance-index rendering,
/// indirect submission, and later VG/HGI scene consumers must all observe the
/// same field offsets.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuPrimitiveData {
    pub(crate) bounds_center: [f32; 3],
    pub(crate) bounds_radius: f32,
    pub(crate) tint: [f32; 4],
    pub(crate) shadow_params: [f32; 4],
    pub(crate) motion_params: [f32; 4],
    pub(crate) flags: u32,
    pub(crate) first_instance_index: u32,
    pub(crate) instance_count: u32,
    pub(crate) payload_slot: u32,
}

impl GpuPrimitiveData {
    pub(crate) fn with_instance_span(first_instance_index: u32, instance_count: u32) -> Self {
        Self {
            first_instance_index,
            instance_count,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            ..Self::default()
        }
    }
}

/// Rust mirror for the WGSL `GpuInstanceData` storage-buffer element.
///
/// Transforms stay in instance data so a draw can advance by `instance_index`
/// without rebinding per-object model uniforms.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuInstanceData {
    pub(crate) world_from_local: [[f32; 4]; 4],
    pub(crate) prev_world_from_local: [[f32; 4]; 4],
    pub(crate) primitive_index: u32,
    pub(crate) flags: u32,
    pub(crate) payload_slot: u32,
    pub(crate) _pad0: u32,
}

impl GpuInstanceData {
    pub(crate) fn for_primitive(primitive_index: u32) -> Self {
        Self {
            world_from_local: identity_matrix(),
            prev_world_from_local: identity_matrix(),
            primitive_index,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            ..Self::default()
        }
    }
}

fn identity_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn render_gpu_scene_layout_matches_wgsl_offsets() {
        assert_eq!(size_of::<GpuPrimitiveData>(), GPU_PRIMITIVE_DATA_STRIDE);
        assert_eq!(
            offset_of!(GpuPrimitiveData, bounds_center),
            GPU_PRIMITIVE_DATA_BOUNDS_CENTER_OFFSET
        );
        assert_eq!(
            offset_of!(GpuPrimitiveData, bounds_radius),
            GPU_PRIMITIVE_DATA_BOUNDS_RADIUS_OFFSET
        );
        assert_eq!(
            offset_of!(GpuPrimitiveData, tint),
            GPU_PRIMITIVE_DATA_TINT_OFFSET
        );
        assert_eq!(
            offset_of!(GpuPrimitiveData, shadow_params),
            GPU_PRIMITIVE_DATA_SHADOW_PARAMS_OFFSET
        );
        assert_eq!(
            offset_of!(GpuPrimitiveData, motion_params),
            GPU_PRIMITIVE_DATA_MOTION_PARAMS_OFFSET
        );
        assert_eq!(
            offset_of!(GpuPrimitiveData, flags),
            GPU_PRIMITIVE_DATA_FLAGS_OFFSET
        );
        assert_eq!(
            offset_of!(GpuPrimitiveData, first_instance_index),
            GPU_PRIMITIVE_DATA_FIRST_INSTANCE_INDEX_OFFSET
        );
        assert_eq!(
            offset_of!(GpuPrimitiveData, instance_count),
            GPU_PRIMITIVE_DATA_INSTANCE_COUNT_OFFSET
        );
        assert_eq!(
            offset_of!(GpuPrimitiveData, payload_slot),
            GPU_PRIMITIVE_DATA_PAYLOAD_SLOT_OFFSET
        );

        assert_eq!(size_of::<GpuInstanceData>(), GPU_INSTANCE_DATA_STRIDE);
        assert_eq!(
            offset_of!(GpuInstanceData, world_from_local),
            GPU_INSTANCE_DATA_WORLD_FROM_LOCAL_OFFSET
        );
        assert_eq!(
            offset_of!(GpuInstanceData, prev_world_from_local),
            GPU_INSTANCE_DATA_PREV_WORLD_FROM_LOCAL_OFFSET
        );
        assert_eq!(
            offset_of!(GpuInstanceData, primitive_index),
            GPU_INSTANCE_DATA_PRIMITIVE_INDEX_OFFSET
        );
        assert_eq!(
            offset_of!(GpuInstanceData, flags),
            GPU_INSTANCE_DATA_FLAGS_OFFSET
        );
        assert_eq!(
            offset_of!(GpuInstanceData, payload_slot),
            GPU_INSTANCE_DATA_PAYLOAD_SLOT_OFFSET
        );
        assert_eq!(
            offset_of!(GpuInstanceData, _pad0),
            GPU_INSTANCE_DATA_PAD0_OFFSET
        );
    }
}
