use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ResourceHandle, ResourceId, TextureMarker};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::plugin::RuntimePluginRegistrationReport;
use zircon_runtime::render_graph::QueueLane;

use super::*;

#[test]
fn particles_plugin_registration_contributes_runtime_module_render_feature_and_component() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == PARTICLES_MODULE_NAME));
    assert!(report
        .extensions
        .render_features()
        .iter()
        .any(|feature| feature.name == PARTICLES_FEATURE_NAME));
    assert!(report
        .extensions
        .components()
        .iter()
        .any(|descriptor| descriptor.type_id == PARTICLE_SYSTEM_COMPONENT_TYPE));
    assert!(report
        .extensions
        .plugin_options()
        .iter()
        .any(|option| option.key == "particles.backend"));
    let executor_ids = report
        .extensions
        .render_pass_executors()
        .iter()
        .map(|registration| registration.executor_id().as_str())
        .collect::<Vec<_>>();
    assert!(executor_ids.contains(&"particle.gpu.spawn-update"));
    assert!(executor_ids.contains(&"particle.gpu.compact-alive"));
    assert!(executor_ids.contains(&"particle.gpu.indirect-args"));
    assert!(executor_ids.contains(&"particle.transparent"));
    assert!(report
        .package_manifest
        .event_catalogs
        .iter()
        .any(|catalog| catalog.namespace == PARTICLES_DYNAMIC_EVENT_NAMESPACE));

    let descriptor = render_feature_descriptor();
    let pass_names = descriptor
        .stage_passes
        .iter()
        .map(|pass| pass.pass_name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        pass_names,
        vec![
            "particle-gpu-spawn-update",
            "particle-gpu-compact-alive",
            "particle-gpu-build-indirect-args",
            "particle-render"
        ]
    );
    assert_eq!(descriptor.stage_passes[0].queue, QueueLane::AsyncCompute);
    assert_eq!(descriptor.stage_passes[3].queue, QueueLane::Graphics);
}

#[test]
fn particles_module_resolves_manager_and_ticks_cpu_spawn_rate() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(PARTICLES_MODULE_NAME).unwrap();
    let manager = runtime
        .handle()
        .resolve_manager::<ParticlesManager>(PARTICLES_MANAGER_NAME)
        .unwrap();

    let handle = manager
        .instantiate(ParticleSystemComponent::new(7, spawn_rate_asset(4.0, 8)))
        .unwrap();
    manager.tick(0.25).unwrap();

    let snapshot = manager.snapshot();
    assert_eq!(snapshot.emitters[0].handle, handle);
    assert_eq!(snapshot.emitters[0].entity, 7);
    assert_eq!(snapshot.emitters[0].live_particles, 1);
    assert_eq!(snapshot.sprites.len(), 1);
}

#[test]
fn cpu_particles_are_deterministic_for_matching_seed_and_ticks() {
    let asset = ParticleSystemAsset::new("deterministic")
        .with_seed(99)
        .with_emitters(vec![ParticleEmitterAsset::sprite("sparks")
            .with_spawn_rate(6.0)
            .with_shape(ParticleShape::Sphere { radius: 1.0 })
            .with_initial_velocity(ParticleVec3Range::new(
                Vec3::new(-1.0, 0.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
            ))]);
    let first = ParticlesManager::default();
    let second = ParticlesManager::default();
    first
        .instantiate(ParticleSystemComponent::new(1, asset.clone()))
        .unwrap();
    second
        .instantiate(ParticleSystemComponent::new(1, asset))
        .unwrap();

    for _ in 0..4 {
        first.tick(1.0 / 6.0).unwrap();
        second.tick(1.0 / 6.0).unwrap();
    }

    assert_eq!(first.snapshot().sprites, second.snapshot().sprites);
}

#[test]
fn cpu_particles_apply_lifetime_death_and_reuse_allocated_slots() {
    let asset =
        ParticleSystemAsset::new("reuse").with_emitters(vec![ParticleEmitterAsset::sprite(
            "short",
        )
        .with_spawn_rate(20.0)
        .with_max_particles(1)
        .with_lifetime(ParticleScalarRange::constant(0.06))]);
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(2, asset))
        .unwrap();

    manager.tick(0.05).unwrap();
    assert_eq!(manager.snapshot().emitters[0].live_particles, 1);
    assert_eq!(manager.snapshot().emitters[0].allocated_particles, 1);
    manager.tick(0.02).unwrap();
    assert_eq!(manager.snapshot().emitters[0].live_particles, 0);
    manager.tick(0.05).unwrap();

    let state = &manager.snapshot().emitters[0];
    assert_eq!(state.live_particles, 1);
    assert_eq!(state.allocated_particles, 1);
}

