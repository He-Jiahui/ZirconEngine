mod binding;
mod bindless_material_payload;
mod gpu_scene;
mod id_allocator;
mod layout;
mod morph;
mod prev_morph_weights;
mod prev_skinned_palette;
mod prev_skinned_source;
mod prev_transform;
mod skinned_palette_buffer_slots;
mod staged_upload;
mod staging_ring;
mod update_queue;
mod upload;
mod virtual_geometry;

pub(crate) use gpu_scene::{
    GpuScene, GpuSceneEntry, GpuSceneStats, GpuSceneUploadPath, GpuSceneUploadReport,
};
pub(crate) use bindless_material_payload::{
    BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT, GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE,
    GpuBindlessMaterialPayload,
};
pub(crate) use layout::{
    GpuInstanceData, GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GpuPrimitiveData,
    GpuVirtualGeometryClusterWord, GpuVirtualGeometryPage, GPU_INSTANCE_DATA_STRIDE,
    GPU_MORPH_DELTA_STRIDE, GPU_MORPH_PAYLOAD_STRIDE, GPU_MORPH_WEIGHT_STRIDE,
    GPU_PRIMITIVE_DATA_STRIDE, GPU_PRIMITIVE_FLAG_CAST_SHADOWS,
    GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM, GPU_PRIMITIVE_FLAG_VISIBLE,
    GPU_SCENE_INVALID_PAYLOAD_SLOT, GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX,
    GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
};
pub(crate) use morph::GpuSceneMorphUploadReport;
pub(crate) use prev_skinned_palette::GpuSceneSkinnedJointPaletteState;
pub(crate) use prev_skinned_source::GpuSceneSkinnedGpuSourceState;
pub(crate) use virtual_geometry::GpuSceneVirtualGeometryUploadReport;
