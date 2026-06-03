use super::super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::bind_group_layouts;
use super::super::create_buffer_bundle::create_buffer_bundle;
use super::super::create_fallback_texture_views::create_fallback_texture_views;
use super::super::create_pipeline_bundle::create_pipeline_bundle;

impl ScenePostProcessResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        backend_name: &str,
    ) -> Self {
        let depth_sampling_mode = PostProcessDepthSamplingMode::for_backend_name(backend_name);
        let bloom_bind_group_layout = bind_group_layouts::bloom(device);
        let ssao_bind_group_layout = bind_group_layouts::ssao(device);
        let cluster_bind_group_layout = bind_group_layouts::cluster(device);
        let post_process_bind_group_layout =
            bind_group_layouts::post_process(device, depth_sampling_mode);
        let pipeline_bundle = create_pipeline_bundle(
            device,
            target_format,
            &bloom_bind_group_layout,
            &cluster_bind_group_layout,
            &post_process_bind_group_layout,
            depth_sampling_mode,
        );
        let buffer_bundle = create_buffer_bundle(device);
        let fallback_texture_views = create_fallback_texture_views(device, queue);

        Self {
            depth_sampling_mode,
            bloom_bind_group_layout,
            ssao_bind_group_layout,
            cluster_bind_group_layout,
            post_process_bind_group_layout,
            bloom_pipeline: pipeline_bundle.bloom_pipeline,
            ssao_pipeline: std::sync::OnceLock::new(),
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
            effect_lut_texture_view: fallback_texture_views.effect_lut_texture_view,
            effect_lut_texture_3d_view: fallback_texture_views.effect_lut_texture_3d_view,
            effect_lut_sampler: effect_lut_sampler(device),
            scene_depth_sampler: scene_depth_sampler(device),
        }
    }
}

fn effect_lut_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-post-process-effect-lut-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

fn scene_depth_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-post-process-scene-depth-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}
