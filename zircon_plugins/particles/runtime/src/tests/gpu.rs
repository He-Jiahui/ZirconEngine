use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use zircon_runtime::core::framework::render::RenderParticleGpuReadbackOutputs;
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::graphics::RenderPassBufferUploadSink;

struct QueueUploadSink<'a>(&'a wgpu::Queue);

impl RenderPassBufferUploadSink for QueueUploadSink<'_> {
    fn write_buffer(&mut self, buffer: &wgpu::Buffer, offset: u64, bytes: &[u8]) {
        self.0.write_buffer(buffer, offset, bytes);
    }
}

use crate::{
    compile_particle_gpu_layout, compile_particle_gpu_program, ParticleBurst, ParticleColorKey,
    ParticleEmitterAsset, ParticleGpuBackend, ParticleGpuCounterReadback,
    ParticleGpuCpuParityReport, ParticleGpuEmitterFrameParams, ParticleGpuFrameParams,
    ParticleGpuFramePlanner, ParticleGpuPassKind, ParticleGpuReadbackRequest,
    ParticleGpuRuntimeOwner, ParticleGpuTransparentRenderConfig,
    ParticleGpuTransparentRenderParams, ParticleShape, ParticleSimulationBackend,
    ParticleSystemAsset, ParticleSystemComponent, ParticleVec3Range, ParticlesManager,
    PARTICLE_GPU_MAX_PARTICLES,
};

use super::support::spawn_rate_asset;

#[test]
fn gpu_backend_uses_shared_layout_and_records_cpu_fallback() {
    let asset = ParticleSystemAsset::new("gpu")
        .with_backend(ParticleSimulationBackend::Gpu)
        .with_emitters(vec![
            ParticleEmitterAsset::sprite("gpu").with_max_particles(64)
        ]);
    let layout = compile_particle_gpu_layout(&asset);
    assert_eq!(layout.capacity, 64);
    assert!(layout
        .attributes
        .iter()
        .any(|attribute| attribute.name == "position"));
    assert!(layout
        .attributes
        .iter()
        .any(|attribute| attribute.name == "previous_position"));
    assert!(layout
        .attributes
        .iter()
        .any(|attribute| attribute.name == "initial_size"));
    assert!(layout
        .attributes
        .iter()
        .any(|attribute| attribute.name == "start_color"));
    assert_eq!(layout.total_words, 64 * layout.stride_words as u64);

    let program = compile_particle_gpu_program(&asset);
    assert_eq!(
        program
            .passes
            .iter()
            .map(|pass| pass.kind)
            .collect::<Vec<_>>(),
        vec![
            ParticleGpuPassKind::SpawnUpdate,
            ParticleGpuPassKind::CompactAlive,
            ParticleGpuPassKind::BuildIndirectArgs,
            ParticleGpuPassKind::TransparentRender
        ]
    );
    assert!(program.shader.wgsl.contains("fn particle_spawn_update"));
    naga::front::wgsl::parse_str(&program.shader.wgsl)
        .expect("generated particle GPU WGSL should parse");
    assert!(program
        .shader
        .transparent_wgsl
        .contains("fn particle_gpu_transparent_vs"));
    naga::front::wgsl::parse_str(&program.shader.transparent_wgsl)
        .expect("generated particle transparent WGSL should parse");

    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(5, asset))
        .unwrap();
    let snapshot = manager.snapshot();
    assert!(snapshot.emitters[0].fallback_to_cpu);
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("CPU simulation is active")));
}

#[test]
fn gpu_particle_extract_projects_neutral_gpu_frame_for_renderer_graph() {
    let asset = ParticleSystemAsset::new("gpu-extract")
        .with_backend(ParticleSimulationBackend::Gpu)
        .with_emitters(vec![ParticleEmitterAsset::sprite("gpu")
            .with_spawn_rate(0.0)
            .with_burst(ParticleBurst::new(0.0, 2))
            .with_max_particles(8)]);
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(6, asset))
        .unwrap();
    manager.tick(0.001).unwrap();

    let extract = manager.build_extract(None);
    let gpu_frame = extract
        .gpu_frame
        .expect("GPU backend should project a neutral renderer frame");

    assert_eq!(gpu_frame.alive_count, 2);
    assert_eq!(gpu_frame.spawned_total, 2);
    assert_eq!(gpu_frame.per_emitter_spawned, vec![2]);
    assert_eq!(gpu_frame.indirect_draw_args, [6, 2, 0, 0]);
}

