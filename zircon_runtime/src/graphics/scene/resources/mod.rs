mod fallback;
mod gpu_material_uniform;
mod gpu_mesh;
mod gpu_model;
mod gpu_texture;
mod output_target_texture;
mod pipeline;
mod post_process_lut_texture;
mod prepared;
mod render_asset_residency;
mod renderer_shader_layout_contract;
mod resource_streamer;
mod runtime;
mod ui_texture;

pub(crate) use fallback::fallback_shader_uri;
pub(crate) use gpu_material_uniform::{
    GPU_MATERIAL_UNIFORM_MIN_SIZE, GpuMaterialUniformResource, standard_material_uniform_contents,
};
pub(crate) use gpu_mesh::{GpuMeshResource, GpuMeshVertex};
pub(crate) use gpu_model::GpuModelResource;
pub(in crate::graphics::scene::resources) use gpu_texture::GpuTextureUploadWork;
pub(crate) use gpu_texture::{
    GpuTextureResource, TextureSamplerCache, texture_upload_support_from_device,
};
pub(in crate::graphics::scene::resources) use output_target_texture::OutputTargetWritebackConverter;
pub(in crate::graphics::scene) use output_target_texture::{
    OutputTargetFramePlan, OutputTargetTextureResource,
};
pub(crate) use pipeline::{PipelineKey, default_pipeline_key};
pub(in crate::graphics::scene::resources) use post_process_lut_texture::{
    PostProcessLutTextureResource, PostProcessLutTextureUploadWork,
};
pub(crate) use render_asset_residency::{
    RenderAssetDemandGeneration, RenderAssetDeviceEpoch, RenderAssetDeviceRecoveryError,
    RenderAssetDeviceRecoveryReport, RenderAssetGpuArtifact, RenderAssetGpuArtifactKind,
    RenderAssetGpuMaintenanceBudget, RenderAssetGpuMaintenanceFailure,
    RenderAssetGpuMaintenanceReport, RenderAssetGpuMeshArtifact, RenderAssetGpuMeshLod,
    RenderAssetGpuPollReceiptError, RenderAssetGpuResidencyLimits, RenderAssetGpuTextureArtifact,
    RenderAssetGpuUploadBindFailure, RenderAssetGpuUploadBudgetClass, RenderAssetGpuUploadLease,
    RenderAssetGpuUploadLimits, RenderAssetGpuUploadPlan, RenderAssetGpuUploadPlanError,
    RenderAssetGpuUploadPlanKind, RenderAssetGpuUploadQuote, RenderAssetGpuUploadSubmitError,
    RenderAssetResidencyAdmissionError, RenderAssetResidencyManager, RenderAssetResidencyMutation,
    RenderAssetResidencyMutationStats, RenderAssetResidencyRelease,
    RenderAssetResidencyReleaseKind, RenderAssetResidencyScope, RenderAssetResidencyState,
    RenderAssetResidencyTicket, RenderAssetResidencyTicketId, RenderAssetResidencyTransitionError,
};
pub(in crate::graphics::scene) use renderer_shader_layout_contract::{
    GPU_SCENE_DRAW_BIND_GROUP, MATERIAL_BASE_COLOR_SAMPLER_BINDING,
    MATERIAL_BASE_COLOR_TEXTURE_BINDING, MATERIAL_BIND_GROUP, MATERIAL_BINDING_COUNT,
    MATERIAL_CLEARCOAT_NORMAL_SAMPLER_BINDING, MATERIAL_CLEARCOAT_NORMAL_TEXTURE_BINDING,
    MATERIAL_EMISSIVE_SAMPLER_BINDING, MATERIAL_EMISSIVE_TEXTURE_BINDING,
    MATERIAL_METALLIC_ROUGHNESS_SAMPLER_BINDING, MATERIAL_METALLIC_ROUGHNESS_TEXTURE_BINDING,
    MATERIAL_NORMAL_SAMPLER_BINDING, MATERIAL_NORMAL_TEXTURE_BINDING,
    MATERIAL_OCCLUSION_SAMPLER_BINDING, MATERIAL_OCCLUSION_TEXTURE_BINDING,
    MATERIAL_UNIFORM_BINDING, RendererShaderBindingContract, gpu_scene_shader_binding_contract,
    material_shader_binding_contract,
};
pub(in crate::graphics::scene::resources) use resource_streamer::TextureSnapshotFramePrepareError;
pub(crate) use resource_streamer::{
    IrradianceVolumeTextureBinding, MaterialDrawGenerationSelection, PublishedMaterialDrawProxy,
    ResourceStreamer,
};
pub(in crate::graphics::scene) use resource_streamer::{
    PublishedMaterialTextureBinding, PublishedMaterialTextureSet,
};
pub(crate) use runtime::MaterialCaptureSeed;
pub(crate) use runtime::{MaterialDisabledPasses, MaterialRuntime};
pub(in crate::graphics::scene) use ui_texture::UiTexturePrepareReceipt;
pub(crate) use ui_texture::ui_image_resource_id;
pub(in crate::graphics::scene::resources) use ui_texture::ui_texture_ids;
