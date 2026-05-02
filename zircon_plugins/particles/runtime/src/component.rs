use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{Real, Transform};
use zircon_runtime::plugin::ComponentTypeDescriptor;

use crate::{ParticleSimulationBackend, ParticleSystemAsset, PLUGIN_ID};

pub const PARTICLE_SYSTEM_COMPONENT_TYPE: &str = "particles.Component.ParticleSystem";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParticleEmitterHandle(u64);

impl ParticleEmitterHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleSystemComponent {
    pub entity: EntityId,
    pub asset: ParticleSystemAsset,
    pub transform: Transform,
    pub playing: bool,
    pub time_scale: Real,
}

impl ParticleSystemComponent {
    pub fn new(entity: EntityId, asset: ParticleSystemAsset) -> Self {
        Self {
            entity,
            asset,
            transform: Transform::identity(),
            playing: true,
            time_scale: 1.0,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }

    pub fn with_time_scale(mut self, time_scale: Real) -> Self {
        self.time_scale = time_scale.max(0.0);
        self
    }

    pub fn backend(&self) -> ParticleSimulationBackend {
        self.asset.backend
    }
}

pub fn particle_component_descriptors() -> Vec<ComponentTypeDescriptor> {
    vec![
        ComponentTypeDescriptor::new(PARTICLE_SYSTEM_COMPONENT_TYPE, PLUGIN_ID, "Particle System")
            .with_property("asset", "particle_system_asset", true)
            .with_property("playing", "bool", true)
            .with_property("backend", "particle_simulation_backend", true)
            .with_property("time_scale", "scalar", true)
            .with_property("seed", "u64", true),
    ]
}