#[test]
fn gpu_transparent_render_plan_uses_alive_indices_and_indirect_args() {
    let asset = ParticleSystemAsset::new("gpu-transparent")
        .with_backend(ParticleSimulationBackend::Gpu)
        .with_emitters(vec![
            ParticleEmitterAsset::sprite("gpu").with_max_particles(32)
        ]);
    let program = compile_particle_gpu_program(&asset);

    assert_eq!(
        program.shader.transparent_entries.vertex,
        "particle_gpu_transparent_vs"
    );
    assert_eq!(
        program.shader.transparent_entries.fragment,
        "particle_gpu_transparent_fs"
    );
    assert_eq!(
        program.resources.transparent_render_params_bytes,
        ParticleGpuTransparentRenderParams::ENCODED_SIZE as u64
    );
    assert!(program.passes.iter().any(|pass| {
        pass.kind == ParticleGpuPassKind::TransparentRender
            && pass.reads.contains(&"particles.gpu.alive-indices")
            && pass.reads.contains(&"particles.gpu.indirect-draw-args")
    }));

    let params = ParticleGpuTransparentRenderParams::new(Vec3::X, Vec3::Y, 0.75);
    assert_eq!(
        params.encode(),
        [
            0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 63, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0,
            0, 0, 0, 0, 0, 0
        ]
    );
}

#[test]
fn gpu_counter_readback_decodes_renderer_outputs_and_cpu_parity() {
    let readback = ParticleGpuCounterReadback::from_words(&[5, 3, 7, 0, 2, 1], 2).unwrap();

    assert_eq!(readback.alive_count, 5);
    assert_eq!(readback.spawned_total, 3);
    assert_eq!(readback.debug_flags, 7);
    assert_eq!(readback.per_emitter_spawned, vec![2, 1]);

    let outputs = readback.to_render_outputs([6, 5, 0, 0]);
    assert_eq!(outputs.alive_count, 5);
    assert_eq!(outputs.spawned_total, 3);
    assert_eq!(outputs.indirect_draw_args, [6, 5, 0, 0]);
    assert_eq!(outputs.per_emitter_spawned, vec![2, 1]);
    assert!(!outputs.is_empty());

    let parity = ParticleGpuCpuParityReport::compare_counts(5, 3, &readback);
    assert!(parity.matches());
    assert!(parity.mismatches().is_empty());

    let mismatch = ParticleGpuCpuParityReport::compare_counts(4, 3, &readback);
    assert!(!mismatch.matches());
    assert_eq!(
        mismatch.mismatches(),
        vec!["alive count CPU=4 GPU=5".to_string()]
    );
}

#[test]
fn particles_manager_records_neutral_gpu_feedback_without_affecting_cpu_snapshot() {
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(42, spawn_rate_asset(4.0, 8)))
        .unwrap();
    manager.tick(0.25).unwrap();
    let before = manager.snapshot();
    assert_eq!(before.emitters[0].live_particles, 1);
    assert!(before.last_gpu_feedback.is_none());

    let readback = RenderParticleGpuReadbackOutputs {
        alive_count: 5,
        spawned_total: 7,
        debug_flags: 3,
        per_emitter_spawned: vec![7],
        indirect_draw_args: [6, 5, 0, 0],
    };
    manager.apply_gpu_feedback(zircon_runtime::graphics::ParticleRuntimeFeedback::new(
        Some(zircon_runtime::graphics::ParticleGpuFeedback::new(
            readback.clone(),
        )),
    ));

    let after = manager.snapshot();
    assert_eq!(after.emitters[0].live_particles, 1);
    assert_eq!(after.last_gpu_feedback, Some(readback));

    manager.apply_gpu_feedback(zircon_runtime::graphics::ParticleRuntimeFeedback::default());
    assert_eq!(
        manager.snapshot().last_gpu_feedback,
        after.last_gpu_feedback
    );
}

