use zircon_runtime::graphics::{
    FrameHistoryBinding, FrameHistorySlot, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassExecutionContext, RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::{QueueLane, RenderGraphComputeWorkload};

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingSsaoRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.ssao";
pub const FEATURE_NAME: &str = "screen_space_ambient_occlusion";
pub const EXECUTOR_ID: &str = "ao.ssao-evaluate";
const SSAO_EVALUATE_PIPELINE_LABEL: &str = "zircon-ssao-evaluate";
const SSAO_EVALUATE_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
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
        .with_executor_id(EXECUTOR_ID)
        .with_compute_workload(RenderGraphComputeWorkload::viewport(
            SSAO_EVALUATE_PIPELINE_LABEL,
            SSAO_EVALUATE_WORKGROUP_SIZE,
        ))
        .read_texture("scene-depth")
        .write_texture("ambient-occlusion")],
    )
}

pub fn render_pass_executor_registration() -> RenderPassExecutorRegistration {
    RenderPassExecutorRegistration::new(EXECUTOR_ID, noop_render_executor)
}

fn noop_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
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
            RenderGraphComputeDispatchExtent::Viewport
        );
    }
}
