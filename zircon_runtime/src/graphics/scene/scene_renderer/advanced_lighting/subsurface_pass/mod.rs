mod executors;
mod pipelines;

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
    RenderGraphComputeWorkload::viewport(SSS_SETUP_PIPELINE_LABEL, SSS_TILE_SIZE)
}

pub fn scatter_compute_workload() -> RenderGraphComputeWorkload {
    RenderGraphComputeWorkload::indirect_args(SSS_SCATTER_PIPELINE_LABEL, SSS_TILE_SIZE)
}

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
            .write_buffer(PostProcessGraphResourceNames::SSS_INDIRECT_ARGS),
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
mod tests {
    use super::*;

    #[test]
    fn render_sss_setup_workload_matches_tile_classifier_owner_constants() {
        let workload = setup_compute_workload();
        assert_eq!(workload.pipeline_label, SSS_SETUP_PIPELINE_LABEL);
        assert_eq!(workload.workgroup_size, SSS_TILE_SIZE);
    }

    #[test]
    fn render_sss_scatter_workload_is_gpu_indirect() {
        let workload = scatter_compute_workload();
        assert_eq!(workload.pipeline_label, SSS_SCATTER_PIPELINE_LABEL);
        assert_eq!(workload.workgroup_size, SSS_TILE_SIZE);
        assert_eq!(
            workload.dispatch_extent,
            crate::render_graph::RenderGraphComputeDispatchExtent::IndirectArgs
        );
    }

    #[test]
    fn render_sss_shaders_keep_tile_indirect_and_recombine_contracts() {
        assert!(pipelines::SETUP_SHADER.contains("atomicAdd(&indirect_args.group_count_x, 1u)"));
        assert!(pipelines::SCATTER_SHADER.contains("BURLEY_SAMPLE_COUNT: u32 = 64u"));
        assert!(pipelines::SCATTER_SHADER.contains("tile_list[workgroup_id.x]"));
        assert!(pipelines::SCATTER_SHADER.contains("mix(center_diffuse, scattered, falloff)"));
        assert!(pipelines::RECOMBINE_SHADER.contains("scattered_sample.rgb + specular_sample.rgb"));
    }

    #[test]
    fn render_sss_descriptor_extends_deferred_lighting_with_diffuse_and_retained_mrts() {
        let descriptor = render_feature_descriptor();
        let extensions = descriptor.resource_extensions().collect::<Vec<_>>();

        assert_eq!(extensions.len(), 2);
        assert!(extensions.iter().all(|extension| {
            extension.target_pass_name == "deferred-lighting"
                && extension.resource.access == crate::graphics::RenderFeatureResourceAccess::Write
        }));
    }
}
