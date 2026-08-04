use bytemuck::{Pod, Zeroable};

pub(crate) const GPU_PRIMITIVE_DATA_STRIDE: usize = 96;
pub(crate) const GPU_PRIMITIVE_DATA_BOUNDS_CENTER_OFFSET: usize = 0;
pub(crate) const GPU_PRIMITIVE_DATA_BOUNDS_RADIUS_OFFSET: usize = 12;
pub(crate) const GPU_PRIMITIVE_DATA_TINT_OFFSET: usize = 16;
pub(crate) const GPU_PRIMITIVE_DATA_SHADOW_PARAMS_OFFSET: usize = 32;
pub(crate) const GPU_PRIMITIVE_DATA_MOTION_PARAMS_OFFSET: usize = 48;
pub(crate) const GPU_PRIMITIVE_DATA_FLAGS_OFFSET: usize = 64;
pub(crate) const GPU_PRIMITIVE_DATA_FIRST_INSTANCE_INDEX_OFFSET: usize = 68;
pub(crate) const GPU_PRIMITIVE_DATA_INSTANCE_COUNT_OFFSET: usize = 72;
pub(crate) const GPU_PRIMITIVE_DATA_PAYLOAD_SLOT_OFFSET: usize = 76;
pub(crate) const GPU_PRIMITIVE_DATA_MATERIAL_PAYLOAD_SLOT_OFFSET: usize = 80;

pub(crate) const GPU_INSTANCE_DATA_STRIDE: usize = 176;
pub(crate) const GPU_INSTANCE_DATA_WORLD_FROM_LOCAL_OFFSET: usize = 0;
pub(crate) const GPU_INSTANCE_DATA_PREV_WORLD_FROM_LOCAL_OFFSET: usize = 64;
pub(crate) const GPU_INSTANCE_DATA_PRIMITIVE_INDEX_OFFSET: usize = 128;
pub(crate) const GPU_INSTANCE_DATA_FLAGS_OFFSET: usize = 132;
pub(crate) const GPU_INSTANCE_DATA_PAYLOAD_SLOT_OFFSET: usize = 136;
pub(crate) const GPU_INSTANCE_DATA_MORPH_PAYLOAD_SLOT_OFFSET: usize = 140;
pub(crate) const GPU_INSTANCE_DATA_LIGHTMAP_UV_RECT_OFFSET: usize = 144;
pub(crate) const GPU_INSTANCE_DATA_LIGHTMAP_PARAMS_OFFSET: usize = 160;

pub(crate) const GPU_MORPH_PAYLOAD_STRIDE: usize = 16;
pub(crate) const GPU_MORPH_DELTA_STRIDE: usize = 16;
pub(crate) const GPU_MORPH_WEIGHT_STRIDE: usize = 4;

pub(crate) const GPU_VIRTUAL_GEOMETRY_PAGE_STRIDE: usize = 16;
pub(crate) const GPU_VIRTUAL_GEOMETRY_PAGE_CLUSTER_BASE_WORD_OFFSET: usize = 0;
pub(crate) const GPU_VIRTUAL_GEOMETRY_PAGE_VERTEX_COUNT_OFFSET: usize = 4;
pub(crate) const GPU_VIRTUAL_GEOMETRY_PAGE_PAGE_ID_OFFSET: usize = 8;
pub(crate) const GPU_VIRTUAL_GEOMETRY_PAGE_FLAGS_OFFSET: usize = 12;
pub(crate) const GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE: usize = 16;

pub(crate) const GPU_SCENE_INVALID_PAYLOAD_SLOT: u32 = u32::MAX;
pub(crate) const GPU_PRIMITIVE_FLAG_VISIBLE: u32 = 1 << 0;
pub(crate) const GPU_PRIMITIVE_FLAG_CAST_SHADOWS: u32 = 1 << 1;
pub(crate) const GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM: u32 = 1 << 2;
pub(crate) const GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT: u32 = 1 << 0;
pub(crate) const GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX: u32 = 4;

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
    pub(crate) material_payload_slot: u32,
    // Explicitly model the storage-array stride tail so this type remains Pod.
    pub(crate) material_payload_padding: [u32; 3],
}

impl GpuPrimitiveData {
    pub(crate) fn with_instance_span(first_instance_index: u32, instance_count: u32) -> Self {
        Self {
            first_instance_index,
            instance_count,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            material_payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
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
    pub(crate) morph_payload_slot: u32,
    pub(crate) lightmap_uv_rect: [f32; 4],
    pub(crate) lightmap_params: [u32; 4],
}

impl GpuInstanceData {
    pub(crate) fn for_primitive(primitive_index: u32) -> Self {
        Self {
            world_from_local: identity_matrix(),
            prev_world_from_local: identity_matrix(),
            primitive_index,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            morph_payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            ..Self::default()
        }
    }

    pub(crate) fn set_lightmap(
        &mut self,
        slot: crate::core::framework::render::LightmapInstanceSlot,
        light_set_generation: u64,
    ) {
        self.lightmap_uv_rect = slot.uv_rect.to_array();
        self.lightmap_params = [
            slot.atlas_page,
            1,
            light_set_generation as u32,
            (light_set_generation >> 32) as u32,
        ];
    }
}

/// A 16-byte morph payload header consumed by `zr_morph_payloads`.
///
/// `delta_base` indexes `GpuMorphDelta` rows, `weight_base` indexes
/// `GpuMorphWeight` rows, and the shader walks `target_count * vertex_count`
/// rows using the incoming vertex index.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub(crate) struct GpuMorphPayload {
    pub(crate) delta_base: u32,
    pub(crate) weight_base: u32,
    pub(crate) vertex_count: u32,
    pub(crate) target_count: u32,
}

impl GpuMorphPayload {
    pub(crate) const fn new(
        delta_base: u32,
        weight_base: u32,
        vertex_count: u32,
        target_count: u32,
    ) -> Self {
        Self {
            delta_base,
            weight_base,
            vertex_count,
            target_count,
        }
    }
}

/// A 16-byte morph target delta row consumed by `zr_morph_deltas`.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuMorphDelta {
    pub(crate) values: [f32; 4],
}

