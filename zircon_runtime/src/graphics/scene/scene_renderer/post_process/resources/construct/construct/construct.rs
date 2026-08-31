use super::super::super::super::scene_post_process_resources::{
    FullScenePostProcessResources, SceneOutputTransferResources, ScenePostProcessResources,
};
use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::super::terminal_resource_cache::TerminalPostProcessResourceCache;
use super::super::bind_group_layouts;
use super::super::create_buffer_bundle::create_buffer_bundle;
use super::super::create_fallback_texture_views::create_fallback_texture_views;
use super::super::create_pipeline_bundle::{create_pipeline_bundle, output_transfer_pipeline};
use crate::graphics::backend::SystemTextureGenerationLease;
use crate::graphics::shader::{FullscreenPassParameterBindings, motion_vector_tile_max_pass_plan};
use crate::graphics::types::GraphicsError;

impl ScenePostProcessResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        system_textures: &SystemTextureGenerationLease,
        final_color_format: wgpu::TextureFormat,
        backend_name: &str,
    ) -> Result<Self, GraphicsError> {
        validate_post_process_construction(device, "full resources", || {
            Self::Full(FullScenePostProcessResources::new(
                device,
                system_textures,
                final_color_format,
                backend_name,
            ))
        })
    }

    pub(crate) fn output_transfer_only(
        device: &wgpu::Device,
        final_color_format: wgpu::TextureFormat,
    ) -> Result<Self, GraphicsError> {
        validate_post_process_construction(device, "output transfer resources", || {
            let bind_group_layout = bind_group_layouts::output_transfer(device);
            let pipeline = output_transfer_pipeline(device, final_color_format, &bind_group_layout);
            Self::OutputTransferOnly(SceneOutputTransferResources {
                terminal_resource_cache: TerminalPostProcessResourceCache::new(),
                bind_group_layout,
                pipeline,
            })
        })
    }
}

fn validate_post_process_construction<T>(
    device: &wgpu::Device,
    operation: &str,
    construct: impl FnOnce() -> T,
) -> Result<T, GraphicsError> {
    // This scope runs once during renderer initialization; frame recording never waits on it.
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let resources = construct();
    match pollster::block_on(error_scope.pop()) {
        Some(error) => Err(GraphicsError::WgpuValidation(format!(
            "post-process {operation} construction: {error}"
        ))),
        None => Ok(resources),
    }
}

