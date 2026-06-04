use zircon_runtime::core::math::Vec3;

use crate::{
    ParticleAnimationBinding, ParticleAnimationEvent, ParticleAnimationEventKind, ParticleBurst,
    ParticleEmitterAsset, ParticleOptionalFeatureStatus, ParticlePhysicsOptions,
    ParticleScalarRange, ParticleSystemAsset, ParticleSystemComponent, ParticlesManager,
};

use super::support::assert_approx_eq;

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
