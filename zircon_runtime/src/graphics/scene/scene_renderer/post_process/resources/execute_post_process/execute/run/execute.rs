use crate::core::math::UVec2;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::super::build_post_process_params::build_post_process_params;
use super::super::create_bind_group::create_bind_group;
use super::super::write_hybrid_gi_buffers::write_hybrid_gi_buffers;
use super::super::write_reflection_probes::write_reflection_probes;
use super::queue_post_process_params::queue_post_process_params;
use super::record_pass::record_pass;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_post_process(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        cluster_dimensions: UVec2,
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        scene_normal_view: &wgpu::TextureView,
        ao_view: &wgpu::TextureView,
        previous_scene_color_view: Option<&wgpu::TextureView>,
        previous_global_illumination_view: Option<&wgpu::TextureView>,
        bloom_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        global_illumination_view: &wgpu::TextureView,
        cluster_buffer: &wgpu::Buffer,
        frame: &ViewportRenderFrame,
        streamer: &ResourceStreamer,
        features: SceneRuntimeFeatureFlags,
        history_available: bool,
    ) {
        let extract = &frame.extract;
        let reflection_probe_count = write_reflection_probes(
            self,
            queue,
            extract,
            viewport_size,
            features.reflection_probes_enabled,
        );
        let (hybrid_gi_probe_count, scheduled_trace_region_count) = write_hybrid_gi_buffers(
            self,
            queue,
            frame,
            viewport_size,
            features.hybrid_global_illumination_enabled,
        );
        let effect_lut = select_effect_lut_texture_views(self, streamer, frame);
        let mut params = build_post_process_params(
            viewport_size,
            cluster_dimensions,
            extract,
            features,
            history_available,
            reflection_probe_count,
            hybrid_gi_probe_count,
            scheduled_trace_region_count,
        );
        params.effect_flags[1] = effect_lut.binding_mode.shader_id();
        queue_post_process_params(self, queue, &params);

        let bind_group = create_bind_group(
            self,
            device,
            scene_color_view,
            scene_depth_view,
            scene_normal_view,
            ao_view,
            previous_scene_color_view,
            previous_global_illumination_view,
            bloom_view,
            effect_lut.texture_2d_view,
            effect_lut.texture_3d_view,
            cluster_buffer,
        );
        record_pass(
            self,
            encoder,
            final_color_view,
            global_illumination_view,
            &bind_group,
        );
    }
}

struct EffectLutTextureViews<'a> {
    texture_2d_view: &'a wgpu::TextureView,
    texture_3d_view: &'a wgpu::TextureView,
    binding_mode: EffectLutBindingMode,
}

#[derive(Clone, Copy)]
enum EffectLutBindingMode {
    Disabled,
    Texture2d,
    Texture2dStrip,
    Texture3d,
}

impl EffectLutBindingMode {
    fn shader_id(self) -> u32 {
        match self {
            Self::Disabled => 0,
            Self::Texture2d => 1,
            Self::Texture2dStrip => 2,
            Self::Texture3d => 3,
        }
    }
}

fn select_effect_lut_texture_views<'a>(
    resources: &'a ScenePostProcessResources,
    streamer: &'a ResourceStreamer,
    frame: &ViewportRenderFrame,
) -> EffectLutTextureViews<'a> {
    let settings = frame.extract.post_process.effect_stack.color_lookup;
    let fallback = EffectLutTextureViews {
        texture_2d_view: &resources.effect_lut_texture_view,
        texture_3d_view: &resources.effect_lut_texture_3d_view,
        binding_mode: if settings.is_enabled() {
            EffectLutBindingMode::Texture2d
        } else {
            EffectLutBindingMode::Disabled
        },
    };

    let Some(texture_id) = settings
        .is_enabled()
        .then(|| settings.texture.map(|texture| texture.id()))
        .flatten()
    else {
        return fallback;
    };

    if let Some(texture_3d_view) =
        streamer.prepared_post_process_lut_3d_view(texture_id, settings.texture_layout)
    {
        return EffectLutTextureViews {
            texture_2d_view: fallback.texture_2d_view,
            texture_3d_view,
            binding_mode: EffectLutBindingMode::Texture3d,
        };
    }

    if let Some((texture_2d_view, is_strip)) =
        streamer.prepared_post_process_lut_2d_view(texture_id, settings.texture_layout)
    {
        return EffectLutTextureViews {
            texture_2d_view,
            texture_3d_view: fallback.texture_3d_view,
            binding_mode: if is_strip {
                EffectLutBindingMode::Texture2dStrip
            } else {
                EffectLutBindingMode::Texture2d
            },
        };
    }

    fallback
}
