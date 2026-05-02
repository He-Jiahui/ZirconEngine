mod cpu;
mod pool;
mod rng;

pub(crate) use cpu::ParticleSystemInstance;
pub(crate) use rng::ParticleRng;

use std::fmt;

use zircon_runtime::core::framework::render::RenderParticleSpriteSnapshot;

pub type ParticleSpriteSnapshot = RenderParticleSpriteSnapshot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParticleSimulationError {
    InvalidDeltaTime,
    InvalidAsset(String),
    UnknownHandle(u64),
}

impl fmt::Display for ParticleSimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDeltaTime => {
                write!(f, "particle delta time must be finite and non-negative")
            }
            Self::InvalidAsset(message) => write!(f, "invalid particle asset: {message}"),
            Self::UnknownHandle(handle) => write!(f, "unknown particle emitter handle {handle}"),
        }
    }
}

impl std::error::Error for ParticleSimulationError {}