#[test]
fn gpu_frame_planner_accumulates_spawn_requests_and_encodes_params() {
    let asset = ParticleSystemAsset::new("gpu-frame")
        .with_backend(ParticleSimulationBackend::Gpu)
        .with_seed(42)
        .with_emitters(vec![ParticleEmitterAsset::sprite("gpu")
            .with_spawn_rate(8.0)
            .with_burst(ParticleBurst::new(0.0, 3))
            .with_max_particles(16)
            .with_shape(ParticleShape::Box {
                half_extents: Vec3::ONE,
            })
            .with_initial_velocity(ParticleVec3Range::new(Vec3::ZERO, Vec3::Y))
            .with_color_over_lifetime(vec![
                ParticleColorKey::new(0.0, Vec4::ONE),
                ParticleColorKey::new(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0)),
            ])]);
    let mut planner = ParticleGpuFramePlanner::new(asset);

    let first = planner
        .build_frame(0.25, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();
    assert_eq!(first.total_spawn_count(), 5);
    assert_eq!(first.expected_frame_extract().alive_count, 5);
    assert_eq!(
        first.expected_frame_extract().indirect_draw_args,
        [6, 5, 0, 0]
    );
    assert_eq!(first.emitters[0].base_slot, 0);
    assert_eq!(first.emitters[0].capacity, 16);
    assert_eq!(
        first.emitters[0].shape,
        ParticleShape::Box {
            half_extents: Vec3::ONE
        }
    );

    let encoded = first.encode_emitters(planner.layout());
    assert_eq!(encoded.len(), ParticleGpuEmitterFrameParams::ENCODED_SIZE);
}

#[test]
fn gpu_frame_planner_does_not_consume_bursts_until_prepared_frame_commit() {
    let asset = ParticleSystemAsset::new("gpu-frame-transaction")
        .with_backend(ParticleSimulationBackend::Gpu)
        .with_emitters(vec![ParticleEmitterAsset::sprite("gpu")
            .with_spawn_rate(0.0)
            .with_burst(ParticleBurst::new(0.0, 3))
            .with_max_particles(16)]);
    let mut planner = ParticleGpuFramePlanner::new(asset);

    let abandoned = planner
        .prepare_frame(0.25, Transform::default())
        .expect("particle frame preparation should succeed");
    assert_eq!(abandoned.frame().total_spawn_count(), 3);
    drop(abandoned);

    let retry = planner
        .build_frame(0.25, Transform::default())
        .expect("abandoned preparation must leave the planner retryable");
    assert_eq!(retry.total_spawn_count(), 3);
}

#[test]
fn gpu_emitter_encoding_uses_dense_lookup_with_first_match_and_zero_fill() {
    let asset = ParticleSystemAsset::new("gpu-encode-index").with_emitters(vec![
        ParticleEmitterAsset::sprite("first").with_max_particles(8),
        ParticleEmitterAsset::sprite("missing").with_max_particles(8),
        ParticleEmitterAsset::sprite("last").with_max_particles(8),
    ]);
    let mut planner = ParticleGpuFramePlanner::new(asset);
    let planned = planner
        .build_frame(0.25, Transform::default())
        .expect("particle GPU frame should build");
    let first = planned.emitters[0].clone();
    let last = planned.emitters[2].clone();
    let mut duplicate = first.clone();
    duplicate.spawn_count = 7;
    let sparse = ParticleGpuFrameParams {
        dt: planned.dt,
        age_seconds: planned.age_seconds,
        emitters: vec![first.clone(), duplicate, last.clone()],
    };

    let encoded = sparse.encode_emitters(planner.layout());
    let mut expected = Vec::new();
    first.encode(&mut expected);
    expected.resize(
        expected.len() + ParticleGpuEmitterFrameParams::ENCODED_SIZE,
        0,
    );
    last.encode(&mut expected);
    assert_eq!(encoded, expected);

    let source = include_str!("../render/gpu/planner.rs");
    let encode = &source[source
        .find("pub fn encode_emitters")
        .expect("particle GPU frame must expose emitter encoding")..];
    assert!(encode.contains("indexed_emitters"));
    assert!(!encode.contains(".find(|params|"));
}

