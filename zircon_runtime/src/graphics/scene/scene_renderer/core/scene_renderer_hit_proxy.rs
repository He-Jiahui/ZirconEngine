use std::sync::{Arc, Mutex};

use crate::core::framework::render::RenderViewportPickPolicy;
use crate::core::math::UVec2;
use crate::graphics::pipeline::PipelineAdmission;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommandList, MeshDrawCommandReplayer, MeshDrawCommandStream, MeshSceneDataBindHandle,
};
use crate::graphics::scene::scene_renderer::mesh::{
    MeshHitProxyTokenSource, MeshPipelineCache, build_hit_proxy_command_list,
    coordinate_material_pipeline_publications,
};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::scene_renderer::SceneRenderer;
use super::scene_renderer_core::{SceneHitProxyTargets, f16_bits_to_f32};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SceneHitProxyProduct {
    pub(crate) token: u32,
    pub(crate) depth: f32,
    pub(crate) world_position: [f32; 3],
    pub(crate) world_normal: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SceneHitProxySubmission {
    OutsideRenderRegion,
    Submitted,
}

type SceneHitProxyCompletionCallback =
    Box<dyn FnOnce(Result<SceneHitProxyProduct, String>) + Send + 'static>;

#[derive(Clone)]
pub(crate) struct SceneHitProxyCompletion {
    callback: Arc<Mutex<Option<SceneHitProxyCompletionCallback>>>,
}

impl SceneHitProxyCompletion {
    pub(crate) fn new(callback: SceneHitProxyCompletionCallback) -> Self {
        Self {
            callback: Arc::new(Mutex::new(Some(callback))),
        }
    }

    pub(crate) fn complete(&self, result: Result<SceneHitProxyProduct, String>) -> bool {
        let callback = self
            .callback
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        let Some(callback) = callback else {
            return false;
        };
        callback(result);
        true
    }
}

impl SceneRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn submit_hit_proxy_product(
        &mut self,
        frame: &ViewportRenderFrame,
        pixel: UVec2,
        policy: RenderViewportPickPolicy,
        virtual_geometry_enabled: bool,
        hit_proxy_tokens: &dyn MeshHitProxyTokenSource,
        completion: SceneHitProxyCompletion,
    ) -> Result<SceneHitProxySubmission, GraphicsError> {
        let Some(mut frame_buffer_uploads) = self
            .core
            .prepare_hit_proxy_scene_uniform_upload(frame, pixel)
        else {
            return Ok(SceneHitProxySubmission::OutsideRenderRegion);
        };
        let device = &self.backend.device;
        let readback_frame_index = self
            .core
            .hit_proxy_resources
            .allocate_readback_frame_index()
            .ok_or_else(|| {
                GraphicsError::BufferMap(
                    "HitProxy readback frame index space is exhausted".to_string(),
                )
            })?;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-hit-proxy-encoder"),
        });
        let (hit_proxy_gpu_scene, targets) = self.core.hit_proxy_resources.parts(device);
        let mut built_mesh_draws = self
            .core
            .advanced_plugin_resources
            .build_hit_proxy_mesh_draws(
                &self.backend,
                &mut encoder,
                &self.core.material_texture_bind_group_layout,
                hit_proxy_gpu_scene,
                &mut self.streamer,
                &mut self.core.mesh_pipelines,
                frame,
                virtual_geometry_enabled,
                policy,
                hit_proxy_tokens,
            )?;
        let mut gpu_scene_upload = built_mesh_draws.take_gpu_scene_prepared_upload();
        gpu_scene_upload.append_to(hit_proxy_gpu_scene, &mut frame_buffer_uploads);

        let material_requirements = built_mesh_draws.take_material_pipeline_requirements();
        coordinate_material_pipeline_publications(
            device,
            &mut self.streamer,
            &mut self.core.mesh_pipelines,
            material_requirements,
            true,
            true,
        );
        let draws = built_mesh_draws.into_draws();
        let commands = build_hit_proxy_command_list(
            &draws,
            &mut self.core.mesh_pipelines,
            policy,
            frame.shader_quality(),
        );
        record_hit_proxy_pass(
            &mut encoder,
            device,
            targets,
            &self.core.scene_bind_group,
            hit_proxy_gpu_scene.scene_bind_group(),
            &commands,
            &mut self.core.mesh_pipelines,
            &self.streamer,
        )?;

        let diagnostic_scope = self
            .backend
            .begin_product_diagnostic_readback_scope(readback_frame_index)?;
        let accumulator = Arc::new(Mutex::new(SceneHitProxyReadbackAccumulator::new(
            completion.clone(),
        )));
        let token_admitted = self
            .backend
            .enqueue_product_diagnostic_texture_r32_uint_texel(
                &targets.token,
                [0, 0],
                Box::new({
                    let accumulator = Arc::clone(&accumulator);
                    move |result| record_token_readback(&accumulator, result)
                }),
            )?;
        let position_admitted = self
            .backend
            .enqueue_product_diagnostic_texture_rgba32float_texel(
                &targets.world_position_depth,
                [0, 0],
                Box::new({
                    let accumulator = Arc::clone(&accumulator);
                    move |result| record_position_readback(&accumulator, result)
                }),
            )?;
        let normal_admitted = self
            .backend
            .enqueue_product_diagnostic_texture_rgba16float(
                &targets.world_normal,
                0,
                0,
                1,
                1,
                Box::new({
                    let accumulator = Arc::clone(&accumulator);
                    move |result| record_normal_readback(&accumulator, result)
                }),
            )?;
        if !(token_admitted && position_admitted && normal_admitted) {
            completion.complete(Err(
                "HitProxy product exceeded the bounded diagnostic readback budget".to_string(),
            ));
            return Err(GraphicsError::BufferMap(
                "HitProxy product exceeded the bounded diagnostic readback budget".to_string(),
            ));
        }
        let diagnostic_frame = diagnostic_scope
            .prepare("zircon-hit-proxy-readback", &mut encoder)?
            .ok_or_else(|| GraphicsError::BufferMap("HitProxy readback admitted no work".into()))?;
        let _upload_submission = self
            .backend
            .enqueue_copy_buffer_upload_batch(frame_buffer_uploads)?;
        if let Err(error) = self
            .backend
            .submit_graphics_command_buffers_with_diagnostics(
                vec![encoder.finish()],
                Some(diagnostic_frame),
            )
        {
            completion.complete(Err(error.to_string()));
            return Err(error);
        }
        gpu_scene_upload.commit(hit_proxy_gpu_scene);
        Ok(SceneHitProxySubmission::Submitted)
    }
}

