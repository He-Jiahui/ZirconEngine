use crate::{
    ParticleAnimationBinding, ParticleBurst, ParticleEmitterAsset, ParticleScalarRange,
    ParticleSimulationError, ParticleSystemAsset, ParticleSystemComponent, ParticlesManager,
};

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
