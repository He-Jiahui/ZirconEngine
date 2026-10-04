mod executors;
mod pipelines;
mod prepared_frame;

pub(crate) use prepared_frame::{
    SSS_PARAMS_BUFFER_SIZE_BYTES, SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES,
};

use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassExecutorRegistration;
use crate::graphics::{RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage};
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps, RenderGraphComputeWorkload};

pub const SSS_SETUP_EXECUTOR_ID: &str = "sss.setup";
pub const SSS_SCATTER_EXECUTOR_ID: &str = "sss.scatter";
pub const SSS_RECOMBINE_EXECUTOR_ID: &str = "sss.recombine";

pub const SSS_SETUP_PIPELINE_LABEL: &str = "sss.setup";
pub const SSS_SCATTER_PIPELINE_LABEL: &str = "sss.scatter.burley";
pub const SSS_RECOMBINE_PIPELINE_LABEL: &str = "sss.recombine";
pub const SSS_TILE_SIZE: [u32; 3] = [8, 8, 1];

pub fn setup_compute_workload() -> RenderGraphComputeWorkload {
    RenderGraphComputeWorkload::per_pixel(
        SSS_SETUP_PIPELINE_LABEL,
        SSS_TILE_SIZE,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        [SSS_TILE_SIZE[0], SSS_TILE_SIZE[1]],
    )
}

pub fn scatter_compute_workload() -> RenderGraphComputeWorkload {
    RenderGraphComputeWorkload::indirect_args(SSS_SCATTER_PIPELINE_LABEL, SSS_TILE_SIZE)
}

/// 把次表面散射插入延迟光照链：延迟光照保留漫反射与高光，
/// setup 选活动 tile，scatter 按间接参数计算，recombine 仅覆盖 SSS 像素。
/// 调用方还须注册三个对应执行器，并满足高级光照启用条件。
pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "subsurface_scattering",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "advanced_lighting".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                SSS_SETUP_EXECUTOR_ID,
                QueueLane::AsyncCompute,
            )
            .with_executor_id(SSS_SETUP_EXECUTOR_ID)
            .with_compute_workload(setup_compute_workload())
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .write_buffer(PostProcessGraphResourceNames::SSS_TILE_LIST)
            .write_buffer(PostProcessGraphResourceNames::SSS_INDIRECT_ARGS)
            .write_buffer(PostProcessGraphResourceNames::SSS_PARAMS)
            .write_buffer(PostProcessGraphResourceNames::SSS_PROFILES),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                SSS_SCATTER_EXECUTOR_ID,
                QueueLane::AsyncCompute,
            )
            .with_executor_id(SSS_SCATTER_EXECUTOR_ID)
            .with_compute_workload(scatter_compute_workload())
            .read_texture(PostProcessGraphResourceNames::SSS_DIFFUSE)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_buffer(PostProcessGraphResourceNames::SSS_TILE_LIST)
            .read_buffer(PostProcessGraphResourceNames::SSS_INDIRECT_ARGS)
            .read_buffer(PostProcessGraphResourceNames::SSS_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::SSS_PROFILES)
            .write_storage_texture(PostProcessGraphResourceNames::SSS_SCATTERED),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                SSS_RECOMBINE_EXECUTOR_ID,
                QueueLane::Graphics,
            )
            .with_executor_id(SSS_RECOMBINE_EXECUTOR_ID)
            .read_texture(PostProcessGraphResourceNames::SSS_SCATTERED)
            .read_texture(PostProcessGraphResourceNames::SSS_SPECULAR)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphAttachmentOps::load_store(),
            ),
        ],
    )
    .with_pass_write_texture(
        "deferred-lighting",
        PostProcessGraphResourceNames::SSS_DIFFUSE,
        RenderGraphAttachmentOps::clear_store(),
    )
    .with_pass_write_texture(
        "deferred-lighting",
        PostProcessGraphResourceNames::SSS_SPECULAR,
        RenderGraphAttachmentOps::clear_store(),
    )
    .when_advanced_lighting_subsurface_enabled()
}

pub(crate) fn registrations() -> Vec<RenderPassExecutorRegistration> {
    executors::registrations()
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/factory_coverage_tests.rs"]
mod factory_coverage_tests;
