//! 维护场景表及变形、几何辅助表的 CPU 镜像、上传准备和成功帧后的历史状态。
//! 绑定槽与行布局须和共享 WGSL 的 group 3 一致；资源及提交边界由各子模块协作维护。

#[path = "binding.rs"]
mod binding;
#[path = "bindless_material_payload.rs"]
mod bindless_material_payload;
#[path = "direct_upload.rs"]
mod direct_upload;
#[path = "gpu_scene.rs"]
mod gpu_scene;
#[path = "id_allocator.rs"]
mod id_allocator;
#[path = "journal_consumer.rs"]
mod journal_consumer;
#[path = "layout.rs"]
mod layout;
#[path = "morph.rs"]
mod morph;
#[path = "prepared_upload.rs"]
mod prepared_upload;
#[path = "prev_morph_weights.rs"]
mod prev_morph_weights;
#[path = "prev_skinned_palette.rs"]
mod prev_skinned_palette;
#[path = "prev_skinned_source.rs"]
mod prev_skinned_source;
#[path = "prev_transform.rs"]
mod prev_transform;
#[path = "skinned_palette_arena.rs"]
mod skinned_palette_arena;
#[path = "staged_upload.rs"]
mod staged_upload;
#[path = "staging_ring.rs"]
mod staging_ring;
#[path = "update_queue.rs"]
mod update_queue;
#[path = "upload.rs"]
mod upload;
#[path = "virtual_geometry.rs"]
mod virtual_geometry;

pub(crate) use binding::gpu_scene_bind_group_layout_entries;
pub(in crate::graphics::scene) use binding::{
    GPU_SCENE_INSTANCE_DATA_BINDING, GPU_SCENE_LIGHT_DATA_BINDING,
    GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING, GPU_SCENE_PRIMITIVE_DATA_BINDING,
    GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
};

pub(crate) use bindless_material_payload::{
    GpuBindlessMaterialPayload, BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT,
    GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE,
};
pub(crate) use gpu_scene::{
    GpuScene, GpuSceneEntry, GpuSceneStats, GpuSceneUploadPath, GpuSceneUploadReport,
};
pub(crate) use journal_consumer::{
    GpuSceneJournalApplyPlan, GpuSceneJournalConsumer, GpuSceneJournalConsumerError,
    GpuSceneJournalOwnedStagedTransaction, GpuSceneJournalReprojectionError,
    GpuSceneJournalReprojectionPlan, GpuSceneJournalReprojectionPreflightError,
    GpuSceneJournalResidentWrite, GpuSceneJournalResidentWriteKind, GpuSceneJournalRetirement,
    GpuSceneJournalSlotMutation, GpuSceneJournalStagedTransaction,
    GpuSceneJournalTransactionCommit, GpuSceneJournalTransactionError,
};
pub(crate) use layout::{
    GpuInstanceData, GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GpuPrimitiveData,
    GpuVirtualGeometryClusterWord, GpuVirtualGeometryPage, GPU_INSTANCE_DATA_STRIDE,
    GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM, GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM,
    GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT, GPU_INSTANCE_FLAG_NON_ORTHOGONAL_TRANSFORM,
    GPU_MORPH_DELTA_STRIDE, GPU_MORPH_PAYLOAD_STRIDE, GPU_MORPH_WEIGHT_STRIDE,
    GPU_PRIMITIVE_DATA_STRIDE, GPU_PRIMITIVE_FLAG_CAST_SHADOWS,
    GPU_PRIMITIVE_FLAG_FORCE_HZB_VISIBLE, GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
    GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT,
    GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
};
pub(crate) use morph::{GpuSceneMorphUploadReport, GpuScenePreparedMorphUpload};
pub(crate) use prepared_upload::GpuScenePreparedUpload;
pub(crate) use prev_skinned_palette::GpuSceneSkinnedJointPaletteState;
pub(crate) use prev_skinned_source::GpuSceneSkinnedGpuSourceState;
pub(crate) use virtual_geometry::{
    GpuScenePreparedVirtualGeometryUpload, GpuSceneVirtualGeometryUploadReport,
};