fn record_hit_proxy_pass(
    encoder: &mut wgpu::CommandEncoder,
    device: &wgpu::Device,
    targets: &SceneHitProxyTargets,
    scene_bind_group: &wgpu::BindGroup,
    gpu_scene_bind_group: &wgpu::BindGroup,
    commands: &MeshDrawCommandList,
    mesh_pipelines: &mut MeshPipelineCache,
    streamer: &crate::graphics::scene::resources::ResourceStreamer,
) -> Result<(), GraphicsError> {
    for command in commands.commands() {
        match mesh_pipelines.ensure_hit_proxy_pipeline_admission_for_variant(
            device,
            streamer,
            command.pipeline_variant_id,
        ) {
            PipelineAdmission::Ready(()) => {}
            PipelineAdmission::Deferred(unavailable) | PipelineAdmission::Failed(unavailable) => {
                return Err(GraphicsError::BufferMap(format!(
                    "HitProxy pipeline {:?} is unavailable: {unavailable:?}",
                    command.pipeline_variant_id.value()
                )));
            }
        }
    }

    let color_attachments = [
        Some(wgpu::RenderPassColorAttachment {
            view: &targets.token_view,
            resolve_target: None,
            depth_slice: None,
            ops: clear_color_attachment(),
        }),
        Some(wgpu::RenderPassColorAttachment {
            view: &targets.world_position_depth_view,
            resolve_target: None,
            depth_slice: None,
            ops: clear_color_attachment(),
        }),
        Some(wgpu::RenderPassColorAttachment {
            view: &targets.world_normal_view,
            resolve_target: None,
            depth_slice: None,
            ops: clear_color_attachment(),
        }),
    ];
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("zircon-hit-proxy-pass"),
        color_attachments: &color_attachments,
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &targets.depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
    pass.set_bind_group(0, scene_bind_group, &[]);
    let gpu_scene_bind_group = MeshSceneDataBindHandle::new(gpu_scene_bind_group);
    let stream = MeshDrawCommandStream::new(commands.commands(), None);
    let mut replayer = MeshDrawCommandReplayer::default();
    replayer.replay_command_stream(&mut pass, stream, |replayer, pass, command| {
        if replayer.should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id) {
            mesh_pipelines.record_bound_mesh_pass_pipeline(
                command.pipeline_kind,
                command.pipeline_variant_id,
            );
            pass.set_pipeline(
                mesh_pipelines.hit_proxy_pipeline_for_ready_variant(command.pipeline_variant_id),
            );
        }
        replayer.bind_gpu_scene_if_needed(pass, command, Some(gpu_scene_bind_group));
        if command.pipeline_key().is_alpha_mask() {
            if mesh_pipelines
                .pipeline_uses_builtin_fallback_shader(streamer, command.pipeline_key())
            {
                replayer.bind_standard_material_if_needed(pass, command);
            } else {
                replayer.bind_material_if_needed(pass, command);
            }
        }
        replayer.bind_geometry_if_needed(pass, command);
        true
    });
    Ok(())
}

