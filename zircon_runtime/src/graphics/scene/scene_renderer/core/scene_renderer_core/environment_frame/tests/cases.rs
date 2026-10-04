use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    RenderEnvironmentCaptureHandle, RenderEnvironmentCaptureRequest, SkyboxSettings,
    SourceCubemapEnvironment, SourceCubemapMipChain, SOURCE_CUBEMAP_MIN_FACE_SIZE,
};
use crate::core::math::UVec2;
use crate::graphics::backend::{read_texture_rgba, RenderBackend};
use crate::graphics::scene::scene_renderer::core::scene_renderer::SceneRenderer;
use crate::graphics::scene::scene_renderer::environment::EnvironmentCaptureSceneBatch;
use crate::graphics::types::GraphicsError;
use zr_rhi::{RenderQueueClass, RhiError, SubmissionStatus, SubmissionTicket};
use zr_rhi_wgpu::{WgpuBufferUploadBatch, WgpuResourceUploadBatch, WgpuTextureUploadBatch};

use super::SceneRendererCore;

#[test]
fn cubemap_rebind_rolls_back_before_upload_and_graphics_admission() {
    let mut renderer = renderer();
    let old = environment(1, 1, [1.0, 0.0, 0.0, 1.0]);
    let next = environment(2, 2, [0.0, 1.0, 0.0, 1.0]);
    submit_environment(&renderer.backend, &mut renderer.core, &old);
    let before = renderer.core.scene_environment_cubemap.upload_report();
    let old_pixels = sample_environment(&renderer.backend, &renderer.core);
    assert_eq!(old_pixels, [255, 0, 0, 255].repeat(3));

    for admit_upload in [false, true] {
        let result: Result<(), GraphicsError> = (|| {
            let mut frame = renderer.core.begin_scene_environment_frame();
            let core = frame.core();
            let (encoder, uploads, rebind) = prepare_environment(&renderer.backend, core, &next);
            assert!(rebind);
            assert_eq!(
                core.scene_environment_cubemap
                    .upload_report()
                    .committed_upload_key,
                old.texture_upload_key()
            );
            assert!(!std::ptr::eq(
                core.frame_scene_bind_group(),
                &core.scene_bind_group
            ));
            if admit_upload {
                admit_uploads(&renderer.backend, uploads);
            }
            drop(encoder);
            Err(GraphicsError::Asset(
                "injected before graphics submission".to_owned(),
            ))
        })();
        assert!(result.is_err());
        let after = renderer.core.scene_environment_cubemap.upload_report();
        assert_eq!(after.committed_upload_key, before.committed_upload_key);
        assert_eq!(after.pending_upload_key, None);
        assert_eq!(after.resident_texture_bytes, before.resident_texture_bytes);
        assert!(!renderer.core.scene_environment_cubemap.has_pending_rebind());
        assert!(renderer.core.pending_scene_environment_bindings.is_none());
        assert_eq!(
            sample_environment(&renderer.backend, &renderer.core),
            old_pixels
        );
    }

    submit_environment(&renderer.backend, &mut renderer.core, &next);
    assert_eq!(
        renderer
            .core
            .scene_environment_cubemap
            .upload_report()
            .committed_upload_key,
        next.texture_upload_key()
    );
    assert_eq!(
        sample_environment(&renderer.backend, &renderer.core),
        [0, 255, 0, 255].repeat(3)
    );
}

#[test]
fn cubemap_same_size_retry_reuses_textures_and_bindings() {
    let mut renderer = renderer();
    let old = environment(2, 1, [1.0, 0.0, 0.0, 1.0]);
    let next = environment(2, 2, [0.0, 0.0, 1.0, 1.0]);
    submit_environment(&renderer.backend, &mut renderer.core, &old);
    let resident = renderer
        .core
        .scene_environment_cubemap
        .upload_report()
        .resident_texture_bytes;
    let binding = renderer.core.scene_bind_group.clone();
    let sh9 = renderer.core.scene_environment_sh9_buffer.clone();
    let staging = renderer.core.scene_environment_sh9_staging_buffer.clone();
    for admit_upload in [false, true] {
        {
            let mut frame = renderer.core.begin_scene_environment_frame();
            let core = frame.core();
            let (encoder, uploads, rebind) = prepare_environment(&renderer.backend, core, &next);
            assert!(!rebind);
            assert!(core.pending_scene_environment_bindings.is_none());
            assert_eq!(core.frame_scene_bind_group(), &binding);
            assert_eq!(core.frame_environment_sh9_buffer(), &sh9);
            assert_eq!(core.scene_environment_sh9_staging_buffer, staging);
            if admit_upload {
                let ticket = admit_uploads(&renderer.backend, uploads);
                assert_eq!(
                    renderer.backend.submission_status(ticket).unwrap(),
                    SubmissionStatus::Accepted
                );
            }
            drop(encoder);
        }
        assert_eq!(
            renderer
                .core
                .scene_environment_cubemap
                .upload_report()
                .committed_upload_key,
            old.texture_upload_key()
        );
        assert_eq!(
            sample_environment(&renderer.backend, &renderer.core),
            [255, 0, 0, 255].repeat(3)
        );
    }
    submit_environment(&renderer.backend, &mut renderer.core, &next);
    assert_eq!(renderer.core.scene_bind_group, binding);
    assert_eq!(renderer.core.scene_environment_sh9_buffer, sh9);
    assert_eq!(renderer.core.scene_environment_sh9_staging_buffer, staging);
    assert_eq!(
        renderer
            .core
            .scene_environment_cubemap
            .upload_report()
            .resident_texture_bytes,
        resident
    );
    assert_eq!(
        sample_environment(&renderer.backend, &renderer.core),
        [0, 0, 255, 255].repeat(3)
    );
}