#[test]
fn pause_stop_and_preview_rewind_control_cpu_state() {
    let manager = ParticlesManager::default();
    let handle = manager
        .instantiate(ParticleSystemComponent::new(3, spawn_rate_asset(60.0, 256)))
        .unwrap();
    manager.pause(handle).unwrap();
    manager.tick(1.0).unwrap();
    assert!(manager.snapshot().sprites.is_empty());

    manager.play(handle).unwrap();
    manager.rewind_preview(handle, 1.0 / 60.0, 0.5).unwrap();
    assert_eq!(manager.snapshot().emitters[0].live_particles, 30);

    manager.stop(handle).unwrap();
    let snapshot = manager.snapshot();
    assert!(!snapshot.emitters[0].playing);
    assert!(snapshot.sprites.is_empty());
}

#[test]
fn extract_sorts_sprites_back_to_front_when_camera_is_known() {
    let manager = ParticlesManager::default();
    let asset =
        ParticleSystemAsset::new("burst").with_emitters(vec![ParticleEmitterAsset::sprite(
            "burst",
        )
        .with_spawn_rate(0.0)
        .with_burst(ParticleBurst::new(0.0, 2))
        .with_shape(ParticleShape::Box {
            half_extents: Vec3::new(0.0, 0.0, 2.0),
        })]);
    manager
        .instantiate(
            ParticleSystemComponent::new(9, asset)
                .with_transform(Transform::from_translation(Vec3::new(0.0, 0.0, 5.0))),
        )
        .unwrap();
    manager.tick(0.001).unwrap();

    let extract = manager.build_extract(Some(Vec3::ZERO));

    assert_eq!(extract.emitters, vec![9]);
    assert_eq!(extract.sprites.len(), 2);
    let first_distance = extract.sprites[0].position.length_squared();
    let second_distance = extract.sprites[1].position.length_squared();
    assert!(first_distance >= second_distance);
}

#[test]
fn cpu_extract_preserves_material_texture_rotation_bounds_and_sort_metadata() {
    let material = ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
        "particles/material/spark",
    ));
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "particles/texture/spark",
    ));
    let mut emitter = ParticleEmitterAsset::sprite("metadata")
        .with_spawn_rate(0.0)
        .with_burst(ParticleBurst::new(0.0, 1))
        .with_initial_rotation(ParticleScalarRange::constant(0.5))
        .with_initial_angular_velocity(ParticleScalarRange::constant(2.0))
        .with_material(material)
        .with_texture(texture);
    emitter.initial_size = ParticleScalarRange::constant(2.0);
    let asset = ParticleSystemAsset::new("metadata").with_emitters(vec![emitter]);
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(33, asset))
        .unwrap();

    manager.tick(0.25).unwrap();
    let snapshot = manager.snapshot();
    let sprite = snapshot.sprites.first().expect("one sprite should spawn");

    assert_eq!(sprite.material, Some(material));
    assert_eq!(sprite.texture, Some(texture));
    assert_approx_eq(sprite.rotation, 1.0);

    let extract = manager.build_extract(Some(Vec3::new(0.0, 0.0, -8.0)));
    assert_eq!(
        extract.sort_camera_position,
        Some(Vec3::new(0.0, 0.0, -8.0))
    );
    assert_eq!(extract.bounds.len(), 1);
    assert_eq!(extract.bounds[0].entity, 33);
    assert_eq!(extract.bounds[0].center, sprite.position);
    assert_approx_eq(extract.bounds[0].radius, 3.0_f32.sqrt());
    assert_eq!(extract.sprites[0].material, Some(material));
    assert_eq!(extract.sprites[0].texture, Some(texture));
}