fn clear_color_attachment() -> wgpu::Operations<wgpu::Color> {
    wgpu::Operations {
        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        store: wgpu::StoreOp::Store,
    }
}

struct SceneHitProxyReadbackAccumulator {
    token: Option<Result<u32, String>>,
    position_depth: Option<Result<[f32; 4], String>>,
    normal: Option<Result<[f32; 3], String>>,
    completion: SceneHitProxyCompletion,
}

impl SceneHitProxyReadbackAccumulator {
    fn new(completion: SceneHitProxyCompletion) -> Self {
        Self {
            token: None,
            position_depth: None,
            normal: None,
            completion,
        }
    }

    fn finish_if_complete(&mut self) {
        if self.token.is_none() || self.position_depth.is_none() || self.normal.is_none() {
            return;
        }
        let Some(token) = self.token.take() else {
            return;
        };
        let Some(position_depth) = self.position_depth.take() else {
            return;
        };
        let Some(normal) = self.normal.take() else {
            return;
        };
        let result = token.and_then(|token| {
            position_depth.and_then(|position_depth| {
                normal.and_then(|world_normal| {
                    let product = SceneHitProxyProduct {
                        token,
                        depth: position_depth[3],
                        world_position: [position_depth[0], position_depth[1], position_depth[2]],
                        world_normal,
                    };
                    validate_hit_proxy_product(product)
                })
            })
        });
        self.completion.complete(result);
    }
}

fn record_token_readback(
    accumulator: &Arc<Mutex<SceneHitProxyReadbackAccumulator>>,
    result: Result<u32, String>,
) {
    let mut accumulator = accumulator
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    accumulator.token = Some(result);
    accumulator.finish_if_complete();
}

fn record_position_readback(
    accumulator: &Arc<Mutex<SceneHitProxyReadbackAccumulator>>,
    result: Result<[f32; 4], String>,
) {
    let mut accumulator = accumulator
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    accumulator.position_depth = Some(result);
    accumulator.finish_if_complete();
}

fn record_normal_readback(
    accumulator: &Arc<Mutex<SceneHitProxyReadbackAccumulator>>,
    result: Result<Vec<u8>, String>,
) {
    let mut accumulator = accumulator
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    accumulator.normal = Some(result.and_then(decode_world_normal));
    accumulator.finish_if_complete();
}

