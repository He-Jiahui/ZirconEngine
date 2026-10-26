use std::fmt;

use zircon_runtime::core::math::Real;

use crate::simulation::{ParticleSimulationError, ParticleSystemInstance};
use crate::{ParticleEmitterHandle, ParticleSimulationBackend};

use super::ParticlesManager;

/// The selected instance's simulation state, independent of rendered-frame readiness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticlePreviewClock {
    pub handle: ParticleEmitterHandle,
    pub age_seconds: Real,
    pub playing: bool,
    pub time_scale: Real,
    pub backend: ParticleSimulationBackend,
    pub fallback_to_cpu: bool,
}

impl ParticlePreviewClock {
    fn from_instance(instance: &ParticleSystemInstance) -> Self {
        Self {
            handle: instance.handle,
            age_seconds: instance.age_seconds,
            playing: instance.playing,
            time_scale: instance.component().time_scale,
            backend: instance.backend(),
            fallback_to_cpu: instance.fallback_to_cpu,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParticlePreviewError {
    UnknownHandle(u64),
    InvalidDeltaTime,
    InvalidScaledDeltaTime,
    AgeNotAdvancing,
    GpuStepUnavailable,
    Simulation(ParticleSimulationError),
}

impl fmt::Display for ParticlePreviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownHandle(handle) => write!(f, "unknown particle preview handle {handle}"),
            Self::InvalidDeltaTime => {
                write!(
                    f,
                    "particle preview delta must be finite and greater than epsilon"
                )
            }
            Self::InvalidScaledDeltaTime => {
                write!(
                    f,
                    "particle preview scaled delta must be finite and positive"
                )
            }
            Self::AgeNotAdvancing => {
                write!(
                    f,
                    "particle preview step must advance the finite instance age"
                )
            }
            Self::GpuStepUnavailable => {
                write!(f, "GPU particle preview stepping requires a GPU executor")
            }
            Self::Simulation(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl std::error::Error for ParticlePreviewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Simulation(error) => Some(error),
            _ => None,
        }
    }
}

impl ParticlesManager {
    /// Reads the current instance under the manager lock without invalidating snapshots.
    pub fn preview_clock(
        &self,
        handle: ParticleEmitterHandle,
    ) -> Result<ParticlePreviewClock, ParticlePreviewError> {
        let state = self.lock_state();
        let instance = state
            .instances
            .get(&handle)
            .ok_or(ParticlePreviewError::UnknownHandle(handle.raw()))?;
        Ok(ParticlePreviewClock::from_instance(instance))
    }

    /// Advances only this instance by one existing simulation tick and returns its actual age.
    /// `dt` is unscaled seconds; the instance applies its time scale once, inside `tick`.
    pub fn step_preview(
        &self,
        handle: ParticleEmitterHandle,
        dt: Real,
    ) -> Result<ParticlePreviewClock, ParticlePreviewError> {
        let mut state = self.lock_state();
        let (result, clock) = {
            let instance = state
                .instances
                .get_mut(&handle)
                .ok_or(ParticlePreviewError::UnknownHandle(handle.raw()))?;
            if instance.backend() == ParticleSimulationBackend::Gpu && !instance.fallback_to_cpu {
                return Err(ParticlePreviewError::GpuStepUnavailable);
            }
            if !dt.is_finite() || dt <= Real::EPSILON {
                return Err(ParticlePreviewError::InvalidDeltaTime);
            }
            let scaled_dt = dt * instance.component().time_scale;
            if !scaled_dt.is_finite() || scaled_dt <= 0.0 {
                return Err(ParticlePreviewError::InvalidScaledDeltaTime);
            }
            let next_age = instance.age_seconds + scaled_dt;
            if !next_age.is_finite() || next_age <= instance.age_seconds {
                return Err(ParticlePreviewError::AgeNotAdvancing);
            }

            // Keep rejection above every mutation. Restore playback before propagating tick errors.
            let was_playing = instance.playing;
            instance.play();
            let result = instance.tick(dt);
            instance.playing = was_playing;
            (result, ParticlePreviewClock::from_instance(instance))
        };
        state.invalidate_snapshot();
        result.map_err(ParticlePreviewError::Simulation)?;
        Ok(clock)
    }
}
