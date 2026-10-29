use super::*;

impl ParticlesManager {
    /// Render preparation shares simulation state, while each initialized runtime
    /// service owns only the scene handles created through that service.
    pub(crate) fn for_session(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            scene_handles: Arc::default(),
        }
    }

    pub(crate) fn same_session(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.scene_handles, &other.scene_handles)
    }

    pub(crate) fn release_scene_emitters(&self) {
        let handles = std::mem::take(
            &mut *self
                .scene_handles
                .lock()
                .expect("particle scene handles mutex poisoned"),
        );
        let mut state = self.lock_state();
        for handle in handles {
            state.instances.remove(&handle);
        }
        state.invalidate_snapshot();
    }

    pub(crate) fn tick_scene_emitter(
        &self,
        previous: Option<ParticleEmitterHandle>,
        component: ParticleSystemComponent,
        dt: Real,
    ) -> Result<ParticleEmitterHandle, ParticleSimulationError> {
        if !dt.is_finite() || dt < 0.0 {
            return Err(ParticleSimulationError::InvalidDeltaTime);
        }
        if !component.time_scale.is_finite() {
            return Err(ParticleSimulationError::InvalidAsset(
                "time scale must be finite".to_string(),
            ));
        }
        {
            let mut state = self.lock_state();
            state.invalidate_snapshot();
            if let Some((handle, instance)) = previous.and_then(|handle| {
                state
                    .instances
                    .get_mut(&handle)
                    .map(|instance| (handle, instance))
            }) {
                if instance.component.asset == component.asset {
                    // Movement and time scaling must not restart emission or its RNG.
                    // Preserve service playback controls when the scene is unchanged.
                    instance.component = component;
                    instance.tick(dt)?;
                    return Ok(handle);
                }
            }
        }
        if let Some(handle) = previous {
            let _ = self.remove(handle);
        }
        let handle = self.instantiate(component)?;
        self.scene_handles
            .lock()
            .expect("particle scene handles mutex poisoned")
            .insert(handle);
        let mut state = self.lock_state();
        state.invalidate_snapshot();
        state
            .instances
            .get_mut(&handle)
            .expect("new scene emitter must exist")
            .tick(dt)?;
        Ok(handle)
    }
}
