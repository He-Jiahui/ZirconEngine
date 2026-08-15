use zircon_runtime::core::framework::render::PostProcessGraphResourceNames;
use zircon_runtime::graphics::{
    ComputePassDescriptor, ComputeShaderSource, FrameHistoryBinding, FrameHistorySlot,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassStage,
};
use zircon_runtime::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphComputeDispatchExtent,
};

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingSsaoRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.ssao";
pub const FEATURE_NAME: &str = "screen_space_ambient_occlusion";
pub const EXECUTOR_ID: &str = "compute.generic";
const SSAO_EVALUATE_PIPELINE_LABEL: &str = "zircon-ssao-pipeline";
const SSAO_EVALUATE_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
const SSAO_WGSL: &str = include_str!(
    "../../../../../../zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao.wgsl"
);

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    let compute_pass = ComputePassDescriptor::new(
        "ssao-evaluate",
        RenderPassStage::AmbientOcclusion,
        QueueLane::AsyncCompute,
        ComputeShaderSource::builtin_wgsl(SSAO_EVALUATE_PIPELINE_LABEL, SSAO_WGSL),
        "cs_main",
        SSAO_EVALUATE_WORKGROUP_SIZE,
        vec![
            BindingSchemaEntry::new(
                0,
                PostProcessGraphResourceNames::SCENE_DEPTH,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                1,
                PostProcessGraphResourceNames::GBUFFER_NORMAL,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                2,
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_AMBIENT_OCCLUSION,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                3,
                PostProcessGraphResourceNames::SSAO_PARAMS,
                ComputeBindingKind::UniformBuffer,
            ),
            BindingSchemaEntry::new(
                4,
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
                ComputeBindingKind::StorageTextureWrite,
            ),
            BindingSchemaEntry::new(
                5,
                PostProcessGraphResourceNames::HZB_FURTHEST,
                ComputeBindingKind::SampledTexture,
            )
            .with_texture_full_mip_chain(),
        ],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string(),
            local_size: [
                SSAO_EVALUATE_WORKGROUP_SIZE[0],
                SSAO_EVALUATE_WORKGROUP_SIZE[1],
            ],
        },
        PassFlags::default(),
    );

    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "ssao-evaluate",
            QueueLane::AsyncCompute,
        )
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
        .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
        .read_external_texture(PostProcessGraphResourceNames::HISTORY_PREVIOUS_AMBIENT_OCCLUSION)
        .read_external_buffer(PostProcessGraphResourceNames::SSAO_PARAMS)
        .write_storage_external_texture(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
        .with_compute_pass(compute_pass)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::render_graph::RenderGraphComputeDispatchExtent;

    #[test]
    fn ssao_feature_registers_history_binding() {
        let report = plugin_feature_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert_eq!(report.manifest.id, FEATURE_ID);
        assert!(report.manifest.enabled_by_default);
        assert_eq!(
            report.extensions.render_features()[0].history_bindings,
            vec![FrameHistoryBinding::read_write(
                FrameHistorySlot::AmbientOcclusion
            )]
        );
        let pass = &report.extensions.render_features()[0].stage_passes[0];
        let workload = pass
            .compute_workload
            .as_ref()
            .expect("ssao async compute pass should declare workload");
        assert_eq!(pass.queue, QueueLane::AsyncCompute);
        assert_eq!(workload.pipeline_label, SSAO_EVALUATE_PIPELINE_LABEL);
        assert_eq!(workload.workgroup_size, SSAO_EVALUATE_WORKGROUP_SIZE);
        assert_eq!(
            workload.dispatch_extent,
            RenderGraphComputeDispatchExtent::PerPixel {
                target: "ambient-occlusion".to_string(),
                local_size: [
                    SSAO_EVALUATE_WORKGROUP_SIZE[0],
                    SSAO_EVALUATE_WORKGROUP_SIZE[1],
                ],
            }
        );
    }
}
