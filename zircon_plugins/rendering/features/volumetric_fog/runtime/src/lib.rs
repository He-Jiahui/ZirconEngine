use zircon_runtime::core::framework::render::PostProcessGraphResourceNames;
use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutorRegistration,
    RenderPassStage,
};
use zircon_runtime::render_graph::{QueueLane, RenderGraphComputeWorkload};

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingVolumetricFogRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.volumetric_fog";
pub const FEATURE_NAME: &str = "volumetric_fog";
pub const MEDIA_INJECT_PASS: &str = "volumetric.media_inject";
pub const LIGHT_SCATTER_PASS: &str = "volumetric.light_scatter";
pub const INTEGRATE_PASS: &str = "volumetric.integrate";
pub const MEDIA_INJECT_EXECUTOR: &str = "volumetric.media_inject";
pub const LIGHT_SCATTER_EXECUTOR: &str = "volumetric.light_scatter";
pub const INTEGRATE_EXECUTOR: &str = "volumetric.integrate";
pub const MEDIA_INJECT_PIPELINE_LABEL: &str = "zircon-volumetric-media-inject";
pub const LIGHT_SCATTER_PIPELINE_LABEL: &str = "zircon-volumetric-light-scatter";
pub const INTEGRATE_PIPELINE_LABEL: &str = "zircon-volumetric-integrate";
pub const FROXEL_WORKGROUP_SIZE: [u32; 3] = [4, 4, 4];
pub const INTEGRATE_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                MEDIA_INJECT_PASS,
                QueueLane::AsyncCompute,
            )
            .with_executor_id(MEDIA_INJECT_EXECUTOR)
            .with_compute_workload(RenderGraphComputeWorkload::froxel_grid(
                MEDIA_INJECT_PIPELINE_LABEL,
                FROXEL_WORKGROUP_SIZE,
            ))
            .write_storage_texture(PostProcessGraphResourceNames::VOLUMETRIC_MEDIA),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                LIGHT_SCATTER_PASS,
                QueueLane::AsyncCompute,
            )
            .with_executor_id(LIGHT_SCATTER_EXECUTOR)
            .with_compute_workload(RenderGraphComputeWorkload::froxel_grid(
                LIGHT_SCATTER_PIPELINE_LABEL,
                FROXEL_WORKGROUP_SIZE,
            ))
            .read_texture(PostProcessGraphResourceNames::VOLUMETRIC_MEDIA)
            .read_external_texture(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING,
            )
            .read_required_external_buffer(PostProcessGraphResourceNames::SCENE_LIGHT_DATA)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .write_storage_texture(PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                INTEGRATE_PASS,
                QueueLane::AsyncCompute,
            )
            .with_executor_id(INTEGRATE_EXECUTOR)
            .with_compute_workload(RenderGraphComputeWorkload::froxel_grid_xy(
                INTEGRATE_PIPELINE_LABEL,
                INTEGRATE_WORKGROUP_SIZE,
            ))
            .read_texture(PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING)
            .write_storage_texture(PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED),
        ],
    )
    .with_pass_read_texture(
        "opaque-mesh",
        PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
    )
    .with_pass_read_texture(
        "alpha-mask-mesh",
        PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
    )
    .with_pass_read_texture(
        "transparent-mesh",
        PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
    )
    .with_pass_read_texture(
        "oit.fragment_store",
        PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
    )
    .with_pass_read_texture(
        "deferred-lighting",
        PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
    )
    .with_pass_read_texture(
        "preview-sky",
        PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
    )
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    zircon_runtime::graphics::volumetric_fog_render_pass_executor_registrations()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod wgpu_product_tests;
