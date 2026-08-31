use std::collections::HashMap;

use crate::core::framework::window::{
    WindowCreateGeneration, WindowCreateSnapshot, WindowCreateSpec, WindowEffectiveGeneration,
    WindowEffectiveSnapshot, WindowEffectiveState, WindowId, WindowObservedGeneration,
    WindowObservedSnapshot, WindowObservedState, WindowRequestedGeneration,
    WindowRequestedSnapshot, WindowRequestedState, WindowStateSnapshot,
};

use super::WindowStateRegistryError;

/// Driver-owned state authority for each live `WindowId`. Native-window
/// mappings remain in `WindowRegistry`; this owner only records independent
/// create/requested/observed/effective snapshots for the matching generation.
#[derive(Default)]
pub(crate) struct WindowStateRegistry {
    entries: HashMap<WindowId, WindowStateEntry>,
}

impl WindowStateRegistry {
    pub(crate) fn register(
        &mut self,
        window: WindowId,
        create: WindowCreateSpec,
        observed: WindowObservedState,
        effective: WindowEffectiveState,
    ) -> Result<WindowStateSnapshot, WindowStateRegistryError> {
        if self.entries.contains_key(&window) {
            return Err(WindowStateRegistryError::DuplicateWindowState { window });
        }
        self.entries
            .try_reserve(1)
            .map_err(|_| WindowStateRegistryError::CapacityExhausted)?;

        let requested = WindowRequestedSnapshot::new(
            WindowRequestedGeneration::initial(),
            create.requested().clone(),
        );
        let entry = WindowStateEntry {
            create: WindowCreateSnapshot::new(WindowCreateGeneration::initial(), create),
            requested,
            observed: WindowObservedSnapshot::new(WindowObservedGeneration::initial(), observed),
            effective: WindowEffectiveSnapshot::new(
                WindowEffectiveGeneration::initial(),
                WindowRequestedGeneration::initial(),
                effective,
            ),
        };
        let snapshot = entry.snapshot(window);
        let previous = self.entries.insert(window, entry);
        debug_assert!(previous.is_none());
        Ok(snapshot)
    }

    pub(crate) fn snapshot(
        &self,
        window: WindowId,
    ) -> Result<WindowStateSnapshot, WindowStateRegistryError> {
        self.entry(window).map(|entry| entry.snapshot(window))
    }

    pub(crate) fn replace_requested(
        &mut self,
        window: WindowId,
        expected_generation: WindowRequestedGeneration,
        requested: WindowRequestedState,
    ) -> Result<WindowStateSnapshot, WindowStateRegistryError> {
        let entry = self.entry_mut(window)?;
        let actual_generation = entry.requested.generation();
        if actual_generation != expected_generation {
            return Err(WindowStateRegistryError::RequestedGenerationMismatch {
                window,
                expected: expected_generation,
                actual: actual_generation,
            });
        }
        let generation = actual_generation
            .next()
            .ok_or(WindowStateRegistryError::RequestedGenerationExhausted { window })?;
        entry.requested = WindowRequestedSnapshot::new(generation, requested);
        Ok(entry.snapshot(window))
    }

    pub(crate) fn publish_observed(
        &mut self,
        window: WindowId,
        observed: WindowObservedState,
    ) -> Result<WindowStateSnapshot, WindowStateRegistryError> {
        let entry = self.entry_mut(window)?;
        let generation = entry
            .observed
            .generation()
            .next()
            .ok_or(WindowStateRegistryError::ObservedGenerationExhausted { window })?;
        entry.observed = WindowObservedSnapshot::new(generation, observed);
        Ok(entry.snapshot(window))
    }

    pub(crate) fn publish_effective(
        &mut self,
        window: WindowId,
        source_requested: WindowRequestedGeneration,
        effective: WindowEffectiveState,
    ) -> Result<WindowStateSnapshot, WindowStateRegistryError> {
        let entry = self.entry_mut(window)?;
        validate_effective_source_generation(window, entry, source_requested)?;
        let generation = entry
            .effective
            .generation()
            .next()
            .ok_or(WindowStateRegistryError::EffectiveGenerationExhausted { window })?;
        entry.effective = WindowEffectiveSnapshot::new(generation, source_requested, effective);
        Ok(entry.snapshot(window))
    }

    /// Verifies that a native command completion can publish all state facts
    /// it owns before the driver mutates either snapshot. Effective state is
    /// a host fact, so it records the source request generation even when a
    /// newer desired state is already pending.
    pub(crate) fn preflight_command_completion(
        &self,
        window: WindowId,
        effective_source: Option<WindowRequestedGeneration>,
    ) -> Result<(), WindowStateRegistryError> {
        let entry = self.entry(window)?;
        entry
            .observed
            .generation()
            .next()
            .ok_or(WindowStateRegistryError::ObservedGenerationExhausted { window })?;
        if let Some(effective_source) = effective_source {
            validate_effective_source_generation(window, entry, effective_source)?;
            entry
                .effective
                .generation()
                .next()
                .ok_or(WindowStateRegistryError::EffectiveGenerationExhausted { window })?;
        }
        Ok(())
    }

    pub(crate) fn remove(
        &mut self,
        window: WindowId,
    ) -> Result<WindowStateSnapshot, WindowStateRegistryError> {
        self.entries
            .remove(&window)
            .map(|entry| entry.snapshot(window))
            .ok_or(WindowStateRegistryError::UnknownWindowState { window })
    }

    pub(crate) fn contains(&self, window: WindowId) -> bool {
        self.entries.contains_key(&window)
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    fn entry(&self, window: WindowId) -> Result<&WindowStateEntry, WindowStateRegistryError> {
        self.entries
            .get(&window)
            .ok_or(WindowStateRegistryError::UnknownWindowState { window })
    }

    fn entry_mut(
        &mut self,
        window: WindowId,
    ) -> Result<&mut WindowStateEntry, WindowStateRegistryError> {
        self.entries
            .get_mut(&window)
            .ok_or(WindowStateRegistryError::UnknownWindowState { window })
    }
}

fn validate_effective_source_generation(
    window: WindowId,
    entry: &WindowStateEntry,
    source_requested: WindowRequestedGeneration,
) -> Result<(), WindowStateRegistryError> {
    let current_requested = entry.requested.generation();
    if source_requested > current_requested {
        return Err(WindowStateRegistryError::EffectiveRequestGenerationAhead {
            window,
            source_requested,
            current_requested,
        });
    }
    let current_effective = entry.effective.requested_generation();
    if source_requested < current_effective {
        return Err(
            WindowStateRegistryError::EffectiveRequestGenerationRegressed {
                window,
                source_requested,
                current_effective,
            },
        );
    }
    Ok(())
}

struct WindowStateEntry {
    create: WindowCreateSnapshot,
    requested: WindowRequestedSnapshot,
    observed: WindowObservedSnapshot,
    effective: WindowEffectiveSnapshot,
}

impl WindowStateEntry {
    fn snapshot(&self, window: WindowId) -> WindowStateSnapshot {
        WindowStateSnapshot::new(
            window,
            self.create.clone(),
            self.requested.clone(),
            self.observed.clone(),
            self.effective.clone(),
        )
    }
}
