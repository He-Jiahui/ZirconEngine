use zircon_runtime::core::framework::render::RenderParticleGpuReadbackOutputs;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};

use crate::{
    compile_particle_gpu_layout, compile_particle_gpu_program, ParticleBurst, ParticleColorKey,
    ParticleEmitterAsset, ParticleGpuCounterReadback, ParticleGpuCpuParityReport,
    ParticleGpuEmitterFrameParams, ParticleGpuFramePlanner, ParticleGpuPassKind,
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