impl GpuMorphDelta {
    pub(crate) const fn position_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            values: [x, y, z, 1.0],
        }
    }

    pub(crate) const fn normal_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            values: [x, y, z, 1.0],
        }
    }

    pub(crate) const fn tangent_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            values: [x, y, z, 1.0],
        }
    }

    pub(crate) const fn color_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            values: [r, g, b, a],
        }
    }
}

/// A scalar morph weight row consumed by `zr_morph_weights`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuMorphWeight {
    pub(crate) value: f32,
}

impl GpuMorphWeight {
    pub(crate) const fn new(value: f32) -> Self {
        Self { value }
    }
}

/// Rust mirror for one VG page-table row consumed by `zr_virtual_geometry_pages`.
///
/// `cluster_base_word` indexes `GpuVirtualGeometryClusterWord` rows, while
/// `vertex_count` guards shader fetch. Pages without resident payloads keep
/// `vertex_count == 0` so WGSL falls back to mesh vertex input.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub(crate) struct GpuVirtualGeometryPage {
    pub(crate) cluster_base_word: u32,
    pub(crate) vertex_count: u32,
    pub(crate) page_id: u32,
    pub(crate) flags: u32,
}

impl GpuVirtualGeometryPage {
    pub(crate) const fn new(
        cluster_base_word: u32,
        vertex_count: u32,
        page_id: u32,
        flags: u32,
    ) -> Self {
        Self {
            cluster_base_word,
            vertex_count,
            page_id,
            flags,
        }
    }
}

/// A 16-byte VG cluster payload word consumed by `zr_virtual_geometry_clusters`.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuVirtualGeometryClusterWord {
    pub(crate) values: [f32; 4],
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
        assert_eq!(
            offset_of!(GpuPrimitiveData, material_payload_slot),
            GPU_PRIMITIVE_DATA_MATERIAL_PAYLOAD_SLOT_OFFSET
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
            offset_of!(GpuInstanceData, morph_payload_slot),
            GPU_INSTANCE_DATA_MORPH_PAYLOAD_SLOT_OFFSET
        );
        assert_eq!(
            offset_of!(GpuInstanceData, lightmap_uv_rect),
            GPU_INSTANCE_DATA_LIGHTMAP_UV_RECT_OFFSET
        );
        assert_eq!(
            offset_of!(GpuInstanceData, lightmap_params),
            GPU_INSTANCE_DATA_LIGHTMAP_PARAMS_OFFSET
        );

        assert_eq!(size_of::<GpuMorphPayload>(), GPU_MORPH_PAYLOAD_STRIDE);
        assert_eq!(size_of::<GpuMorphDelta>(), GPU_MORPH_DELTA_STRIDE);
        assert_eq!(size_of::<GpuMorphWeight>(), GPU_MORPH_WEIGHT_STRIDE);

        assert_eq!(
            size_of::<GpuVirtualGeometryPage>(),
            GPU_VIRTUAL_GEOMETRY_PAGE_STRIDE
        );
        assert_eq!(
            offset_of!(GpuVirtualGeometryPage, cluster_base_word),
            GPU_VIRTUAL_GEOMETRY_PAGE_CLUSTER_BASE_WORD_OFFSET
        );
        assert_eq!(
            offset_of!(GpuVirtualGeometryPage, vertex_count),
            GPU_VIRTUAL_GEOMETRY_PAGE_VERTEX_COUNT_OFFSET
        );
        assert_eq!(
            offset_of!(GpuVirtualGeometryPage, page_id),
            GPU_VIRTUAL_GEOMETRY_PAGE_PAGE_ID_OFFSET
        );
        assert_eq!(
            offset_of!(GpuVirtualGeometryPage, flags),
            GPU_VIRTUAL_GEOMETRY_PAGE_FLAGS_OFFSET
        );
        assert_eq!(
            size_of::<GpuVirtualGeometryClusterWord>(),
            GPU_VIRTUAL_GEOMETRY_CLUSTER_WORD_STRIDE
        );
    }

    #[test]
    fn render_gpu_scene_wgsl_primitive_tail_preserves_the_rust_storage_stride() {
        let source = include_str!("../scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");

        assert!(source.contains("material_payload_padding_0: u32,"));
        assert!(source.contains("material_payload_padding_1: u32,"));
        assert!(source.contains("material_payload_padding_2: u32,"));
        assert!(!source.contains("material_payload_padding: vec3<u32>"));
    }

    #[test]
    fn render_gpu_scene_lightmap_slot_preserves_uv_page_and_generation() {
        let mut instance = GpuInstanceData::default();
        let slot = crate::core::framework::render::LightmapInstanceSlot {
            atlas_page: 7,
            uv_rect: glam::Vec4::new(0.25, 0.5, 0.125, 0.25),
        };

        instance.set_lightmap(slot, 0x0123_4567_89ab_cdef);

        assert_eq!(instance.lightmap_uv_rect, [0.25, 0.5, 0.125, 0.25]);
        assert_eq!(instance.lightmap_params, [7, 1, 0x89ab_cdef, 0x0123_4567]);
    }
}
