use super::super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::bind_group_layouts;
use super::super::create_buffer_bundle::create_buffer_bundle;
use super::super::create_fallback_texture_views::create_fallback_texture_views;
use super::super::create_pipeline_bundle::create_pipeline_bundle;

impl ScenePostProcessResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let bloom_bind_group_layout = bind_group_layouts::bloom(device);
        let ssao_bind_group_layout = bind_group_layouts::ssao(device);
        let cluster_bind_group_layout = bind_group_layouts::cluster(device);
        let post_process_bind_group_layout = bind_group_layouts::post_process(device);
        let pipeline_bundle = create_pipeline_bundle(
            device,
            target_format,
            &bloom_bind_group_layout,
            &ssao_bind_group_layout,
            &cluster_bind_group_layout,
            &post_process_bind_group_layout,
        );
        let buffer_bundle = create_buffer_bundle(device);
        let fallback_texture_views = create_fallback_texture_views(device, queue);

        Self {
            bloom_bind_group_layout,
            ssao_bind_group_layout,
            cluster_bind_group_layout,
            post_process_bind_group_layout,
            bloom_pipeline: pipeline_bundle.bloom_pipeline,
            ssao_pipeline: pipeline_bundle.ssao_pipeline,
            cluster_pipeline: pipeline_bundle.cluster_pipeline,
            post_process_pipeline: pipeline_bundle.post_process_pipeline,
            bloom_params_buffer: buffer_bundle.bloom_params_buffer,
            ssao_params_buffer: buffer_bundle.ssao_params_buffer,
            cluster_params_buffer: buffer_bundle.cluster_params_buffer,
            post_process_params_buffer: buffer_bundle.post_process_params_buffer,
            light_buffer: buffer_bundle.light_buffer,
            hybrid_gi_probe_buffer: buffer_bundle.hybrid_gi_probe_buffer,
            hybrid_gi_trace_region_buffer: buffer_bundle.hybrid_gi_trace_region_buffer,
            reflection_probe_buffer: buffer_bundle.reflection_probe_buffer,
            black_texture_view: fallback_texture_views.black_texture_view,
            white_texture_view: fallback_texture_views.white_texture_view,
        }
    }
}
