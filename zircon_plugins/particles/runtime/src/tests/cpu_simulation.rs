use zircon_runtime::core::math::Vec3;

use crate::{
    ParticleBurst, ParticleEmitterAsset, ParticleScalarRange, ParticleShape, ParticleSystemAsset,
    ParticleSystemComponent, ParticleVec3Range, ParticlesManager,
};

use super::support::spawn_rate_asset;

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
