use serde::Deserialize;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::World;

use crate::{
    ParticleSimulationBackend, ParticleSimulationError, ParticleSystemAsset,
    ParticleSystemComponent, PARTICLE_SYSTEM_COMPONENT_TYPE,
};

#[derive(Deserialize)]
struct SceneComponent {
    // Only inline assets are admitted here. Asset URI loading needs an actual
    // particle asset loader; a missing asset must not spawn a default effect.
    asset: ParticleSystemAsset,
    #[serde(default = "default_playing")]
    playing: bool,
    #[serde(default = "default_time_scale")]
    time_scale: Real,
    backend: Option<ParticleSimulationBackend>,
    seed: Option<u64>,
}

fn default_playing() -> bool {
    true
}

fn default_time_scale() -> Real {
    1.0
}

pub(super) fn enabled(world: &World, entity: EntityId) -> bool {
    world
        .dynamic_component(entity, PARTICLE_SYSTEM_COMPONENT_TYPE)
        .is_some_and(|value| {
            value.get("playing").and_then(|value| value.as_bool()) != Some(false)
                && world.active_in_hierarchy(entity) == Some(true)
        })
}

pub(super) fn read(
    world: &World,
    entity: EntityId,
) -> Result<ParticleSystemComponent, ParticleSimulationError> {
    let invalid = |message: String| ParticleSimulationError::InvalidAsset(message);
    let value = world
        .dynamic_component(entity, PARTICLE_SYSTEM_COMPONENT_TYPE)
        .expect("enabled scene emitter must have a component");
    let mut source =
        SceneComponent::deserialize(value).map_err(|error| invalid(error.to_string()))?;
    if let Some(backend) = source.backend {
        source.asset.backend = backend;
    }
    if let Some(seed) = source.seed {
        source.asset.seed = seed;
    }
    let transform = world
        .world_transform(entity)
        .ok_or_else(|| invalid("emitter has no world transform".to_string()))?;
    Ok(ParticleSystemComponent::new(entity, source.asset)
        .with_transform(transform)
        .with_playing(source.playing)
        .with_time_scale(source.time_scale))
}