#[test]
fn gpu_layout_clamps_capacity_and_reports_diagnostic() {
    let asset = ParticleSystemAsset::new("huge-gpu")
        .with_backend(ParticleSimulationBackend::Gpu)
        .with_emitters(vec![
            ParticleEmitterAsset::sprite("first").with_max_particles(PARTICLE_GPU_MAX_PARTICLES),
            ParticleEmitterAsset::sprite("overflow").with_max_particles(128),
        ]);

    let program = compile_particle_gpu_program(&asset);

    assert_eq!(program.layout.capacity, PARTICLE_GPU_MAX_PARTICLES);
    assert!(program.layout.clamped);
    assert_eq!(program.layout.emitters[1].capacity, 0);
    assert!(program
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.reason == crate::ParticleGpuFallbackReason::CapacityExceeded));
}

#[test]
fn particle_gpu_runtime_owner_executes_backend_and_exposes_active_buffers() {
    let Some((device, queue)) = offscreen_wgpu_device() else {
        return;
    };
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(
            77,
            ParticleSystemAsset::new("gpu-owner")
                .with_backend(ParticleSimulationBackend::Gpu)
                .with_seed(7)
                .with_emitters(vec![ParticleEmitterAsset::sprite("gpu")
                    .with_spawn_rate(0.0)
                    .with_burst(ParticleBurst::new(0.0, 3))
                    .with_max_particles(16)]),
        ))
        .unwrap();
    manager.tick(0.001).unwrap();

    let instances = manager.gpu_runtime_instances();
    assert_eq!(instances.len(), 1);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-particle-gpu-runtime-owner-test"),
    });
    let mut owner = ParticleGpuRuntimeOwner::default();

    let mut upload_sink = QueueUploadSink(&queue);
    let frame = owner
        .execute_instances(&device, &mut upload_sink, &mut encoder, &instances)
        .unwrap()
        .expect("playing GPU particle instance should execute a backend frame");

    assert_eq!(frame.outputs.spawned_total, 3);
    assert_eq!(frame.outputs.indirect_draw_args, [6, 3, 0, 0]);
    let bindings = owner.active_bindings().unwrap();
    assert_eq!(bindings.indirect_draw_args.size(), 16);
    assert!(bindings.particles_a.size() > 0);
    assert_eq!(bindings.particles_a.size(), bindings.particles_b.size());
    assert!(!std::ptr::eq(bindings.particles_a, bindings.particles_b));
    queue.submit([encoder.finish()]);
    owner.commit_frame_transaction(frame.transaction_id());
}

#[test]
fn particle_gpu_runtime_owner_retries_abandoned_planner_and_ping_pong_state() {
    let Some((device, queue)) = offscreen_wgpu_device() else {
        return;
    };
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(
            78,
            ParticleSystemAsset::new("gpu-owner-rollback")
                .with_backend(ParticleSimulationBackend::Gpu)
                .with_emitters(vec![ParticleEmitterAsset::sprite("gpu")
                    .with_spawn_rate(0.0)
                    .with_burst(ParticleBurst::new(0.0, 5))
                    .with_max_particles(16)]),
        ))
        .unwrap();
    manager.tick(0.001).unwrap();
    let instances = manager.gpu_runtime_instances();
    let mut owner = ParticleGpuRuntimeOwner::default();

    let abandoned = {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-particle-gpu-runtime-owner-abandoned-test"),
        });
        let mut upload_sink = QueueUploadSink(&queue);
        owner
            .execute_instances(&device, &mut upload_sink, &mut encoder, &instances)
            .unwrap()
            .expect("first particle frame should prepare")
    };
    assert_eq!(abandoned.outputs.spawned_total, 5);
    owner.rollback_frame_transaction(abandoned.transaction_id());

    let mut retry_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-particle-gpu-runtime-owner-retry-test"),
    });
    let mut retry_upload_sink = QueueUploadSink(&queue);
    let retry = owner
        .execute_instances(
            &device,
            &mut retry_upload_sink,
            &mut retry_encoder,
            &instances,
        )
        .unwrap()
        .expect("rolled-back particle frame should remain retryable");

    assert_eq!(retry.outputs.spawned_total, 5);
    queue.submit([retry_encoder.finish()]);
    owner.commit_frame_transaction(retry.transaction_id());
}

