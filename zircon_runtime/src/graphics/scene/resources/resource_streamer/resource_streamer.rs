use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetWritebackReport,
};
use crate::core::resource::ResourceId;
use crate::graphics::material::ShadingModelRegistry;

use super::super::prepared::{
    PreparedMaterial, PreparedMesh, PreparedModel, PreparedOutputTargetTexture,
    PreparedPostProcessLutTexture, PreparedShader, PreparedTexture,
};
use super::super::{
    GpuMaterialUniformResource, GpuTextureResource, OutputTargetWritebackConverter,
};

pub(crate) struct ResourceStreamer {
    pub(super) asset_manager: Arc<ProjectAssetManager>,
    pub(super) shading_model_registry: ShadingModelRegistry,
    pub(super) models: HashMap<ResourceId, PreparedModel>,
    pub(super) meshes: HashMap<ResourceId, PreparedMesh>,
    pub(super) materials: HashMap<ResourceId, PreparedMaterial>,
    pub(super) textures: HashMap<ResourceId, PreparedTexture>,
    pub(super) output_target_textures: HashMap<ResourceId, PreparedOutputTargetTexture>,
    pub(super) post_process_lut_textures: HashMap<ResourceId, PreparedPostProcessLutTexture>,
    pub(super) shaders: HashMap<ResourceId, PreparedShader>,
    pub(super) fallback_texture: Arc<GpuTextureResource>,
    pub(super) fallback_normal_texture: Arc<GpuTextureResource>,
    pub(super) fallback_material_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) fallback_standard_material_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) output_target_writeback_converter: OutputTargetWritebackConverter,
    pub(super) last_material_count: usize,
    pub(super) last_material_ready_count: usize,
    pub(super) last_material_fallback_count: usize,
    pub(super) last_material_validation_error_count: usize,
    pub(super) last_material_diagnostic_count: usize,
    pub(super) last_sprite_count: usize,
    pub(super) last_sprite_ready_count: usize,
    pub(super) last_sprite_texture_fallback_count: usize,
    pub(super) last_post_process_lut_request_count: usize,
    pub(super) last_post_process_lut_ready_count: usize,
    pub(super) last_post_process_lut_fallback_count: usize,
    pub(super) last_post_process_lut_2d_strip_ready_count: usize,
    pub(super) last_post_process_lut_3d_request_count: usize,
    pub(super) last_post_process_lut_unsupported_shape_count: usize,
    pub(super) last_output_target_graph_import_report: RenderCameraTargetGraphImportReport,
    pub(super) last_output_target_writeback_report: RenderCameraTargetWritebackReport,
}
