use bytemuck::Zeroable;
use zircon_math::{view_matrix, Mat4, UVec2};
use zircon_scene::{ProjectionMode, RenderFrameExtract};

use super::super::constants::MAX_REFLECTION_PROBES;
use super::super::hybrid_gi_probe_gpu::GpuHybridGiProbe;
use super::super::post_process_params::PostProcessParams;
use super::super::reflection_probe_gpu::GpuReflectionProbe;
use super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use crate::types::EditorOrRuntimeFrame;

const MAX_HYBRID_GI_PROBES: usize = 16;

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
        ao_view: &wgpu::TextureView,
        previous_scene_color_view: Option<&wgpu::TextureView>,
        bloom_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        cluster_buffer: &wgpu::Buffer,
        frame: &EditorOrRuntimeFrame,
        features: SceneRuntimeFeatureFlags,
        history_available: bool,
    ) {
        let extract = &frame.extract;
        let color_grading = if features.color_grading_enabled {
            extract.post_process.color_grading
        } else {
            zircon_scene::RenderColorGradingSettings::default()
        };
        let baked_lighting = if features.baked_lighting_enabled {
            extract.lighting.baked_lighting.unwrap_or_default()
        } else {
            zircon_scene::RenderBakedLightingExtract::default()
        };
        let (reflection_probes, reflection_probe_count) =
            encode_reflection_probes(extract, viewport_size, features.reflection_probes_enabled);
        queue.write_buffer(
            &self.reflection_probe_buffer,
            0,
            bytemuck::cast_slice(&reflection_probes),
        );
        let (hybrid_gi_probes, hybrid_gi_probe_count, scheduled_trace_region_count) =
            encode_hybrid_gi_probes(frame, features.hybrid_global_illumination_enabled);
        queue.write_buffer(
            &self.hybrid_gi_probe_buffer,
            0,
            bytemuck::cast_slice(&hybrid_gi_probes),
        );

        let params = PostProcessParams {
            viewport_and_clusters: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                cluster_dimensions.x.max(1),
                cluster_dimensions.y.max(1),
            ],
            feature_flags: [
                u32::from(features.ssao_enabled),
                u32::from(features.clustered_lighting_enabled),
                u32::from(features.history_resolve_enabled && history_available),
                reflection_probe_count,
            ],
            hybrid_gi_counts: [hybrid_gi_probe_count, scheduled_trace_region_count, 0, 0],
            blends: [
                0.24,
                0.42,
                0.28,
                if features.bloom_enabled {
                    extract.post_process.bloom.intensity.max(0.0)
                } else {
                    0.0
                },
            ],
            grading: [
                color_grading.exposure.max(0.0),
                color_grading.contrast.max(0.0),
                color_grading.saturation.max(0.0),
                color_grading.gamma.max(0.001),
            ],
            tint_and_probe: [
                color_grading.tint.x.max(0.0),
                color_grading.tint.y.max(0.0),
                color_grading.tint.z.max(0.0),
                if features.reflection_probes_enabled {
                    0.35
                } else {
                    0.0
                },
            ],
            hybrid_gi_color_and_intensity: [
                0.32,
                0.38,
                0.46,
                if features.hybrid_global_illumination_enabled && hybrid_gi_probe_count > 0 {
                    0.12
                } else {
                    0.0
                },
            ],
            baked_color_and_intensity: [
                baked_lighting.color.x.max(0.0),
                baked_lighting.color.y.max(0.0),
                baked_lighting.color.z.max(0.0),
                baked_lighting.intensity.max(0.0),
            ],
        };
        queue.write_buffer(
            &self.post_process_params_buffer,
            0,
            bytemuck::bytes_of(&params),
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-post-process-bind-group"),
            layout: &self.post_process_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(ao_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        previous_scene_color_view.unwrap_or(&self.black_texture_view),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(bloom_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.post_process_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: cluster_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.reflection_probe_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.hybrid_gi_probe_buffer.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("PostProcessPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: final_color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.post_process_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn encode_reflection_probes(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuReflectionProbe; MAX_REFLECTION_PROBES], u32) {
    let mut probes = [GpuReflectionProbe::zeroed(); MAX_REFLECTION_PROBES];
    if !enabled {
        return (probes, 0);
    }

    let camera = &extract.view.camera;
    let aspect = viewport_size.x.max(1) as f32 / viewport_size.y.max(1) as f32;
    let projection = match camera.projection_mode {
        ProjectionMode::Perspective => Mat4::perspective_rh(
            camera.fov_y_radians,
            aspect.max(0.001),
            camera.z_near.max(0.001),
            camera.z_far,
        ),
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let half_width = half_height * aspect.max(0.001);
            Mat4::orthographic_rh(
                -half_width,
                half_width,
                -half_height,
                half_height,
                camera.z_near.max(0.001),
                camera.z_far,
            )
        }
    };
    let view_proj = projection * view_matrix(camera.transform);
    let camera_position = camera.transform.translation;
    let mut count = 0;

    for probe in extract
        .lighting
        .reflection_probes
        .iter()
        .take(MAX_REFLECTION_PROBES)
    {
        let clip = view_proj * probe.position.extend(1.0);
        if clip.w.abs() <= f32::EPSILON {
            continue;
        }
        let ndc = clip.truncate() / clip.w;
        if ndc.z < -1.0 || ndc.z > 1.0 {
            continue;
        }
        let uv_x = (0.5 + ndc.x * 0.5).clamp(0.0, 1.0);
        let uv_y = (0.5 - ndc.y * 0.5).clamp(0.0, 1.0);
        let distance = (camera_position - probe.position).length().max(1.0);
        let radius = (probe.radius.max(0.05) / distance).clamp(0.04, 0.6);
        probes[count] = GpuReflectionProbe {
            screen_uv_and_radius: [uv_x, uv_y, radius, 0.0],
            color_and_intensity: [
                probe.color.x.max(0.0),
                probe.color.y.max(0.0),
                probe.color.z.max(0.0),
                probe.intensity.max(0.0),
            ],
        };
        count += 1;
    }

    (probes, count as u32)
}

fn encode_hybrid_gi_probes(
    frame: &EditorOrRuntimeFrame,
    enabled: bool,
) -> ([GpuHybridGiProbe; MAX_HYBRID_GI_PROBES], u32, u32) {
    let mut probes = [GpuHybridGiProbe::zeroed(); MAX_HYBRID_GI_PROBES];
    if !enabled {
        return (probes, 0, 0);
    }

    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return (probes, 0, 0);
    };

    let mut count = 0;
    for probe in prepare.resident_probes.iter().take(MAX_HYBRID_GI_PROBES) {
        probes[count] = GpuHybridGiProbe {
            slot_and_budget: [probe.slot as f32, probe.ray_budget as f32, 0.0, 0.0],
            irradiance_and_weight: [
                probe.irradiance_rgb[0] as f32 / 255.0,
                probe.irradiance_rgb[1] as f32 / 255.0,
                probe.irradiance_rgb[2] as f32 / 255.0,
                (probe.ray_budget as f32 / 128.0).clamp(0.25, 1.0),
            ],
        };
        count += 1;
    }

    (
        probes,
        count as u32,
        prepare.scheduled_trace_region_ids.len() as u32,
    )
}
