use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::asset::ProjectAssetManagerAccess;
use crate::core::framework::render::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetWritebackReport,
};
use crate::core::resource::ResourceId;
use crate::graphics::material::ShadingModelRegistry;
use crate::graphics::scene::scene_renderer::mip_gen::RuntimeMipGenPass;
use crate::plugin::ShaderModuleSourceBinding;

use super::super::prepared::{
    PreparedMaterial, PreparedMesh, PreparedModel, PreparedOutputTargetTexture,
    PreparedPostProcessLutTexture, PreparedShader, PreparedTexture,
};
use super::super::{
    GpuMaterialUniformResource, GpuTextureResource, OutputTargetFramePlan,
    OutputTargetWritebackConverter, TextureSamplerCache, UiTexturePrepareReceipt,
};
use super::resource_streamer_mip_streaming::{MipStreamingState, MipStreamingVisibility};

pub(crate) struct ResourceStreamer {
    pub(super) asset_manager_access: ProjectAssetManagerAccess,
    pub(super) shading_model_registry: ShadingModelRegistry,
    pub(super) shader_module_sources: BTreeMap<String, ShaderModuleSourceBinding>,
    pub(super) models: HashMap<ResourceId, PreparedModel>,
    pub(super) meshes: HashMap<ResourceId, PreparedMesh>,
    pub(super) materials: HashMap<ResourceId, PreparedMaterial>,
    pub(super) active_staged_material_ids: HashSet<ResourceId>,
    pub(super) next_material_draw_generation: u64,
    pub(super) textures: HashMap<ResourceId, PreparedTexture>,
    pub(super) mip_streaming_states: HashMap<ResourceId, MipStreamingState>,
    pub(super) mip_streaming_visible_instance_keys: HashSet<u64>,
    pub(super) mip_streaming_visibility: Vec<MipStreamingVisibility>,
    pub(super) mip_streaming_residency_budget_bytes: u64,
    pub(super) output_target_textures: HashMap<ResourceId, PreparedOutputTargetTexture>,
    pub(super) post_process_lut_textures: HashMap<ResourceId, PreparedPostProcessLutTexture>,
    pub(super) shaders: HashMap<ResourceId, PreparedShader>,
    pub(super) texture_sampler_cache: Arc<TextureSamplerCache>,
    pub(super) fallback_texture: Arc<GpuTextureResource>,
    pub(super) fallback_normal_texture: Arc<GpuTextureResource>,
    pub(super) fallback_material_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) fallback_standard_material_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) runtime_mip_gen_pass: RuntimeMipGenPass,
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
    pub(in crate::graphics::scene::resources) next_ui_texture_prepare_epoch: u64,
    pub(in crate::graphics::scene::resources) last_ui_texture_prepare_receipt:
        Option<UiTexturePrepareReceipt>,
    pub(super) last_output_target_frame_plan: OutputTargetFramePlan,
    pub(super) last_output_target_graph_import_report: RenderCameraTargetGraphImportReport,
    pub(super) last_output_target_writeback_report: RenderCameraTargetWritebackReport,
}
