use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::render::{ParticleExtract, RenderParticleGpuReadbackOutputs};
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{Real, Vec3};

use crate::component::{ParticleEmitterHandle, ParticleSystemComponent};
use crate::interop::{ParticleAnimationEvent, ParticleAnimationEventKind};
use crate::render::{
    ParticleGpuFallbackDiagnostic, ParticleGpuFallbackReason, build_particle_extract,
};
use crate::simulation::{ParticleSimulationError, ParticleSystemInstance};
use crate::{PARTICLES_RUNTIME_CAPABILITY, ParticleSimulationBackend};

pub const PARTICLES_PHYSICS_CAPABILITY: &str = "runtime.feature.particles.physics";
pub const PARTICLES_ANIMATION_CAPABILITY: &str = "runtime.feature.particles.animation_control";
const MAX_RUNTIME_DIAGNOSTICS: usize = 256;
const MAX_RUNTIME_DIAGNOSTIC_PAGE: usize = 64;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleRuntimeDiagnosticEntry {
    pub sequence: u64,
    pub diagnostic: ParticleRuntimeDiagnostic,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParticleRuntimeDiagnosticPage {
    pub entries: Vec<ParticleRuntimeDiagnosticEntry>,
    pub oldest_available_sequence: u64,
    pub next_sequence: u64,
    pub dropped_total: u64,
    pub stale_cursor: bool,
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
    pub sprites: Arc<[crate::ParticleSpriteSnapshot]>,
    pub diagnostics: Arc<[ParticleRuntimeDiagnostic]>,
    pub diagnostic_sequence: u64,
    pub dropped_diagnostics: u64,
    pub last_gpu_feedback: Option<RenderParticleGpuReadbackOutputs>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleGpuRuntimeInstance {
    pub handle: ParticleEmitterHandle,
    pub component: ParticleSystemComponent,
    pub playing: bool,
    pub age_seconds: Real,
}

#[derive(Clone, Debug, Default)]
pub struct ParticlesManager {
    state: Arc<Mutex<ParticlesManagerState>>,
}

#[derive(Clone, Debug)]
struct ParticlesManagerState {
    next_handle: u64,
    instances: BTreeMap<ParticleEmitterHandle, ParticleSystemInstance>,
    diagnostics: VecDeque<ParticleRuntimeDiagnosticEntry>,
    next_diagnostic_sequence: Option<u64>,
    dropped_diagnostics: u64,
    diagnostics_snapshot: Arc<[ParticleRuntimeDiagnostic]>,
    diagnostics_dirty: bool,
    cached_snapshot: Option<ParticleRuntimeSnapshot>,
    last_gpu_feedback: Option<RenderParticleGpuReadbackOutputs>,
    capabilities: Vec<String>,
}

impl Default for ParticlesManagerState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            instances: BTreeMap::new(),
            diagnostics: VecDeque::with_capacity(MAX_RUNTIME_DIAGNOSTICS),
            next_diagnostic_sequence: Some(1),
            dropped_diagnostics: 0,
            diagnostics_snapshot: Arc::default(),
            diagnostics_dirty: false,
            cached_snapshot: None,
            last_gpu_feedback: None,
            capabilities: vec![PARTICLES_RUNTIME_CAPABILITY.to_string()],
        }
    }
}

impl ParticlesManagerState {
    fn invalidate_snapshot(&mut self) {
        self.cached_snapshot = None;
    }

    fn push_diagnostic(&mut self, diagnostic: ParticleRuntimeDiagnostic) {
        let Some(sequence) = self.next_diagnostic_sequence else {
            self.dropped_diagnostics = self.dropped_diagnostics.saturating_add(1);
            self.invalidate_snapshot();
            return;
        };
        self.next_diagnostic_sequence = sequence.checked_add(1);
        if self.diagnostics.len() == MAX_RUNTIME_DIAGNOSTICS {
            self.diagnostics.pop_front();
            self.dropped_diagnostics = self.dropped_diagnostics.saturating_add(1);
        }
        self.diagnostics.push_back(ParticleRuntimeDiagnosticEntry {
            sequence,
            diagnostic,
        });
        self.diagnostics_dirty = true;
        self.invalidate_snapshot();
    }

    fn shared_diagnostics(&mut self) -> Arc<[ParticleRuntimeDiagnostic]> {
        if self.diagnostics_dirty {
            self.diagnostics_snapshot = self
                .diagnostics
                .iter()
                .map(|entry| entry.diagnostic.clone())
                .collect::<Vec<_>>()
                .into();
            self.diagnostics_dirty = false;
        }
        Arc::clone(&self.diagnostics_snapshot)
    }
}

