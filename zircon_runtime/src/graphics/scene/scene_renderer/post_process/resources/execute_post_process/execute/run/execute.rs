use crate::core::framework::render::RenderPipelinePhase;
use crate::core::math::UVec2;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::super::build_post_process_params::build_post_process_params_with_hybrid_gi_policy;
use super::super::create_bind_group::create_bind_group;
use super::super::post_process_params_upload;
use super::super::prepare_scene_data_uploads;
use super::record_pass::record_pass;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_post_process(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        cluster_dimensions: UVec2,
        scene_color_origin: [u32; 2],
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        motion_vector_neighbor_max_view: &wgpu::TextureView,
        scene_normal_view: &wgpu::TextureView,
        scene_material_view: Option<&wgpu::TextureView>,
        ao_view: &wgpu::TextureView,
        contact_shadow_view: &wgpu::TextureView,
        previous_scene_color_view: Option<&wgpu::TextureView>,
        current_hybrid_gi_lighting_view: Option<&wgpu::TextureView>,
        previous_global_illumination_view: Option<&wgpu::TextureView>,
        previous_screen_space_reflection_history_view: Option<&wgpu::TextureView>,
        bloom_view: &wgpu::TextureView,
        depth_of_field_coc_view: &wgpu::TextureView,
        depth_of_field_bokeh_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        global_illumination_view: &wgpu::TextureView,
        screen_space_reflection_history_view: &wgpu::TextureView,
        screen_space_reflection_specular_occlusion_view: &wgpu::TextureView,
        baked_color_lut_view: Option<&wgpu::TextureView>,
        cluster_buffer: wgpu::BufferBinding<'_>,
        exposure_buffer: wgpu::BufferBinding<'_>,
        frame: &ViewportRenderFrame,
        streamer: &ResourceStreamer,
        features: SceneRuntimeFeatureFlags,
        temporal_history_available: bool,
        hybrid_gi_history_available: bool,
        skip_depth_of_field: bool,
        skip_motion_blur: bool,
        skip_blur: bool,
        skip_scene_composite: bool,
    ) -> WgpuBufferUploadBatch {
        let extract = &frame.extract;
        let (
            reflection_probe_count,
            hybrid_gi_probe_count,
            scheduled_trace_region_count,
            mut uploads,
        ) = prepare_scene_data_uploads(
            self,
            frame,
            viewport_size,
            features.reflection_probes_enabled,
            features.hybrid_global_illumination_enabled,
        );
        let effect_lut =
            select_effect_lut_texture_views(self, streamer, frame, baked_color_lut_view);
        let render_region = frame
            .render_region_for_phase(RenderPipelinePhase::DisplayMapping)
            .expect("uber post process requires the display-mapping phase");
        let local_viewport_size = render_region.local_size();
        let hybrid_gi_composite_policy = frame
            .prepared_runtime_sidebands()
            .hybrid_gi_prepared_frame()
            .map(|prepared| prepared.composite_policy)
            .unwrap_or_default();
        let mut params = build_post_process_params_with_hybrid_gi_policy(
            local_viewport_size,
            cluster_dimensions,
            render_region,
            scene_color_origin,
            extract,
            frame.post_process(),
            features,
            temporal_history_available,
            hybrid_gi_history_available,
            reflection_probe_count,
            hybrid_gi_probe_count,
            scheduled_trace_region_count,
            current_hybrid_gi_lighting_view.is_some(),
            hybrid_gi_composite_policy,
        );
        if skip_depth_of_field {
            params.effect_blur_dof[1] = 0.0;
            params.effect_blur_dof[2] = 0.0;
            params.effect_blur_dof[3] = 0.0;
            params.effect_dof_lens = [0.0; 4];
        }
        if skip_blur {
            params.effect_blur_dof[0] = 0.0;
        }
        if skip_motion_blur {
            params.effect_motion_blur = [0.0; 4];
        }
        if skip_scene_composite {
            params.effect_chromatic_fog[2] = 0.0;
            params.effect_chromatic_fog[3] = 0.0;
            params.effect_dither_ssr[2] = 0.0;
        }
        params.effect_flags[1] = effect_lut.binding_mode.shader_id();
        let params_buffer = &self.post_process_pass_parameter_buffers.post_process;
        let mut params_uploads = post_process_params_upload(params_buffer, &params);
        uploads.append(&mut params_uploads);
        let resolved_screen_space_reflection_history_view = if skip_scene_composite {
            &self.black_texture_view
        } else {
            screen_space_reflection_history_view
        };

        let bind_group = create_bind_group(
            self,
            device,
            params_buffer,
            scene_color_view,
            scene_depth_view,
            motion_vector_neighbor_max_view,
            scene_normal_view,
            scene_material_view,
            ao_view,
            contact_shadow_view,
            previous_scene_color_view,
            current_hybrid_gi_lighting_view.or(previous_global_illumination_view),
            previous_screen_space_reflection_history_view,
            Some(resolved_screen_space_reflection_history_view),
            Some(screen_space_reflection_specular_occlusion_view),
            None,
            None,
            None,
            None,
            bloom_view,
            depth_of_field_coc_view,
            depth_of_field_bokeh_view,
            effect_lut.texture_2d_view,
            effect_lut.texture_3d_view,
            cluster_buffer,
            exposure_buffer,
        );
        record_pass(
            self,
            encoder,
            final_color_view,
            global_illumination_view,
            &bind_group,
            render_region,
        );
        uploads
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
    BakedColorLut3d,
}

impl EffectLutBindingMode {
    fn shader_id(self) -> u32 {
        match self {
            Self::Disabled => 0,
            Self::Texture2d => 1,
            Self::Texture2dStrip => 2,
            Self::Texture3d => 3,
            Self::BakedColorLut3d => 4,
        }
    }
}

fn select_effect_lut_texture_views<'a>(
    resources: &'a ScenePostProcessResources,
    streamer: &'a ResourceStreamer,
    frame: &ViewportRenderFrame,
    baked_color_lut_view: Option<&'a wgpu::TextureView>,
) -> EffectLutTextureViews<'a> {
    let settings = frame.post_process().effect_stack.color_lookup;
    let fallback = EffectLutTextureViews {
        texture_2d_view: &resources.effect_lut_texture_view,
        texture_3d_view: &resources.effect_lut_texture_3d_view,
        binding_mode: if settings.is_enabled() {
            EffectLutBindingMode::Texture2d
        } else {
            EffectLutBindingMode::Disabled
        },
    };

    if let Some(texture_id) = settings
        .is_enabled()
        .then(|| settings.texture.map(|texture| texture.id()))
        .flatten()
    {
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
    }

    if let Some(texture_3d_view) = baked_color_lut_view {
        return EffectLutTextureViews {
            texture_2d_view: fallback.texture_2d_view,
            texture_3d_view,
            binding_mode: EffectLutBindingMode::BakedColorLut3d,
        };
    }

    fallback
}

#[cfg(test)]
mod tests {
    #[test]
    fn post_process_scene_data_and_params_share_the_pass_upload_transaction() {
        let source = include_str!("execute.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("post-process execute source");

        assert!(!production.contains("queue.write_buffer"));
        assert!(!production.contains("queue: &wgpu::Queue"));
        let scene_data = production
            .find("prepare_scene_data_uploads(")
            .expect("scene-data preparation");
        let params = production
            .find("post_process_params_upload(")
            .expect("parameter preparation");
        let append = production.find("uploads.append(").expect("batch append");
        assert!(scene_data < params && params < append);
    }
}