#[test]
fn particle_gpu_runtime_owner_aggregates_playing_gpu_instances() {
    let Some((device, queue)) = offscreen_wgpu_device() else {
        return;
    };
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(
            81,
            ParticleSystemAsset::new("gpu-owner-a")
                .with_backend(ParticleSimulationBackend::Gpu)
                .with_seed(11)
                .with_emitters(vec![ParticleEmitterAsset::sprite("gpu-a")
                    .with_spawn_rate(0.0)
                    .with_burst(ParticleBurst::new(0.0, 2))
                    .with_max_particles(16)]),
        ))
        .unwrap();
    manager
        .instantiate(ParticleSystemComponent::new(
            82,
            ParticleSystemAsset::new("gpu-owner-b")
                .with_backend(ParticleSimulationBackend::Gpu)
                .with_seed(13)
                .with_emitters(vec![ParticleEmitterAsset::sprite("gpu-b")
                    .with_spawn_rate(0.0)
                    .with_burst(ParticleBurst::new(0.0, 4))
                    .with_max_particles(32)]),
        ))
        .unwrap();
    manager.tick(0.001).unwrap();

    let instances = manager.gpu_runtime_instances();
    assert_eq!(instances.len(), 2);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-particle-gpu-runtime-owner-aggregate-test"),
    });
    let mut owner = ParticleGpuRuntimeOwner::default();

    let mut upload_sink = QueueUploadSink(&queue);
    let frame = owner
        .execute_instances(&device, &mut upload_sink, &mut encoder, &instances)
        .unwrap()
        .expect("playing GPU particle instances should execute an aggregate backend frame");

    assert_eq!(frame.outputs.spawned_total, 6);
    assert_eq!(frame.outputs.per_emitter_spawned, vec![2, 4]);
    assert_eq!(frame.outputs.indirect_draw_args, [6, 6, 0, 0]);
    let bindings = owner.active_bindings().unwrap();
    assert!(bindings.particles_a.size() > 0);
    assert_eq!(bindings.particles_a.size(), bindings.particles_b.size());
    assert!(!std::ptr::eq(bindings.particles_a, bindings.particles_b));
    queue.submit([encoder.finish()]);
    owner.commit_frame_transaction(frame.transaction_id());
}

#[test]
fn render_particle_cpu_gpu_parity_small_scene_matches_counts_and_indirect_args() {
    let Some((device, queue)) = offscreen_wgpu_device() else {
        return;
    };
    let asset = ParticleSystemAsset::new("gpu-parity-small-scene")
        .with_backend(ParticleSimulationBackend::Gpu)
        .with_seed(23)
        .with_emitters(vec![ParticleEmitterAsset::sprite("gpu-parity")
            .with_spawn_rate(0.0)
            .with_burst(ParticleBurst::new(0.0, 5))
            .with_max_particles(16)
            .with_lifetime(crate::ParticleScalarRange::constant(4.0))
            .with_initial_velocity(ParticleVec3Range::new(Vec3::ZERO, Vec3::ZERO))
            .with_gravity(Vec3::ZERO)
            .with_drag(0.0)]);
    let frame_dt = 0.001;

    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(93, asset.clone()))
        .unwrap();
    manager.tick(frame_dt).unwrap();
    let cpu_extract = manager.build_extract(None);
    let cpu_live_particles = cpu_extract.sprites.len() as u32;
    let cpu_spawned_particles = cpu_extract
        .gpu_frame
        .as_ref()
        .expect("GPU backend extract should expose neutral spawn counts")
        .spawned_total;

    let mut planner = ParticleGpuFramePlanner::new(asset.clone());
    let frame = planner
        .build_frame(frame_dt, Transform::default())
        .expect("deterministic GPU parity frame should build");
    let mut backend = ParticleGpuBackend::new(&device, &asset).unwrap();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-particle-gpu-cpu-parity-small-scene-test"),
    });
    let mut upload_sink = QueueUploadSink(&queue);
    let backend_commit = backend
        .execute_frame(
            &mut upload_sink,
            &mut encoder,
            &frame,
            ParticleGpuReadbackRequest::Counters,
        )
        .unwrap();
    queue.submit([encoder.finish()]);
    assert!(backend.commit_prepared_frame(backend_commit));

    let gpu_counters = backend.read_counter_readback(&device).unwrap();
    let parity = ParticleGpuCpuParityReport::compare_counts(
        cpu_live_particles,
        cpu_spawned_particles,
        &gpu_counters,
    );
    assert!(
        parity.matches(),
        "CPU/GPU particle count parity failed: {:?}",
        parity.mismatches()
    );

    let gpu_outputs = backend.read_render_outputs_readback(&device).unwrap();
    assert_eq!(
        gpu_outputs.indirect_draw_args,
        [6, cpu_live_particles, 0, 0]
    );
    assert_eq!(gpu_outputs.per_emitter_spawned, vec![cpu_spawned_particles]);
}

