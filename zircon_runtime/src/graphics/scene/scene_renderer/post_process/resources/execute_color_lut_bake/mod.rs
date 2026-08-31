use crate::core::framework::render::{COLOR_LUT_SIZE_DEFAULT, RenderPipelinePhase};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::post_process::cluster_dimensions_for_size;
use crate::graphics::types::ViewportRenderFrame;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::super::params::color_lut_bake_params::ColorLutBakeParams;
use super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::execute_post_process::build_post_process_params;

const COLOR_LUT_BAKE_WORKGROUP_SIZE: u32 = 4;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer) fn execute_color_lut_bake(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_lut_view: &wgpu::TextureView,
        exposure_buffer: wgpu::BufferBinding<'_>,
        frame: &ViewportRenderFrame,
        streamer: &ResourceStreamer,
    ) -> ([u32; 3], WgpuBufferUploadBatch) {
        let user_lut = select_user_lut_texture_views(self, streamer, frame);
        let params = color_lut_bake_params(frame, user_lut.binding_mode.shader_id());
        let params_uploads = WgpuBufferUploadBatch::from(WgpuBufferUpload::from_bytes(
            self.color_lut_bake_params_buffer.clone(),
            0,
            bytemuck::bytes_of(&params),
        ));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-color-lut-bake-bind-group"),
            layout: &self.color_lut_bake_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.color_lut_bake_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(exposure_buffer),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(user_lut.texture_2d_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(user_lut.texture_3d_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.effect_lut_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(color_lut_view),
                },
            ],
        });

        let dispatch_groups = color_lut_bake_dispatch_groups();
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ColorLutBakePass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.color_lut_bake_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch_groups[0], dispatch_groups[1], dispatch_groups[2]);
        (dispatch_groups, params_uploads)
    }
}

pub(in crate::graphics::scene::scene_renderer) fn color_lut_bake_workgroup_size() -> [u32; 3] {
    [
        COLOR_LUT_BAKE_WORKGROUP_SIZE,
        COLOR_LUT_BAKE_WORKGROUP_SIZE,
        COLOR_LUT_BAKE_WORKGROUP_SIZE,
    ]
}

pub(in crate::graphics::scene::scene_renderer) fn color_lut_bake_dispatch_groups() -> [u32; 3] {
    [
        COLOR_LUT_SIZE_DEFAULT.div_ceil(COLOR_LUT_BAKE_WORKGROUP_SIZE),
        COLOR_LUT_SIZE_DEFAULT.div_ceil(COLOR_LUT_BAKE_WORKGROUP_SIZE),
        COLOR_LUT_SIZE_DEFAULT.div_ceil(COLOR_LUT_BAKE_WORKGROUP_SIZE),
    ]
}

struct UserLutTextureViews<'a> {
    texture_2d_view: &'a wgpu::TextureView,
    texture_3d_view: &'a wgpu::TextureView,
    binding_mode: UserLutBindingMode,
}

#[derive(Clone, Copy)]
enum UserLutBindingMode {
    Disabled,
    Texture2d,
    Texture2dStrip,
    Texture3d,
}

impl UserLutBindingMode {
    fn shader_id(self) -> u32 {
        match self {
            Self::Disabled => 0,
            Self::Texture2d => 1,
            Self::Texture2dStrip => 2,
            Self::Texture3d => 3,
        }
    }
}

fn select_user_lut_texture_views<'a>(
    resources: &'a ScenePostProcessResources,
    streamer: &'a ResourceStreamer,
    frame: &ViewportRenderFrame,
) -> UserLutTextureViews<'a> {
    let settings = frame.post_process().effect_stack.color_lookup;
    let fallback = UserLutTextureViews {
        texture_2d_view: &resources.effect_lut_texture_view,
        texture_3d_view: &resources.effect_lut_texture_3d_view,
        binding_mode: if settings.is_enabled() {
            UserLutBindingMode::Texture2d
        } else {
            UserLutBindingMode::Disabled
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
        return UserLutTextureViews {
            texture_2d_view: fallback.texture_2d_view,
            texture_3d_view,
            binding_mode: UserLutBindingMode::Texture3d,
        };
    }

    if let Some((texture_2d_view, is_strip)) =
        streamer.prepared_post_process_lut_2d_view(texture_id, settings.texture_layout)
    {
        return UserLutTextureViews {
            texture_2d_view,
            texture_3d_view: fallback.texture_3d_view,
            binding_mode: if is_strip {
                UserLutBindingMode::Texture2dStrip
            } else {
                UserLutBindingMode::Texture2d
            },
        };
    }

    fallback
}

