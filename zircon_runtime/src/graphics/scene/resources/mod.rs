mod fallback;
mod gpu_material_uniform;
mod gpu_mesh;
mod gpu_model;
mod gpu_texture;
mod output_target_texture;
mod pipeline;
mod post_process_lut_texture;
mod prepared;
mod resource_streamer;
mod runtime;
mod ui_texture;

pub(crate) use fallback::fallback_shader_uri;
pub(crate) use gpu_material_uniform::{
    GpuMaterialUniformResource, GPU_MATERIAL_UNIFORM_MIN_SIZE, standard_material_uniform_contents,
};
pub(crate) use gpu_mesh::{GpuMeshResource, GpuMeshVertex};
pub(crate) use gpu_model::GpuModelResource;
pub(crate) use gpu_texture::{
    texture_upload_support_from_device, GpuTextureResource, TextureSamplerCache,
};
pub(in crate::graphics::scene) use output_target_texture::OutputTargetTextureResource;
pub(in crate::graphics::scene::resources) use output_target_texture::OutputTargetWritebackConverter;
pub(crate) use pipeline::{default_pipeline_key, PipelineKey};
pub(in crate::graphics::scene::resources) use post_process_lut_texture::PostProcessLutTextureResource;
pub(crate) use resource_streamer::{IrradianceVolumeTextureBinding, ResourceStreamer};
pub(crate) use runtime::MaterialCaptureSeed;
pub(crate) use runtime::{MaterialDisabledPasses, MaterialRuntime};
pub(crate) use ui_texture::ui_image_resource_id;
pub(in crate::graphics::scene::resources) use ui_texture::{
    ui_texture_id_for_upload, ui_texture_ids,
};