impl FullScenePostProcessResources {
    fn new(
        device: &wgpu::Device,
        system_textures: &SystemTextureGenerationLease,
        final_color_format: wgpu::TextureFormat,
        backend_name: &str,
    ) -> Self {
        let depth_sampling_mode = PostProcessDepthSamplingMode::for_backend_name(backend_name);
        let bloom_bind_group_layout = bind_group_layouts::bloom(device);
        let cluster_bind_group_layout = bind_group_layouts::cluster(device);
        let hzb_bind_group_layout = bind_group_layouts::hzb(device);
        let hzb_msaa_bind_group_layout = bind_group_layouts::hzb_msaa(device);
        let half_res_transparency_depth_downsample_bind_group_layout =
            bind_group_layouts::half_res_transparency_depth_downsample(device);
        let half_res_transparency_composite_bind_group_layout =
            bind_group_layouts::half_res_transparency_composite(device);
        let exposure_histogram_bind_group_layout = bind_group_layouts::exposure_histogram(device);
        let exposure_resolve_bind_group_layout = bind_group_layouts::exposure_resolve(device);
        let color_lut_bake_bind_group_layout = bind_group_layouts::color_lut_bake(device);
        let depth_of_field_prepare_bind_group_layout =
            bind_group_layouts::depth_of_field_prepare(device, depth_sampling_mode);
        let taa_resolve_bind_group_layout =
            bind_group_layouts::taa_resolve(device, depth_sampling_mode);
        let velocity_camera_bind_group_layout =
            bind_group_layouts::velocity_camera(device, depth_sampling_mode);
        let motion_vector_tile_max_bind_group_layout =
            bind_group_layouts::motion_vector_tile_max(device);
        let motion_vector_tile_max_parameter_bind_group_layout =
            bind_group_layouts::motion_vector_tile_max_parameters(device);
        let motion_vector_tile_max_parameter_bindings = FullscreenPassParameterBindings::new(
            device,
            motion_vector_tile_max_pass_plan(),
            &motion_vector_tile_max_parameter_bind_group_layout,
        )
        .expect("motion-vector tile-max fullscreen plan must declare parameters");
        let motion_vector_neighbor_max_bind_group_layout =
            bind_group_layouts::motion_vector_neighbor_max(device);
        let post_process_bind_group_layout =
            bind_group_layouts::post_process(device, depth_sampling_mode);
        let upscale_bind_group_layout = bind_group_layouts::upscale(device);
        let output_transfer_bind_group_layout = bind_group_layouts::output_transfer(device);
        let smaa_bind_group_layout = bind_group_layouts::smaa(device);
        let pipeline_bundle = create_pipeline_bundle(
            device,
            final_color_format,
            &bloom_bind_group_layout,
            &cluster_bind_group_layout,
            &hzb_bind_group_layout,
            &hzb_msaa_bind_group_layout,
            &half_res_transparency_depth_downsample_bind_group_layout,
            &half_res_transparency_composite_bind_group_layout,
            &exposure_histogram_bind_group_layout,
            &exposure_resolve_bind_group_layout,
            &color_lut_bake_bind_group_layout,
            &depth_of_field_prepare_bind_group_layout,
            &taa_resolve_bind_group_layout,
            &velocity_camera_bind_group_layout,
            &motion_vector_tile_max_bind_group_layout,
            &motion_vector_tile_max_parameter_bind_group_layout,
            &motion_vector_neighbor_max_bind_group_layout,
            &post_process_bind_group_layout,
            &upscale_bind_group_layout,
            &output_transfer_bind_group_layout,
            &smaa_bind_group_layout,
            depth_sampling_mode,
        );
        let buffer_bundle = create_buffer_bundle(device);
        let fallback_texture_views = create_fallback_texture_views(system_textures);

        Self {
            post_process_pass_parameter_buffers: buffer_bundle
                .post_process_pass_parameter_buffers,
            hzb_fallback_resource_identity:
                crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity::new(),
            depth_sampling_mode,
            bloom_bind_group_layout,
            cluster_bind_group_layout,
            hzb_bind_group_layout,
            hzb_msaa_bind_group_layout,
            half_res_transparency_depth_downsample_bind_group_layout,
            half_res_transparency_composite_bind_group_layout,
            exposure_histogram_bind_group_layout,
            exposure_resolve_bind_group_layout,
            color_lut_bake_bind_group_layout,
            depth_of_field_prepare_bind_group_layout,
            taa_resolve_bind_group_layout,
            taa_resolve_bind_group_cache: std::sync::Mutex::new(
                crate::graphics::scene::scene_renderer::temporal::taa::taa_resolve_bind_group_cache::TaaResolveBindGroupCache::default(),
            ),
            velocity_camera_bind_group_layout,
            motion_vector_tile_max_bind_group_layout,
            motion_vector_tile_max_parameter_bindings,
            motion_vector_neighbor_max_bind_group_layout,
            post_process_bind_group_layout,
            upscale_bind_group_layout,
            output_transfer_bind_group_layout,
            smaa_bind_group_layout,
            terminal_resource_cache: TerminalPostProcessResourceCache::new(),
            bloom_pipeline: pipeline_bundle.bloom_pipeline,
            cluster_pipeline: pipeline_bundle.cluster_pipeline,
            hzb_pipeline: pipeline_bundle.hzb_pipeline,
            hzb_msaa_pipeline: pipeline_bundle.hzb_msaa_pipeline,
            half_res_transparency_depth_downsample_pipeline: pipeline_bundle
                .half_res_transparency_depth_downsample_pipeline,
            half_res_transparency_composite_pipeline: pipeline_bundle
                .half_res_transparency_composite_pipeline,
            exposure_histogram_pipeline: pipeline_bundle.exposure_histogram_pipeline,
            exposure_resolve_pipeline: pipeline_bundle.exposure_resolve_pipeline,
            color_lut_bake_pipeline: pipeline_bundle.color_lut_bake_pipeline,
            depth_of_field_prepare_pipeline: pipeline_bundle.depth_of_field_prepare_pipeline,
            depth_of_field_pipeline: pipeline_bundle.depth_of_field_pipeline,
            taa_resolve_pipeline: pipeline_bundle.taa_resolve_pipeline,
            velocity_camera_pipeline: pipeline_bundle.velocity_camera_pipeline,
            motion_vector_tile_max_pipeline: pipeline_bundle.motion_vector_tile_max_pipeline,
            motion_vector_neighbor_max_pipeline: pipeline_bundle
                .motion_vector_neighbor_max_pipeline,
            motion_blur_pipeline: pipeline_bundle.motion_blur_pipeline,
            blur_pipeline: pipeline_bundle.blur_pipeline,
            scene_composite_pipeline: pipeline_bundle.scene_composite_pipeline,
            screen_space_reflection_reflection_pyramid_pipeline: pipeline_bundle
                .screen_space_reflection_reflection_pyramid_pipeline,
            screen_space_reflection_reflection_pyramid_coarse_pipeline: pipeline_bundle
                .screen_space_reflection_reflection_pyramid_coarse_pipeline,
            screen_space_reflection_resolve_pipeline: pipeline_bundle
                .screen_space_reflection_resolve_pipeline,
            screen_space_reflection_specular_occlusion_pipeline: pipeline_bundle
                .screen_space_reflection_specular_occlusion_pipeline,
            post_process_pipeline: pipeline_bundle.post_process_pipeline,
            upscale_pipeline: pipeline_bundle.upscale_pipeline,
            output_transfer_pipeline: pipeline_bundle.output_transfer_pipeline,
            fxaa_pipeline: pipeline_bundle.fxaa_pipeline,
            smaa_edge_pipeline: pipeline_bundle.smaa_edge_pipeline,
            smaa_blend_pipeline: pipeline_bundle.smaa_blend_pipeline,
            smaa_resolve_pipeline: pipeline_bundle.smaa_resolve_pipeline,
            bloom_params_buffer: buffer_bundle.bloom_params_buffer,
            ssao_params_buffer: buffer_bundle.ssao_params_buffer,
            cluster_params_buffer: buffer_bundle.cluster_params_buffer,
            hzb_params_buffer: buffer_bundle.hzb_params_buffer,
            half_res_transparency_params_buffer: buffer_bundle.half_res_transparency_params_buffer,
            taa_resolve_params_buffer: buffer_bundle.taa_resolve_params_buffer,
            primary_upscale_params_buffer: buffer_bundle.primary_upscale_params_buffer,
            secondary_upscale_params_buffer: buffer_bundle.secondary_upscale_params_buffer,
            exposure_params_buffer: buffer_bundle.exposure_params_buffer,
            color_lut_bake_params_buffer: buffer_bundle.color_lut_bake_params_buffer,
            default_exposure_buffer: buffer_bundle.default_exposure_buffer,
            default_exposure_histogram_buffer: buffer_bundle.default_exposure_histogram_buffer,
            depth_of_field_prepare_params_buffer: buffer_bundle
                .depth_of_field_prepare_params_buffer,
            velocity_camera_params_buffer: buffer_bundle.velocity_camera_params_buffer,
            light_buffer: buffer_bundle.light_buffer,
            hybrid_gi_probe_buffer: buffer_bundle.hybrid_gi_probe_buffer,
            hybrid_gi_trace_region_buffer: buffer_bundle.hybrid_gi_trace_region_buffer,
            reflection_probe_buffer: buffer_bundle.reflection_probe_buffer,
            black_texture_view: fallback_texture_views.black_texture_view,
            black_texture_identity: crate::graphics::resource_identity::SampledTextureIdentity::new(),
            white_texture_view: fallback_texture_views.white_texture_view,
            hzb_source_texture_view: fallback_texture_views.hzb_source_texture_view,
            effect_lut_texture_view: fallback_texture_views.effect_lut_texture_view,
            effect_lut_texture_3d_view: fallback_texture_views.effect_lut_texture_3d_view,
            effect_lut_sampler: effect_lut_sampler(device),
            scene_depth_sampler: scene_depth_sampler(device),
            upscale_sampler: upscale_sampler(device),
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

fn upscale_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-post-process-upscale-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::types::GraphicsError;

    use super::validate_post_process_construction;

    #[test]
    fn post_process_construction_validation_scope_reports_its_operation() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };

        let result =
            validate_post_process_construction(&backend.device, "test-invalid-shader", || {
                backend
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("zircon-post-process-invalid-test-shader"),
                        source: wgpu::ShaderSource::Wgsl("not valid WGSL".into()),
                    })
            });

        assert!(matches!(
            result,
            Err(GraphicsError::WgpuValidation(message))
                if message.contains("post-process test-invalid-shader construction")
        ));
    }
}