impl ParticlesManager {
    pub fn with_capabilities<S: AsRef<str>>(capabilities: &[S]) -> Self {
        let mut state = ParticlesManagerState::default();
        for capability in capabilities {
            push_unique(&mut state.capabilities, capability.as_ref().to_string());
        }
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn enable_capability(&self, capability: impl Into<String>) {
        let mut state = self.lock_state();
        let capability = capability.into();
        let enables_physics = capability == PARTICLES_PHYSICS_CAPABILITY;
        push_unique(&mut state.capabilities, capability);
        if enables_physics {
            for instance in state.instances.values_mut() {
                instance.set_physics_enabled(true);
            }
        }
        state.invalidate_snapshot();
    }

    pub fn instantiate(
        &self,
        component: ParticleSystemComponent,
    ) -> Result<ParticleEmitterHandle, ParticleSimulationError> {
        let mut state = self.lock_state();
        let handle = ParticleEmitterHandle::new(state.next_handle);
        state.next_handle = state.next_handle.saturating_add(1).max(1);
        let fallback_to_cpu = component.backend() == ParticleSimulationBackend::Gpu;
        let physics_enabled = state
            .capabilities
            .iter()
            .any(|capability| capability == PARTICLES_PHYSICS_CAPABILITY);
        let instance =
            ParticleSystemInstance::new(handle, component, fallback_to_cpu, physics_enabled)?;
        if fallback_to_cpu {
            let diagnostic = ParticleGpuFallbackDiagnostic::new(
                handle,
                ParticleGpuFallbackReason::BackendUnavailable,
                "GPU particle simulation requires a renderer-owned wgpu executor; this manager has no executor attached, so CPU simulation is active",
            );
            state.push_diagnostic(ParticleRuntimeDiagnostic::warning(
                Some(handle),
                diagnostic.message,
            ));
        }
        push_optional_feature_diagnostics(&mut state, handle, &instance);
        state.instances.insert(handle, instance);
        state.invalidate_snapshot();
        Ok(handle)
    }

    pub fn remove(&self, handle: ParticleEmitterHandle) -> Result<(), ParticleSimulationError> {
        let mut state = self.lock_state();
        if state.instances.remove(&handle).is_none() {
            return Err(ParticleSimulationError::UnknownHandle(handle.raw()));
        }
        state.invalidate_snapshot();
        Ok(())
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
        state.invalidate_snapshot();
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
        state.invalidate_snapshot();
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

    pub fn apply_animation_event(
        &self,
        event: ParticleAnimationEvent,
    ) -> Result<(), ParticleSimulationError> {
        let mut state = self.lock_state();
        if !state
            .capabilities
            .iter()
            .any(|capability| capability == PARTICLES_ANIMATION_CAPABILITY)
        {
            state.push_diagnostic(ParticleRuntimeDiagnostic::warning(
                event.handle,
                format!(
                    "animation-controlled particle event {:?} for entity {} ignored because capability `{}` is unavailable",
                    event.kind, event.entity, PARTICLES_ANIMATION_CAPABILITY
                ),
            ));
            return Ok(());
        }
        let Some(handle) = event.handle.or_else(|| {
            state.instances.iter().find_map(|(handle, instance)| {
                (instance.entity() == event.entity).then_some(*handle)
            })
        }) else {
            return Ok(());
        };
        state.invalidate_snapshot();
        let instance = state
            .instances
            .get_mut(&handle)
            .ok_or(ParticleSimulationError::UnknownHandle(handle.raw()))?;
        match event.kind {
            ParticleAnimationEventKind::SpawnOnce => instance.trigger_burst_now(),
            ParticleAnimationEventKind::TimedEmissionBegin => instance.play(),
            ParticleAnimationEventKind::TimedEmissionEnd => instance.pause(),
        }
        Ok(())
    }

    pub fn snapshot(&self) -> ParticleRuntimeSnapshot {
        let mut state = self.lock_state();
        if let Some(snapshot) = state.cached_snapshot.as_ref() {
            return snapshot.clone();
        }
        let diagnostics = state.shared_diagnostics();
        let mut snapshot = ParticleRuntimeSnapshot {
            diagnostics,
            diagnostic_sequence: state
                .next_diagnostic_sequence
                .map_or(u64::MAX, |sequence| sequence.saturating_sub(1)),
            dropped_diagnostics: state.dropped_diagnostics,
            last_gpu_feedback: state.last_gpu_feedback.clone(),
            ..ParticleRuntimeSnapshot::default()
        };
        let mut sprites = Vec::new();
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
            instance.append_sprites(&mut sprites);
        }
        snapshot.sprites = sprites.into();
        state.cached_snapshot = Some(snapshot.clone());
        snapshot
    }

    pub fn diagnostics_page(
        &self,
        after_sequence: u64,
        limit: usize,
    ) -> ParticleRuntimeDiagnosticPage {
        let state = self.lock_state();
        let oldest_available_sequence = state
            .diagnostics
            .front()
            .map(|entry| entry.sequence)
            .unwrap_or(state.next_diagnostic_sequence.unwrap_or(u64::MAX));
        let stale_cursor = after_sequence.saturating_add(1) < oldest_available_sequence;
        let entries = state
            .diagnostics
            .iter()
            .filter(|entry| entry.sequence > after_sequence)
            .take(limit.min(MAX_RUNTIME_DIAGNOSTIC_PAGE))
            .cloned()
            .collect::<Vec<_>>();
        let next_sequence = entries
            .last()
            .map(|entry| entry.sequence)
            .unwrap_or(after_sequence);
        ParticleRuntimeDiagnosticPage {
            entries,
            oldest_available_sequence,
            next_sequence,
            dropped_total: state.dropped_diagnostics,
            stale_cursor,
        }
    }

    pub fn acknowledge_diagnostics(&self, through_sequence: u64) -> usize {
        let mut state = self.lock_state();
        let mut acknowledged = 0;
        while state
            .diagnostics
            .front()
            .is_some_and(|entry| entry.sequence <= through_sequence)
        {
            state.diagnostics.pop_front();
            acknowledged += 1;
        }
        if acknowledged > 0 {
            state.diagnostics_dirty = true;
            state.invalidate_snapshot();
        }
        acknowledged
    }

    pub fn gpu_runtime_instances(&self) -> Vec<ParticleGpuRuntimeInstance> {
        let state = self.lock_state();
        state
            .instances
            .values()
            .filter(|instance| instance.backend() == ParticleSimulationBackend::Gpu)
            .map(|instance| ParticleGpuRuntimeInstance {
                handle: instance.handle,
                component: instance.component().clone(),
                playing: instance.playing,
                age_seconds: instance.age_seconds,
            })
            .collect()
    }

    pub fn build_extract(&self, camera_position: Option<Vec3>) -> ParticleExtract {
        build_particle_extract(&self.snapshot(), camera_position)
    }

    pub fn apply_gpu_feedback(&self, feedback: zircon_runtime::graphics::ParticleRuntimeFeedback) {
        let Some(outputs) = feedback
            .into_gpu_feedback()
            .map(|feedback| feedback.into_readback_outputs())
            .filter(|outputs| !outputs.is_empty())
        else {
            return;
        };

        let mut state = self.lock_state();
        state.last_gpu_feedback = Some(outputs);
        state.invalidate_snapshot();
    }

    fn with_instance(
        &self,
        handle: ParticleEmitterHandle,
        update: impl FnOnce(&mut ParticleSystemInstance),
    ) -> Result<(), ParticleSimulationError> {
        let mut state = self.lock_state();
        state.invalidate_snapshot();
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

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
        values.sort();
    }
}

fn push_optional_feature_diagnostics(
    state: &mut ParticlesManagerState,
    handle: ParticleEmitterHandle,
    instance: &ParticleSystemInstance,
) {
    if instance.requires_physics()
        && !state
            .capabilities
            .iter()
            .any(|capability| capability == PARTICLES_PHYSICS_CAPABILITY)
    {
        state.push_diagnostic(ParticleRuntimeDiagnostic::warning(
            Some(handle),
            format!(
                "particle physics modules are running as no-op because capability `{PARTICLES_PHYSICS_CAPABILITY}` is unavailable"
            ),
        ));
    }
    if instance.requires_animation()
        && !state
            .capabilities
            .iter()
            .any(|capability| capability == PARTICLES_ANIMATION_CAPABILITY)
    {
        state.push_diagnostic(ParticleRuntimeDiagnostic::warning(
            Some(handle),
            format!(
                "particle animation bindings are disabled because capability `{PARTICLES_ANIMATION_CAPABILITY}` is unavailable"
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod snapshot {
        use super::*;

        #[test]
        fn diagnostic_sequence_exhaustion_never_reuses_the_last_sequence() {
            let manager = ParticlesManager::default();
            {
                let mut state = manager.lock_state();
                state.next_diagnostic_sequence = Some(u64::MAX);
                state.push_diagnostic(ParticleRuntimeDiagnostic::warning(None, "last sequence"));
                state.push_diagnostic(ParticleRuntimeDiagnostic::warning(
                    None,
                    "must be dropped after exhaustion",
                ));
            }

            let snapshot = manager.snapshot();
            assert_eq!(snapshot.diagnostic_sequence, u64::MAX);
            assert_eq!(snapshot.diagnostics.len(), 1);
            assert_eq!(snapshot.dropped_diagnostics, 1);

            let page = manager.diagnostics_page(u64::MAX - 1, usize::MAX);
            assert_eq!(page.entries.len(), 1);
            assert_eq!(page.entries[0].sequence, u64::MAX);
        }
    }
}