#[test]
fn particle_gpu_runtime_owner_records_transparent_draw_from_executed_backend() {
    let Some((device, queue)) = offscreen_wgpu_device() else {
        return;
    };
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(
            91,
            ParticleSystemAsset::new("gpu-transparent-owner")
                .with_backend(ParticleSimulationBackend::Gpu)
                .with_seed(17)
                .with_emitters(vec![ParticleEmitterAsset::sprite("gpu-transparent")
                    .with_spawn_rate(0.0)
                    .with_burst(ParticleBurst::new(0.0, 2))
                    .with_max_particles(16)]),
        ))
        .unwrap();
    manager.tick(0.001).unwrap();

    let instances = manager.gpu_runtime_instances();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-particle-gpu-runtime-owner-transparent-test"),
    });
    let mut owner = ParticleGpuRuntimeOwner::default();
    let mut runtime_upload_sink = QueueUploadSink(&queue);
    let frame = owner
        .execute_instances(&device, &mut runtime_upload_sink, &mut encoder, &instances)
        .unwrap()
        .expect("playing GPU particle instance should execute before transparent draw");

    let (scene_layout, _scene_uniform, scene_bind_group) =
        create_test_scene_bind_group(&device, &queue);
    let (color_texture, color_view) = create_test_texture_view(
        &device,
        "zircon-particle-gpu-transparent-owner-color",
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    let (_depth_texture, depth_view) = create_test_texture_view(
        &device,
        "zircon-particle-gpu-transparent-owner-depth",
        wgpu::TextureFormat::Depth32Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    );
    clear_test_render_targets(&mut encoder, &color_view, &depth_view);

    let recorded = {
        let mut upload_sink = QueueUploadSink(&queue);
        owner
            .record_transparent_render(
                &device,
                &scene_layout,
                ParticleGpuTransparentRenderConfig::new(
                    wgpu::TextureFormat::Rgba8Unorm,
                    wgpu::TextureFormat::Depth32Float,
                ),
                &mut upload_sink,
                &mut encoder,
                &color_view,
                &depth_view,
                &scene_bind_group,
                ParticleGpuTransparentRenderParams::new(Vec3::X, Vec3::Y, 1.0),
                zircon_runtime::graphics::ViewportRenderRegion::full_target(UVec2::new(32, 32)),
            )
            .unwrap()
    };

    assert!(recorded);
    queue.submit([encoder.finish()]);
    owner.commit_frame_transaction(frame.transaction_id());

    let color_pixels = read_test_texture_rgba8(&device, &queue, &color_texture, 32, 32);
    let visible_pixel_count = color_pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[3] > 0 && pixel[..3].iter().any(|channel| *channel > 0))
        .count();
    assert!(
        visible_pixel_count > 0,
        "transparent GPU draw should write visible RGBA pixels into the color target"
    );
}

