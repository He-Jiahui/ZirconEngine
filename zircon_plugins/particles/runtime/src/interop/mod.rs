#[path = "animation.rs"]
mod animation;
#[path = "physics.rs"]
mod physics;

pub use animation::{ParticleAnimationBinding, ParticleAnimationEvent, ParticleAnimationEventKind};
pub use physics::{ParticleOptionalFeatureStatus, ParticlePhysicsOptions};