fn decode_world_normal(bytes: Vec<u8>) -> Result<[f32; 3], String> {
    let bytes: [u8; 8] = bytes.try_into().map_err(|bytes: Vec<u8>| {
        format!(
            "HitProxy RGBA16F normal returned {} bytes; expected 8",
            bytes.len()
        )
    })?;
    let mut normal: [f32; 3] = std::array::from_fn(|index| {
        let offset = index * 2;
        f16_bits_to_f32(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
    });
    let length_squared = normal.iter().map(|value| value * value).sum::<f32>();
    if !length_squared.is_finite() || length_squared <= f32::EPSILON {
        return Ok([0.0; 3]);
    }
    let inverse_length = length_squared.sqrt().recip();
    for value in &mut normal {
        *value *= inverse_length;
    }
    Ok(normal)
}

fn validate_hit_proxy_product(
    product: SceneHitProxyProduct,
) -> Result<SceneHitProxyProduct, String> {
    if !product.depth.is_finite()
        || !product.world_position.iter().all(|value| value.is_finite())
        || !product.world_normal.iter().all(|value| value.is_finite())
    {
        return Err("HitProxy product contained a non-finite geometry value".to_string());
    }
    if product.token != 0
        && (product.depth < 0.0 || product.depth > 1.0 || product.world_normal == [0.0; 3])
    {
        return Err("HitProxy hit product failed its depth or normal contract".to_string());
    }
    Ok(product)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{
        SceneHitProxyCompletion, SceneHitProxyProduct, SceneHitProxyReadbackAccumulator,
        decode_world_normal, record_normal_readback, record_position_readback,
        record_token_readback, validate_hit_proxy_product,
    };

    #[test]
    fn hit_proxy_readback_completion_is_fail_closed_without_production_expect() {
        let source = include_str!("scene_renderer_hit_proxy.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("hit-proxy test boundary");

        assert!(!source.contains(".expect("));
        assert!(source.contains("let Some(token) = self.token.take()"));
    }

    #[test]
    fn hit_proxy_commits_gpu_scene_only_after_diagnostics_and_terminal_submission() {
        let source = include_str!("scene_renderer_hit_proxy.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("hit-proxy test boundary");
        let prepare_diagnostics = source
            .find("let diagnostic_frame = diagnostic_scope")
            .expect("diagnostic frame prepare");
        let enqueue_upload = source
            .find(".enqueue_copy_buffer_upload_batch(frame_buffer_uploads)?")
            .expect("GPU-scene upload enqueue");
        let terminal_submit = source
            .find(".submit_graphics_command_buffers_with_diagnostics(")
            .expect("hit-proxy terminal submission");
        let commit_upload = source
            .find("gpu_scene_upload.commit(hit_proxy_gpu_scene)")
            .expect("GPU-scene upload commit");

        assert!(prepare_diagnostics < enqueue_upload);
        assert!(enqueue_upload < terminal_submit);
        assert!(terminal_submit < commit_upload);
    }

    #[test]
    fn hit_proxy_normal_decode_preserves_signed_unit_direction() {
        let bytes = [0x00, 0xbc, 0x00, 0x00, 0x00, 0x3c, 0x00, 0x00];
        let normal = decode_world_normal(bytes.to_vec()).expect("RGBA16F normal");

        assert!((normal[0] + std::f32::consts::FRAC_1_SQRT_2).abs() < 0.001);
        assert_eq!(normal[1], 0.0);
        assert!((normal[2] - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.001);
    }

    #[test]
    fn hit_proxy_product_rejects_invalid_hit_geometry_but_accepts_clear_no_hit() {
        assert!(
            validate_hit_proxy_product(SceneHitProxyProduct {
                token: 0,
                depth: 0.0,
                world_position: [0.0; 3],
                world_normal: [0.0; 3],
            })
            .is_ok()
        );
        assert!(
            validate_hit_proxy_product(SceneHitProxyProduct {
                token: 7,
                depth: 2.0,
                world_position: [0.0; 3],
                world_normal: [0.0, 1.0, 0.0],
            })
            .is_err()
        );
    }

    #[test]
    fn hit_proxy_readback_aggregation_retains_early_channels_until_all_complete() {
        let observed = Arc::new(Mutex::new(None));
        let completion = SceneHitProxyCompletion::new(Box::new({
            let observed = Arc::clone(&observed);
            move |result| *observed.lock().unwrap() = Some(result)
        }));
        let accumulator = Arc::new(Mutex::new(SceneHitProxyReadbackAccumulator::new(
            completion,
        )));

        record_token_readback(&accumulator, Ok(0));
        assert!(observed.lock().unwrap().is_none());
        record_position_readback(&accumulator, Ok([0.0; 4]));
        assert!(observed.lock().unwrap().is_none());
        record_normal_readback(&accumulator, Ok(vec![0; 8]));

        let result = observed.lock().unwrap().take().expect("completion result");
        assert_eq!(result.expect("clear product").token, 0);
    }
}
