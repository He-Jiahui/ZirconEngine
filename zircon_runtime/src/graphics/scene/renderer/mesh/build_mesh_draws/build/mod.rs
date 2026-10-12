//! 从视图快照收集 pending draw，完成材质准入、变形与 GPUScene 准备，并把静态缓存命令和残余 MeshDraw 交给上层提交。
#[path = "build.rs"]
mod build;
#[path = "../../build/draw_build_context.rs"]
mod build_mesh_draw_build_context;
#[path = "../../build/collect_pending_draws.rs"]
mod collect_pending_draws;
#[path = "../../../../../mesh/build/extend_pending_draws_for_mesh_instance.rs"]
mod extend_pending_draws_for_mesh_instance;
#[path = "../../../../mesh/build/geometry_source_selection.rs"]
mod geometry_source_selection;
#[path = "../../build/gpu_scene_bounds.rs"]
mod gpu_scene_bounds;
#[path = "../../build/gpu_scene_sync.rs"]
mod gpu_scene_sync;
#[path = "../../../../mesh/build/material_context_admission.rs"]
mod material_context_admission;
#[path = "../../build/material_draw_selection.rs"]
mod material_draw_selection;
#[path = "../../../../mesh/build/material_pipeline_requirements.rs"]
mod material_pipeline_requirements;
#[path = "../../build/mesh_draw_build_context.rs"]
mod mesh_draw_build_context;
#[path = "../../build/morph_payload_upload.rs"]
mod morph_payload_upload;
#[path = "../../build/command_cache.rs"]
mod pending_command_cache_extract;
#[path = "../../../../mesh/build/pending_command_cache_plan.rs"]
mod pending_command_cache_plan;
#[path = "../../build/pending_material_draw.rs"]
mod pending_material_draw;
#[path = "../../build/pending_mesh_draw.rs"]
mod pending_mesh_draw;
#[path = "../../build/phase_ordering.rs"]
mod phase_ordering;
#[path = "../../../../mesh/build/previous_skinned_palette.rs"]
mod previous_skinned_palette;
#[path = "../../build/skinning.rs"]
mod skinning;
#[path = "../../../../mesh/build/virtual_geometry_indirect.rs"]
mod virtual_geometry_indirect;
#[path = "../../../../mesh/build/virtual_geometry_resident_upload.rs"]
mod virtual_geometry_resident_upload;

pub(crate) use build::{build_mesh_draws, BuiltMeshDraws};
pub(crate) use material_pipeline_requirements::{
    MaterialPipelineFeatureSet, MaterialPipelineRequirementCensus,
};
pub(crate) use pending_command_cache_extract::{
    PendingMeshCommandCacheExtractionContext, PendingMeshCommandCacheExtractionStats,
};
pub(crate) use pending_command_cache_plan::PendingMeshCommandCachePlanStats;