fn color_lut_bake_params(
    frame: &ViewportRenderFrame,
    user_lut_binding_mode: u32,
) -> ColorLutBakeParams {
    let extract = &frame.extract;
    let render_region = frame
        .render_region_for_phase(RenderPipelinePhase::DisplayMapping)
        .expect("color LUT bake requires the display-mapping phase");
    let params = build_post_process_params(
        render_region.local_size(),
        cluster_dimensions_for_size(render_region.local_size()),
        render_region,
        [0, 0],
        extract,
        frame.post_process(),
        SceneRuntimeFeatureFlags {
            color_grading_enabled: true,
            ..SceneRuntimeFeatureFlags::default()
        },
        false,
        0,
        0,
        0,
        false,
    );
    let effect_stack = frame.post_process().effect_stack;

    ColorLutBakeParams {
        lut_size_and_flags: [
            COLOR_LUT_SIZE_DEFAULT,
            user_lut_binding_mode,
            effect_stack.tonemap.render_operator_id(),
            0,
        ],
        tonemap_lut: [
            effect_stack.tonemap.render_exposure_bias(),
            effect_stack.tonemap.render_white_point(),
            effect_stack.color_lookup.render_intensity(),
            0.0,
        ],
        grading: params.grading,
        tint_and_exposure: [
            params.tint_and_probe[0],
            params.tint_and_probe[1],
            params.tint_and_probe[2],
            0.0,
        ],
    }
}

#[cfg(test)]
pub(in crate::graphics::scene::scene_renderer) fn color_transform_requires_lut_bake(
    color_grading: crate::core::framework::render::RenderColorGradingSettings,
    effect_stack: crate::core::framework::render::RenderPostProcessEffectStackSettings,
) -> bool {
    color_grading != crate::core::framework::render::RenderColorGradingSettings::default()
        || effect_stack.tonemap.is_enabled()
        || effect_stack.color_lookup.is_enabled()
}

#[cfg(test)]
pub(in crate::graphics::scene::scene_renderer) fn color_lookup_layout_binding_mode(
    layout: crate::core::framework::render::RenderColorLookupTextureLayout,
) -> u32 {
    match layout {
        crate::core::framework::render::RenderColorLookupTextureLayout::Auto => 1,
        crate::core::framework::render::RenderColorLookupTextureLayout::Texture2dStrip {
            ..
        } => 2,
        crate::core::framework::render::RenderColorLookupTextureLayout::Texture3d { .. } => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        color_lookup_layout_binding_mode, color_lut_bake_dispatch_groups,
        color_lut_bake_workgroup_size, color_transform_requires_lut_bake,
    };
    use crate::core::framework::render::{
        COLOR_LUT_SIZE_DEFAULT, RenderColorGradingSettings, RenderColorLookupSettings,
        RenderColorLookupTextureLayout, RenderPostProcessEffectStackSettings,
        RenderTonemapOperator, RenderTonemapSettings,
    };

    #[test]
    fn color_lut_bake_dispatch_covers_default_lut_volume() {
        assert_eq!(color_lut_bake_workgroup_size(), [4, 4, 4]);
        assert_eq!(color_lut_bake_dispatch_groups(), [8, 8, 8]);
        assert_eq!(COLOR_LUT_SIZE_DEFAULT, 32);
    }

    #[test]
    fn color_lut_params_are_returned_as_pre_submit_uploads() {
        let source = include_str!("mod.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("color LUT production source");

        assert!(!production.contains("queue.write_buffer"));
        assert!(production.contains("WgpuBufferUpload::from_bytes("));
        assert!(production.contains("WgpuBufferUploadBatch"));
    }

    #[test]
    fn color_lut_bake_is_required_for_all_color_transform_sources() {
        assert!(!color_transform_requires_lut_bake(
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings::default(),
        ));
        assert!(color_transform_requires_lut_bake(
            RenderColorGradingSettings {
                exposure: 1.1,
                ..Default::default()
            },
            RenderPostProcessEffectStackSettings::default(),
        ));
        assert!(color_transform_requires_lut_bake(
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings {
                tonemap: RenderTonemapSettings {
                    operator: RenderTonemapOperator::Aces,
                    ..Default::default()
                },
                ..Default::default()
            },
        ));
        assert!(color_transform_requires_lut_bake(
            RenderColorGradingSettings::default(),
            RenderPostProcessEffectStackSettings {
                color_lookup: RenderColorLookupSettings {
                    intensity: 0.5,
                    ..Default::default()
                },
                ..Default::default()
            },
        ));
    }

    #[test]
    fn color_lookup_layout_binding_modes_match_shader_contract() {
        assert_eq!(
            color_lookup_layout_binding_mode(RenderColorLookupTextureLayout::Auto),
            1
        );
        assert_eq!(
            color_lookup_layout_binding_mode(RenderColorLookupTextureLayout::Texture2dStrip {
                size: 32
            }),
            2
        );
        assert_eq!(
            color_lookup_layout_binding_mode(RenderColorLookupTextureLayout::Texture3d {
                size: 32
            }),
            3
        );
    }
}