#[test]
fn physics_modules_noop_without_capability_and_apply_external_force_when_enabled() {
    let asset =
        ParticleSystemAsset::new("physics").with_emitters(vec![ParticleEmitterAsset::sprite(
            "force",
        )
        .with_spawn_rate(0.0)
        .with_lifetime(ParticleScalarRange::constant(2.0))
        .with_burst(ParticleBurst::new(0.0, 1))
        .with_physics(ParticlePhysicsOptions::disabled().with_external_force(Vec3::Y))]);

    let missing = ParticlesManager::default();
    missing
        .instantiate(ParticleSystemComponent::new(41, asset.clone()))
        .unwrap();
    missing.tick(1.0).unwrap();
    let missing_snapshot = missing.snapshot();
    assert_approx_eq(missing_snapshot.sprites[0].position.y, 0.0);
    assert!(missing_snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("particle physics modules are running as no-op")
    }));

    let enabled =
        ParticlesManager::with_capabilities(&[crate::service::PARTICLES_PHYSICS_CAPABILITY]);
    enabled
        .instantiate(ParticleSystemComponent::new(42, asset))
        .unwrap();
    enabled.tick(1.0).unwrap();
    let enabled_snapshot = enabled.snapshot();
    assert_approx_eq(enabled_snapshot.sprites[0].position.y, 1.0);
    assert!(!enabled_snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("particle physics modules are running as no-op")
    }));
}

#[test]
fn enabling_physics_capability_after_instantiate_updates_existing_instances() {
    let asset =
        ParticleSystemAsset::new("late-physics").with_emitters(vec![ParticleEmitterAsset::sprite(
            "force",
        )
        .with_spawn_rate(0.0)
        .with_lifetime(ParticleScalarRange::constant(3.0))
        .with_burst(ParticleBurst::new(0.0, 1))
        .with_physics(ParticlePhysicsOptions::disabled().with_external_force(Vec3::Y))]);
    let manager = ParticlesManager::default();
    manager
        .instantiate(ParticleSystemComponent::new(43, asset))
        .unwrap();

    manager.tick(1.0).unwrap();
    assert_approx_eq(manager.snapshot().sprites[0].position.y, 0.0);

    manager.enable_capability(crate::service::PARTICLES_PHYSICS_CAPABILITY);
    manager.tick(1.0).unwrap();

    assert_approx_eq(manager.snapshot().sprites[0].position.y, 1.0);
}

#[test]
fn animation_events_are_diagnostic_noops_without_capability_and_control_emission_when_enabled() {
    let asset =
        ParticleSystemAsset::new("animation").with_emitters(vec![ParticleEmitterAsset::sprite(
            "anim",
        )
        .with_spawn_rate(4.0)
        .with_lifetime(ParticleScalarRange::constant(4.0))
        .with_max_particles(4)
        .with_animation_binding(ParticleAnimationBinding::new(
            "emission.rate",
            "Run/Speed",
            0.5,
        ))]);
    let missing = ParticlesManager::default();
    let missing_handle = missing
        .instantiate(ParticleSystemComponent::new(51, asset.clone()))
        .unwrap();

    missing
        .apply_animation_event(ParticleAnimationEvent::spawn_once(51).with_handle(missing_handle))
        .unwrap();
    let missing_snapshot = missing.snapshot();
    assert!(missing_snapshot.sprites.is_empty());
    assert!(missing_snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("particle animation bindings are disabled")
    }));
    assert!(missing_snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("animation-controlled particle event")
    }));

    let enabled =
        ParticlesManager::with_capabilities(&[crate::service::PARTICLES_ANIMATION_CAPABILITY]);
    let enabled_handle = enabled
        .instantiate(ParticleSystemComponent::new(52, asset).with_playing(false))
        .unwrap();
    enabled
        .apply_animation_event(ParticleAnimationEvent::spawn_once(52).with_handle(enabled_handle))
        .unwrap();
    assert_eq!(enabled.snapshot().emitters[0].live_particles, 1);

    enabled
        .apply_animation_event(
            ParticleAnimationEvent::timed_emission_begin(52).with_handle(enabled_handle),
        )
        .unwrap();
    enabled.tick(0.25).unwrap();
    assert_eq!(enabled.snapshot().emitters[0].live_particles, 2);

    enabled
        .apply_animation_event(
            ParticleAnimationEvent::timed_emission_end(52).with_handle(enabled_handle),
        )
        .unwrap();
    enabled.tick(1.0).unwrap();
    assert_eq!(enabled.snapshot().emitters[0].live_particles, 2);
}