#[test]
fn particle_gpu_runtime_owner_skips_transparent_draw_without_executed_backend() {
    let Some((device, queue)) = offscreen_wgpu_device() else {
        return;
    };
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-particle-gpu-runtime-owner-empty-transparent-test"),
    });
    let (scene_layout, _scene_uniform, scene_bind_group) =
        create_test_scene_bind_group(&device, &queue);
    let (_color_texture, color_view) = create_test_texture_view(
        &device,
        "zircon-particle-gpu-transparent-owner-empty-color",
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    let (_depth_texture, depth_view) = create_test_texture_view(
        &device,
        "zircon-particle-gpu-transparent-owner-empty-depth",
        wgpu::TextureFormat::Depth32Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    );
    clear_test_render_targets(&mut encoder, &color_view, &depth_view);

    let mut owner = ParticleGpuRuntimeOwner::default();
    let recorded = {
        let mut upload_sink = QueueUploadSink(&queue);
        owner
            .record_transparent_render(
                &device,
                &scene_layout,
                ParticleGpuTransparentRenderConfig::new(
                    wgpu::TextureFormat::Rgba8Unorm,
                    wgpu::TextureFormat::Depth32Float,
                ),
                &mut upload_sink,
                &mut encoder,
                &color_view,
                &depth_view,
                &scene_bind_group,
                ParticleGpuTransparentRenderParams::new(Vec3::X, Vec3::Y, 1.0),
                zircon_runtime::graphics::ViewportRenderRegion::full_target(UVec2::new(32, 32)),
            )
            .unwrap()
    };

    assert!(!recorded);
    queue.submit([encoder.finish()]);
}

fn offscreen_wgpu_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = block_on_test_future(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    block_on_test_future(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-particle-gpu-runtime-owner-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}

fn create_test_scene_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (wgpu::BindGroupLayout, wgpu::Buffer, wgpu::BindGroup) {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-particle-gpu-transparent-test-scene-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX
                | wgpu::ShaderStages::FRAGMENT
                | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-particle-gpu-transparent-test-scene-uniform"),
        size: 256,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut bytes = [0u8; 256];
    for (index, value) in [
        1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
    .iter()
    .enumerate()
    {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    queue.write_buffer(&uniform, 0, &bytes);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-particle-gpu-transparent-test-scene-bind-group"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }],
    });
    (layout, uniform, bind_group)
}

fn create_test_texture_view(
    device: &wgpu::Device,
    label: &'static str,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: 32,
            height: 32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn clear_test_render_targets(
    encoder: &mut wgpu::CommandEncoder,
    color_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("zircon-particle-gpu-transparent-test-clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color_view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
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
}

fn read_test_texture_rgba8(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let bytes_per_pixel = 4_u32;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer_size = padded_bytes_per_row as u64 * height as u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-particle-gpu-transparent-readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-particle-gpu-transparent-readback-encoder"),
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("transparent draw readback poll should succeed");
    receiver
        .recv()
        .expect("transparent draw readback should report map status")
        .expect("transparent draw readback buffer should map");

    let mapped = slice.get_mapped_range();
    let mut rgba = vec![0_u8; (width * height * bytes_per_pixel) as usize];
    for row in 0..height as usize {
        let source_offset = row * padded_bytes_per_row as usize;
        let target_offset = row * unpadded_bytes_per_row as usize;
        rgba[target_offset..target_offset + unpadded_bytes_per_row as usize].copy_from_slice(
            &mapped[source_offset..source_offset + unpadded_bytes_per_row as usize],
        );
    }
    drop(mapped);
    buffer.unmap();

    rgba
}

fn block_on_test_future<T>(future: impl Future<Output = T>) -> T {
    let waker = noop_waker();
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);
    loop {
        match Pin::new(&mut future).poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn noop_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        noop_raw_waker()
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}

    fn noop_raw_waker() -> RawWaker {
        RawWaker::new(
            std::ptr::null(),
            &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
        )
    }

    unsafe { Waker::from_raw(noop_raw_waker()) }
}
