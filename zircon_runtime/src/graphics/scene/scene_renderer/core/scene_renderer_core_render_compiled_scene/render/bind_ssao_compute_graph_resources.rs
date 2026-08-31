use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::CompiledRenderPipeline;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::scene::scene_renderer::post_process::ScenePostProcessResources;
use crate::graphics::types::GraphicsError;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

pub(super) fn bind_ssao_compute_graph_resources(
    pipeline: &CompiledRenderPipeline,
    target: &OffscreenTarget,
    post_process: &ScenePostProcessResources,
    graph_resources: &mut RenderGraphExecutionResources,
    frame_buffer_uploads: &mut WgpuBufferUploadBatch,
) -> Result<(), GraphicsError> {
    let graph = pipeline.graph();
    if graph
        .resource_lifetime_by_name(PostProcessGraphResourceNames::SSAO_PARAMS)
        .is_some()
    {
        let profile = pipeline.ambient_occlusion_profile().ok_or_else(|| {
            GraphicsError::Asset(
                "compiled SSAO graph is missing its compiled AO profile".to_string(),
            )
        })?;
        post_process
            .prepare_ssao_compute_params_upload(profile, target.render_size, frame_buffer_uploads)
            .map_err(GraphicsError::Asset)?;
        graph_resources.import_borrowed_buffer_with_physical_desc(
            PostProcessGraphResourceNames::SSAO_PARAMS,
            post_process.ssao_params_buffer(),
            post_process.ssao_params_buffer_desc(PostProcessGraphResourceNames::SSAO_PARAMS),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn ssao_binding_prepares_upload_without_touching_the_queue() {
        let source = include_str!("bind_ssao_compute_graph_resources.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("compiled graph SSAO resource binding source");

        assert!(!production.contains("queue: &wgpu::Queue"));
        assert!(production.contains("prepare_ssao_compute_params_upload("));
        assert!(production.contains("pipeline.ambient_occlusion_profile()"));
        assert!(production.contains("target.render_size"));
        assert!(production.contains("import_borrowed_buffer_with_physical_desc("));
        assert!(production.contains("ssao_params_buffer_desc("));
        assert!(production.contains("frame_buffer_uploads"));
        assert!(!production.contains("HISTORY_PREVIOUS_AMBIENT_OCCLUSION"));
        assert!(!production.contains("white_texture_view"));
    }
}
