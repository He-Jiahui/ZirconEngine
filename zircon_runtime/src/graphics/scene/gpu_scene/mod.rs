mod binding;
mod bindless_material_payload;
mod direct_upload;
mod gpu_scene;
mod id_allocator;
mod journal_consumer;
mod layout;
mod morph;
mod prepared_upload;
mod prev_morph_weights;
mod prev_skinned_palette;
mod prev_skinned_source;
mod prev_transform;
mod skinned_palette_arena;
mod staged_upload;
mod staging_ring;
mod update_queue;
mod upload;
mod virtual_geometry;

pub(crate) use binding::gpu_scene_bind_group_layout_entries;
pub(in crate::graphics::scene) use binding::{
    GPU_SCENE_INSTANCE_DATA_BINDING, GPU_SCENE_LIGHT_DATA_BINDING,
    GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING, GPU_SCENE_PRIMITIVE_DATA_BINDING,
    GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
};

pub(crate) use bindless_material_payload::{
    BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT, GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE,
    GpuBindlessMaterialPayload,
};
pub(crate) use gpu_scene::{
    GpuScene, GpuSceneEntry, GpuSceneStats, GpuSceneUploadPath, GpuSceneUploadReport,
};
pub(crate) use journal_consumer::{
    GpuSceneJournalApplyPlan, GpuSceneJournalConsumer, GpuSceneJournalConsumerError,
    GpuSceneJournalReprojectionError, GpuSceneJournalReprojectionPlan,
    GpuSceneJournalReprojectionPreflightError, GpuSceneJournalResidentWrite,
    GpuSceneJournalResidentWriteKind, GpuSceneJournalRetirement, GpuSceneJournalSlotMutation,
    GpuSceneJournalTransactionCommit, GpuSceneJournalTransactionError,
};
pub(crate) use layout::{
    GPU_INSTANCE_DATA_STRIDE, GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM,
    GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM, GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT,
    GPU_INSTANCE_FLAG_NON_ORTHOGONAL_TRANSFORM, GPU_MORPH_DELTA_STRIDE, GPU_MORPH_PAYLOAD_STRIDE,
    GPU_MORPH_WEIGHT_STRIDE, GPU_PRIMITIVE_DATA_STRIDE, GPU_PRIMITIVE_FLAG_CAST_SHADOWS,
    GPU_PRIMITIVE_FLAG_FORCE_HZB_VISIBLE, GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
    GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT,
    GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
    GpuInstanceData, GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GpuPrimitiveData,
    GpuVirtualGeometryClusterWord, GpuVirtualGeometryPage,
};
pub(crate) use morph::{GpuSceneMorphUploadReport, GpuScenePreparedMorphUpload};
pub(crate) use prepared_upload::GpuScenePreparedUpload;
pub(crate) use prev_skinned_palette::GpuSceneSkinnedJointPaletteState;
pub(crate) use prev_skinned_source::GpuSceneSkinnedGpuSourceState;
pub(crate) use virtual_geometry::{
    GpuScenePreparedVirtualGeometryUpload, GpuSceneVirtualGeometryUploadReport,
};
