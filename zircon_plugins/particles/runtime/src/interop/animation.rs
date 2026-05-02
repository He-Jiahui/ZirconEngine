use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::Real;

use crate::ParticleEmitterHandle;

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleAnimationBinding {
    pub parameter: String,
    pub curve_path: String,
    pub normalized_progress: Real,
}

impl ParticleAnimationBinding {
    pub fn new(
        parameter: impl Into<String>,
        curve_path: impl Into<String>,
        normalized_progress: Real,
    ) -> Self {
        Self {
            parameter: parameter.into(),
            curve_path: curve_path.into(),
            normalized_progress: normalized_progress.clamp(0.0, 1.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParticleAnimationEventKind {
    SpawnOnce,
    TimedEmissionBegin,
    TimedEmissionEnd,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleAnimationEvent {
    pub entity: EntityId,
    pub handle: Option<ParticleEmitterHandle>,
    pub kind: ParticleAnimationEventKind,
    pub bindings: Vec<ParticleAnimationBinding>,
}

impl ParticleAnimationEvent {
    pub fn spawn_once(entity: EntityId) -> Self {
        Self {
            entity,
            handle: None,
            kind: ParticleAnimationEventKind::SpawnOnce,
            bindings: Vec::new(),
        }
    }

    pub fn with_binding(mut self, binding: ParticleAnimationBinding) -> Self {
        self.bindings.push(binding);
        self
    }
}
