use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::render::ParticleExtract;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{Real, Vec3};

use crate::component::{ParticleEmitterHandle, ParticleSystemComponent};
use crate::render::{
    build_particle_extract, ParticleGpuFallbackDiagnostic, ParticleGpuFallbackReason,
};
use crate::simulation::{ParticleSimulationError, ParticleSystemInstance};
use crate::ParticleSimulationBackend;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleRuntimeDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleRuntimeDiagnostic {
    pub handle: Option<ParticleEmitterHandle>,
    pub severity: ParticleRuntimeDiagnosticSeverity,
    pub message: String,
}

impl ParticleRuntimeDiagnostic {
    pub fn warning(handle: Option<ParticleEmitterHandle>, message: impl Into<String>) -> Self {
        Self {
            handle,
            severity: ParticleRuntimeDiagnosticSeverity::Warning,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEmitterState {
    pub handle: ParticleEmitterHandle,
    pub emitter_id: String,
    pub entity: EntityId,
    pub live_particles: usize,
    pub allocated_particles: usize,
    pub playing: bool,
    pub backend: ParticleSimulationBackend,
    pub fallback_to_cpu: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParticleRuntimeSnapshot {
    pub emitters: Vec<ParticleEmitterState>,
    pub sprites: Vec<crate::ParticleSpriteSnapshot>,
    pub diagnostics: Vec<ParticleRuntimeDiagnostic>,
}

#[derive(Clone, Debug, Default)]
pub struct ParticlesManager {
    state: Arc<Mutex<ParticlesManagerState>>,
}

#[derive(Clone, Debug)]
struct ParticlesManagerState {
    next_handle: u64,
    instances: BTreeMap<ParticleEmitterHandle, ParticleSystemInstance>,
    diagnostics: Vec<ParticleRuntimeDiagnostic>,
}

impl Default for ParticlesManagerState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            instances: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }
}

impl ParticlesManager {
    pub fn instantiate(
        &self,
        component: ParticleSystemComponent,
    ) -> Result<ParticleEmitterHandle, ParticleSimulationError> {
        let mut state = self.lock_state();
        let handle = ParticleEmitterHandle::new(state.next_handle);
        state.next_handle = state.next_handle.saturating_add(1).max(1);
        let fallback_to_cpu = component.backend() == ParticleSimulationBackend::Gpu;
        let instance = ParticleSystemInstance::new(handle, component, fallback_to_cpu)?;
        if fallback_to_cpu {
            let diagnostic = ParticleGpuFallbackDiagnostic::new(
                handle,
                ParticleGpuFallbackReason::BackendUnavailable,
                "GPU particle simulation requires a renderer-owned wgpu executor; this manager has no executor attached, so CPU simulation is active",
            );
            state.diagnostics.push(ParticleRuntimeDiagnostic::warning(
                Some(handle),
                diagnostic.message,
            ));
        }
        state.instances.insert(handle, instance);
        Ok(handle)
    }

    pub fn remove(&self, handle: ParticleEmitterHandle) -> Result<(), ParticleSimulationError> {
        let mut state = self.lock_state();
        state
            .instances
            .remove(&handle)
            .map(|_| ())
            .ok_or(ParticleSimulationError::UnknownHandle(handle.raw()))
    }

    pub fn play(&self, handle: ParticleEmitterHandle) -> Result<(), ParticleSimulationError> {
        self.with_instance(handle, |instance| instance.play())
    }

    pub fn pause(&self, handle: ParticleEmitterHandle) -> Result<(), ParticleSimulationError> {
        self.with_instance(handle, |instance| instance.pause())
    }

    pub fn stop(&self, handle: ParticleEmitterHandle) -> Result<(), ParticleSimulationError> {
        self.with_instance(handle, |instance| instance.stop())
    }

    pub fn tick(&self, dt: Real) -> Result<(), ParticleSimulationError> {
        let mut state = self.lock_state();
        for instance in state.instances.values_mut() {
            instance.tick(dt)?;
        }
        Ok(())
    }

    pub fn rewind_preview(
        &self,
        handle: ParticleEmitterHandle,
        fixed_dt: Real,
        playback_seconds: Real,
    ) -> Result<(), ParticleSimulationError> {
        if !fixed_dt.is_finite() || fixed_dt <= 0.0 || !playback_seconds.is_finite() {
            return Err(ParticleSimulationError::InvalidDeltaTime);
        }
        let mut state = self.lock_state();
        let instance = state
            .instances
            .get_mut(&handle)
            .ok_or(ParticleSimulationError::UnknownHandle(handle.raw()))?;
        let was_playing = instance.playing;
        instance.reset_particles();
        instance.play();
        let mut remaining = playback_seconds.max(0.0);
        while remaining > Real::EPSILON {
            let dt = remaining.min(fixed_dt);
            instance.tick(dt)?;
            remaining -= dt;
        }
        if !was_playing {
            instance.pause();
        }
        Ok(())
    }

    pub fn snapshot(&self) -> ParticleRuntimeSnapshot {
        let state = self.lock_state();
        let mut snapshot = ParticleRuntimeSnapshot {
            diagnostics: state.diagnostics.clone(),
            ..ParticleRuntimeSnapshot::default()
        };
        for instance in state.instances.values() {
            for emitter_state in instance.emitter_states() {
                snapshot.emitters.push(ParticleEmitterState {
                    handle: instance.handle,
                    emitter_id: emitter_state.emitter_id,
                    entity: instance.entity(),
                    live_particles: emitter_state.live_particles,
                    allocated_particles: emitter_state.allocated_particles,
                    playing: instance.playing,
                    backend: instance.backend(),
                    fallback_to_cpu: instance.fallback_to_cpu,
                });
            }
            snapshot.sprites.extend(instance.sprites());
        }
        snapshot
    }

    pub fn build_extract(&self, camera_position: Option<Vec3>) -> ParticleExtract {
        build_particle_extract(&self.snapshot(), camera_position)
    }

    fn with_instance(
        &self,
        handle: ParticleEmitterHandle,
        update: impl FnOnce(&mut ParticleSystemInstance),
    ) -> Result<(), ParticleSimulationError> {
        let mut state = self.lock_state();
        let instance = state
            .instances
            .get_mut(&handle)
            .ok_or(ParticleSimulationError::UnknownHandle(handle.raw()))?;
        update(instance);
        Ok(())
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, ParticlesManagerState> {
        self.state.lock().expect("particles manager mutex poisoned")
    }
}