#[test]
fn capture_graphics_backpressure_cancels_admitted_uploads_and_preserves_last_good() {
    let mut renderer = renderer();
    let old = environment(2, 1, [1.0, 0.0, 0.0, 1.0]);
    let next = environment(2, 2, [0.0, 1.0, 0.0, 1.0]);
    submit_environment(&renderer.backend, &mut renderer.core, &old);
    let old_pixels = sample_environment(&renderer.backend, &renderer.core);
    renderer.backend.poll_submission_completions().unwrap();

    let mut scene = crate::scene::World::new().to_render_snapshot();
    scene.environment.skybox = SkyboxSettings::source_cubemap(next);
    let request = RenderEnvironmentCaptureRequest::new("backpressure", [0.0; 3], 1)
        .unwrap()
        .with_face_size(SOURCE_CUBEMAP_MIN_FACE_SIZE)
        .unwrap();
    let scene_batch = EnvironmentCaptureSceneBatch::new(scene, request);
    let capacity = renderer
        .backend
        .device_profile()
        .submission_limits()
        .max_unresolved_submissions();
    let mut occupied = Vec::with_capacity(capacity - 1);
    for _ in 0..capacity - 1 {
        let encoder = renderer
            .backend
            .device
            .create_command_encoder(&Default::default());
        occupied.push(
            renderer
                .backend
                .enqueue_graphics_command_buffers(vec![encoder.finish()])
                .unwrap(),
        );
    }
    let preceding = occupied.last().copied().unwrap();
    let upload = SubmissionTicket::new(
        preceding.device_id(),
        preceding.generation(),
        RenderQueueClass::Copy,
        preceding.sequence() + 1,
    );
    let error = renderer
        .submit_environment_capture_scene_batch(
            RenderEnvironmentCaptureHandle::new(1).unwrap(),
            scene_batch,
        )
        .err()
        .expect("capture graphics admission must reject after the upload takes the last slot");
    assert!(
        matches!(
            error,
            GraphicsError::Rhi(RhiError::SubmissionBackpressure { .. })
        ),
        "{error:?}"
    );
    assert_eq!(
        renderer.backend.submission_status(upload).unwrap(),
        SubmissionStatus::Cancelled
    );
    assert_eq!(
        renderer
            .core
            .scene_environment_cubemap
            .upload_report()
            .committed_upload_key,
        old.texture_upload_key()
    );
    assert_eq!(
        renderer
            .core
            .scene_environment_cubemap
            .upload_report()
            .pending_upload_key,
        None
    );
    assert!(renderer.core.pending_scene_environment_bindings.is_none());
    let settled = renderer
        .backend
        .settle_abandoned_submissions(&occupied)
        .unwrap();
    assert!(settled
        .iter()
        .all(|status| *status == SubmissionStatus::Cancelled));
    assert_eq!(
        sample_environment(&renderer.backend, &renderer.core),
        old_pixels
    );
}