#[test]
fn instantiate_rejects_non_finite_emitter_settings() {
    let mut emitter = ParticleEmitterAsset::sprite("invalid");
    emitter.spawn_rate_per_second = f32::NAN;
    let asset = ParticleSystemAsset::new("invalid").with_emitters(vec![emitter]);
    let manager = ParticlesManager::default();

    let error = manager
        .instantiate(ParticleSystemComponent::new(61, asset))
        .unwrap_err();

    assert!(
        matches!(error, ParticleSimulationError::InvalidAsset(message) if message.contains("non-finite scalar"))
    );
}

#[test]
fn instantiate_rejects_non_finite_bursts_and_animation_bindings() {
    let burst_asset = ParticleSystemAsset::new("invalid-burst")
        .with_emitters(vec![ParticleEmitterAsset::sprite("invalid-burst")
            .with_burst(ParticleBurst::new(f32::NAN, 1))]);
    let manager = ParticlesManager::default();
    let burst_error = manager
        .instantiate(ParticleSystemComponent::new(62, burst_asset))
        .unwrap_err();
    assert!(
        matches!(burst_error, ParticleSimulationError::InvalidAsset(message) if message.contains("non-finite burst"))
    );

    let mut binding = ParticleAnimationBinding::new("emission.rate", "Run/Speed", 0.5);
    binding.normalized_progress = f32::NAN;
    let binding_asset = ParticleSystemAsset::new("invalid-binding").with_emitters(vec![
        ParticleEmitterAsset::sprite("invalid-binding").with_animation_binding(binding),
    ]);
    let binding_error = manager
        .instantiate(ParticleSystemComponent::new(63, binding_asset))
        .unwrap_err();
    assert!(
        matches!(binding_error, ParticleSimulationError::InvalidAsset(message) if message.contains("non-finite animation binding"))
    );
}

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
        .any(|diagnostic| diagnostic.reason == ParticleGpuFallbackReason::CapacityExceeded));
}

#[test]
fn optional_physics_and_animation_helpers_report_missing_capabilities() {
    let status = ParticleOptionalFeatureStatus::from_capabilities(
        crate::service::PARTICLES_PHYSICS_CAPABILITY,
        &["runtime.plugin.particles"],
    );
    assert!(!status.is_available());

    let binding = ParticleAnimationBinding::new("emission.rate", "Run/Speed", 1.4);
    assert_eq!(binding.normalized_progress, 1.0);
    let event = ParticleAnimationEvent::spawn_once(12).with_binding(binding);
    assert_eq!(event.kind, ParticleAnimationEventKind::SpawnOnce);
}

fn assert_approx_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 1.0e-4,
        "expected {actual} to be approximately {expected}"
    );
}

fn spawn_rate_asset(spawn_rate: f32, max_particles: u32) -> ParticleSystemAsset {
    ParticleSystemAsset::new("spawn_rate").with_emitters(vec![ParticleEmitterAsset::sprite(
        "sparks",
    )
    .with_spawn_rate(spawn_rate)
    .with_max_particles(max_particles)
    .with_lifetime(ParticleScalarRange::constant(2.0))
    .with_size_over_lifetime(vec![
        ParticleScalarKey::new(0.0, 1.0),
        ParticleScalarKey::new(1.0, 0.5),
    ])
    .with_color_over_lifetime(vec![
        ParticleColorKey::new(0.0, Vec4::new(1.0, 0.5, 0.1, 1.0)),
        ParticleColorKey::new(1.0, Vec4::new(1.0, 0.1, 0.0, 0.0)),
    ])])
}