#[test]
fn environment_frame_guard_wraps_every_submission_entrypoint() {
    for source in [
        include_str!("../../../scene_renderer_core_render_scene/render_scene.rs"),
        include_str!("../../../scene_renderer_core_render_compiled_scene/render/render.rs"),
        include_str!("../../../scene_renderer_environment_capture.rs"),
    ] {
        let guard = source.find("begin_scene_environment_frame()").unwrap();
        let prepare = source
            .find(".write_scene_uniform(")
            .or_else(|| source.find(".prepare_compiled_scene_frame_foundation("))
            .or_else(|| source.find(".scene_environment_cubemap.ensure_uploaded("))
            .unwrap();
        assert!(guard < prepare);
    }
    for source in [
        include_str!("../../../scene_renderer_core_render_scene/render_scene.rs"),
        include_str!(
            "../../../scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs"
        ),
        include_str!("../../../scene_renderer_environment_capture.rs"),
    ] {
        let submit = source
            .find(".submit_graphics_command_buffers_with_")
            .unwrap();
        let commit = source.find(".commit_scene_environment_frame()").unwrap();
        assert!(submit < commit);
    }
}

fn renderer() -> SceneRenderer {
    SceneRenderer::new_for_test(Arc::new(ProjectAssetManager::default()))
        .expect("environment transaction regression requires a WGPU renderer")
}

fn environment(size: u32, revision: u64, color: [f32; 4]) -> SourceCubemapEnvironment {
    let texels = vec![color; (size * size * 6) as usize];
    let mips = SourceCubemapMipChain::new(size, 1, texels.clone(), size, 1, texels);
    let mut environment = SourceCubemapEnvironment::new(mips, revision, [revision as u32; 4]);
    environment.irradiance_sh9 = [color; 9];
    environment.with_prepared_upload_artifact()
}

fn prepare_environment(
    backend: &RenderBackend,
    core: &mut SceneRendererCore,
    environment: &SourceCubemapEnvironment,
) -> (wgpu::CommandEncoder, WgpuBufferUploadBatch, bool) {
    let mut encoder = backend.device.create_command_encoder(&Default::default());
    let mut uploads = WgpuBufferUploadBatch::new();
    let rebind = core
        .scene_environment_cubemap
        .ensure_uploaded(&backend.device, &mut encoder, environment, &mut uploads)
        .unwrap();
    if rebind {
        core.prepare_static_environment_bindings(&backend.device);
    }
    let payload: Arc<[u8]> = Arc::from(bytemuck::bytes_of(&environment.irradiance_sh9));
    let source_range = 0..payload.len();
    core.encode_environment_sh9_upload(
        &backend.device,
        &mut encoder,
        payload,
        source_range,
        &mut uploads,
    )
    .unwrap();
    (encoder, uploads, rebind)
}

fn admit_uploads(backend: &RenderBackend, uploads: WgpuBufferUploadBatch) -> SubmissionTicket {
    backend
        .enqueue_copy_resource_upload_batch(WgpuResourceUploadBatch::from_batches(
            uploads,
            WgpuTextureUploadBatch::new(),
        ))
        .unwrap()
}

fn submit_environment(
    backend: &RenderBackend,
    core: &mut SceneRendererCore,
    environment: &SourceCubemapEnvironment,
) {
    let mut frame = core.begin_scene_environment_frame();
    let core = frame.core();
    let (encoder, uploads, _) = prepare_environment(backend, core, environment);
    admit_uploads(backend, uploads);
    backend
        .submit_graphics_command_buffers_with_diagnostics(vec![encoder.finish()], None)
        .unwrap();
    core.commit_scene_environment_frame();
}

fn sample_environment(backend: &RenderBackend, core: &SceneRendererCore) -> Vec<u8> {
    let device = &backend.device;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("environment-generation-sample"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(r#"
struct Sh9 { coefficients: array<vec4<f32>, 9> }
@group(0) @binding(1) var source: texture_cube<f32>;
@group(0) @binding(2) var cube_sampler: sampler;
@group(0) @binding(4) var specular: texture_cube<f32>;
@group(0) @binding(6) var<uniform> sh9: Sh9;
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    return vec4(positions[i], 0.0, 1.0);
}
@fragment fn fs(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    if position.x < 1.0 { return textureSampleLevel(source, cube_sampler, vec3(1.0, 0.0, 0.0), 0.0); }
    if position.x < 2.0 { return textureSampleLevel(specular, cube_sampler, vec3(1.0, 0.0, 0.0), 0.0); }
    return sh9.coefficients[0];
}
"#)),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("environment-generation-sample"),
        bind_group_layouts: &[Some(&core.scene_bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("environment-generation-sample"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("environment-generation-sample"),
        size: wgpu::Extent3d {
            width: 3,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("environment-generation-sample"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &core.scene_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    backend
        .submit_graphics_command_buffers_with_diagnostics(vec![encoder.finish()], None)
        .unwrap();
    read_texture_rgba(device, &backend.queue, &texture, UVec2::new(3, 1)).unwrap()
}
