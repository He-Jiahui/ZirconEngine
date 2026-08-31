use std::collections::{HashMap, HashSet};

use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::{
    PreparedSurfaceLease, SurfaceLease, SurfaceLeaseError, SurfaceLeaseGeneration,
    SurfaceLeasePublication, SurfaceLeaseRequest, SurfaceLeaseRetirementPlan,
};
use crate::core::framework::window::{DisplayTopologySnapshot, WindowId};

/// Platform-host owner for qualified surface binding authority. It does not
/// own native windows or graphics resources: callers must validate the window
/// registry first, create/fence graphics resources outside this registry, and
/// then publish or retire the returned lease at the documented transition.
#[derive(Default)]
pub struct SurfaceLeaseRegistry {
    entries: HashMap<SurfaceLeaseKey, SurfaceLeaseEntry>,
    viewport_to_window: HashMap<ZrRuntimeViewportHandle, WindowId>,
    last_generation: u64,
}

impl SurfaceLeaseRegistry {
    pub(crate) fn prepare(
        &mut self,
        request: SurfaceLeaseRequest,
        topology: &DisplayTopologySnapshot,
    ) -> Result<PreparedSurfaceLease, SurfaceLeaseError> {
        if !request.viewport().is_valid() {
            return Err(SurfaceLeaseError::InvalidViewport {
                viewport: request.viewport(),
            });
        }
        if request.topology_generation() != topology.generation() {
            return Err(SurfaceLeaseError::TopologyGenerationMismatch {
                requested: request.topology_generation(),
                observed: topology.generation(),
            });
        }
        if !topology.contains(request.output()) {
            return Err(SurfaceLeaseError::OutputUnavailable {
                output: request.output().clone(),
                topology_generation: topology.generation(),
            });
        }

        let key = SurfaceLeaseKey::from_request(&request);
        let entry_exists = self.entries.contains_key(&key);
        match self.viewport_to_window.get(&request.viewport()).copied() {
            Some(bound_window) if bound_window != request.window() => {
                let bound_key = SurfaceLeaseKey {
                    window: bound_window,
                    viewport: request.viewport(),
                };
                return match self.entries.get(&bound_key) {
                    Some(SurfaceLeaseEntry::Active { .. }) => {
                        Err(SurfaceLeaseError::ViewportAlreadyBound {
                            viewport: request.viewport(),
                            window: bound_window,
                        })
                    }
                    Some(SurfaceLeaseEntry::Preparing { .. }) => {
                        Err(SurfaceLeaseError::ReplacementInFlight {
                            window: bound_window,
                            viewport: request.viewport(),
                        })
                    }
                    Some(SurfaceLeaseEntry::Retiring { lease }) => {
                        Err(SurfaceLeaseError::LeaseRetiring {
                            lease: lease.clone(),
                        })
                    }
                    None => Err(SurfaceLeaseError::InconsistentViewportBinding {
                        window: bound_window,
                        viewport: request.viewport(),
                    }),
                };
            }
            Some(_) if !entry_exists => {
                return Err(SurfaceLeaseError::InconsistentViewportBinding {
                    window: request.window(),
                    viewport: request.viewport(),
                });
            }
            None if entry_exists => {
                return Err(SurfaceLeaseError::InconsistentViewportBinding {
                    window: request.window(),
                    viewport: request.viewport(),
                });
            }
            None => {
                if let Some((bound_key, _)) = self
                    .entries
                    .iter()
                    .find(|(candidate, _)| candidate.viewport == request.viewport())
                {
                    return Err(SurfaceLeaseError::InconsistentViewportBinding {
                        window: bound_key.window,
                        viewport: request.viewport(),
                    });
                }
            }
            Some(_) => {}
        }
        let active = match self.entries.get(&key) {
            Some(SurfaceLeaseEntry::Preparing { .. }) => {
                return Err(SurfaceLeaseError::ReplacementInFlight {
                    window: request.window(),
                    viewport: request.viewport(),
                });
            }
            Some(SurfaceLeaseEntry::Retiring { lease }) => {
                return Err(SurfaceLeaseError::LeaseRetiring {
                    lease: lease.clone(),
                });
            }
            Some(SurfaceLeaseEntry::Active { lease }) => Some(lease.clone()),
            None => {
                self.entries
                    .try_reserve(1)
                    .map_err(|_| SurfaceLeaseError::CapacityExhausted)?;
                self.viewport_to_window
                    .try_reserve(1)
                    .map_err(|_| SurfaceLeaseError::CapacityExhausted)?;
                None
            }
        };
        let candidate = SurfaceLease::new(request, self.next_generation()?);
        if !entry_exists {
            let previous = self
                .viewport_to_window
                .insert(candidate.viewport(), candidate.window());
            debug_assert!(previous.is_none());
        }
        self.entries.insert(
            key,
            SurfaceLeaseEntry::Preparing {
                active,
                candidate: candidate.clone(),
            },
        );
        Ok(PreparedSurfaceLease::new(candidate))
    }

    /// Publishes only the exact candidate returned by `prepare`. If a prior
    /// lease existed it becomes stale for routing immediately and is returned
    /// solely for graphics-resource retirement.
    pub(crate) fn publish(
        &mut self,
        prepared: &PreparedSurfaceLease,
        topology: &DisplayTopologySnapshot,
    ) -> Result<SurfaceLeasePublication, SurfaceLeaseError> {
        let candidate = prepared.candidate();
        if candidate.topology_generation() != topology.generation() {
            return Err(SurfaceLeaseError::TopologyGenerationMismatch {
                requested: candidate.topology_generation(),
                observed: topology.generation(),
            });
        }
        if !topology.contains(candidate.output()) {
            return Err(SurfaceLeaseError::OutputUnavailable {
                output: candidate.output().clone(),
                topology_generation: topology.generation(),
            });
        }
        let key = SurfaceLeaseKey::from_lease(candidate);
        self.ensure_viewport_binding(key)?;
        let Some(entry) = self.entries.get(&key) else {
            return Err(SurfaceLeaseError::StaleLease {
                lease: candidate.clone(),
            });
        };
        let SurfaceLeaseEntry::Preparing {
            active,
            candidate: registered,
        } = entry
        else {
            return Err(SurfaceLeaseError::StaleLease {
                lease: candidate.clone(),
            });
        };
        if registered != candidate {
            return Err(SurfaceLeaseError::StaleLease {
                lease: candidate.clone(),
            });
        }
        let retired = active.clone();
        self.entries.insert(
            key,
            SurfaceLeaseEntry::Active {
                lease: candidate.clone(),
            },
        );
        Ok(SurfaceLeasePublication::new(candidate.clone(), retired))
    }

    /// Abandons a preparation failure without disturbing the last routable
    /// surface lease for the window and viewport.
    pub(crate) fn cancel(
        &mut self,
        prepared: &PreparedSurfaceLease,
    ) -> Result<(), SurfaceLeaseError> {
        let candidate = prepared.candidate();
        let key = SurfaceLeaseKey::from_lease(candidate);
        self.ensure_viewport_binding(key)?;
        let Some(entry) = self.entries.get(&key) else {
            return Err(SurfaceLeaseError::StaleLease {
                lease: candidate.clone(),
            });
        };
        let SurfaceLeaseEntry::Preparing {
            active,
            candidate: registered,
        } = entry
        else {
            return Err(SurfaceLeaseError::StaleLease {
                lease: candidate.clone(),
            });
        };
        if registered != candidate {
            return Err(SurfaceLeaseError::StaleLease {
                lease: candidate.clone(),
            });
        }
        let active = active.clone();
        if let Some(active) = active {
            self.entries
                .insert(key, SurfaceLeaseEntry::Active { lease: active });
            return Ok(());
        }
        let removed_entry = self.entries.remove(&key);
        debug_assert!(removed_entry.is_some());
        let removed_window = self.viewport_to_window.remove(&candidate.viewport());
        debug_assert_eq!(removed_window, Some(candidate.window()));
        Ok(())
    }

    /// Marks one exact active lease as non-routable before the caller releases
    /// its graphics surface. New preparation is rejected until retirement is
    /// completed, preventing a destroyed native window from being rebound.
    pub(crate) fn begin_retirement(
        &mut self,
        lease: &SurfaceLease,
    ) -> Result<(), SurfaceLeaseError> {
        let key = SurfaceLeaseKey::from_lease(lease);
        self.ensure_viewport_binding(key)?;
        let Some(entry) = self.entries.get(&key) else {
            return Err(SurfaceLeaseError::StaleLease {
                lease: lease.clone(),
            });
        };
        match entry {
            SurfaceLeaseEntry::Active { lease: active } if active == lease => {
                self.entries.insert(
                    key,
                    SurfaceLeaseEntry::Retiring {
                        lease: lease.clone(),
                    },
                );
                Ok(())
            }
            SurfaceLeaseEntry::Retiring { lease: retiring } if retiring == lease => {
                Err(SurfaceLeaseError::LeaseRetiring {
                    lease: lease.clone(),
                })
            }
            SurfaceLeaseEntry::Preparing { .. } => Err(SurfaceLeaseError::ReplacementInFlight {
                window: lease.window(),
                viewport: lease.viewport(),
            }),
            _ => Err(SurfaceLeaseError::StaleLease {
                lease: lease.clone(),
            }),
        }
    }

    /// Begins retirement for every active lease associated with one window
    /// generation. The all-or-nothing preflight rejects a pending replacement
    /// before any viewport route becomes unavailable.
    pub(crate) fn begin_retire_window(
        &mut self,
        window: WindowId,
    ) -> Result<Vec<SurfaceLease>, SurfaceLeaseError> {
        let plan = self.plan_window_retirement(std::slice::from_ref(&window))?;
        Ok(self.commit_retirement(plan))
    }

    /// Preflights retirement for every surface lease while preserving every
    /// native window route. This is the surface-only half of suspend and
    /// `destroy_surfaces`; the caller releases graphics resources after the
    /// returned leases become non-routable.
    pub(crate) fn plan_all_retirement(
        &self,
    ) -> Result<SurfaceLeaseRetirementPlan, SurfaceLeaseError> {
        for (viewport, window) in &self.viewport_to_window {
            if !self.entries.contains_key(&SurfaceLeaseKey {
                window: *window,
                viewport: *viewport,
            }) {
                return Err(SurfaceLeaseError::InconsistentViewportBinding {
                    window: *window,
                    viewport: *viewport,
                });
            }
        }
        let mut unique_windows = HashSet::new();
        for key in self.entries.keys() {
            if unique_windows.contains(&key.window) {
                continue;
            }
            unique_windows
                .try_reserve(1)
                .map_err(|_| SurfaceLeaseError::CapacityExhausted)?;
            let inserted = unique_windows.insert(key.window);
            debug_assert!(inserted);
        }
        let mut windows = Vec::new();
        windows
            .try_reserve(unique_windows.len())
            .map_err(|_| SurfaceLeaseError::CapacityExhausted)?;
        windows.extend(unique_windows);
        windows.sort_unstable_by_key(|window| {
            (window.registry().raw(), window.slot(), window.generation())
        });
        self.plan_window_retirement(&windows)
    }

    /// Collects every active lease in child-first window order before any
    /// route changes. The plan is intentionally opaque so only the registry
    /// can commit its already-validated state transition.
    pub(crate) fn plan_window_retirement(
        &self,
        windows: &[WindowId],
    ) -> Result<SurfaceLeaseRetirementPlan, SurfaceLeaseError> {
        let mut window_positions = HashMap::new();
        window_positions
            .try_reserve(windows.len())
            .map_err(|_| SurfaceLeaseError::CapacityExhausted)?;
        for (position, window) in windows.iter().copied().enumerate() {
            let previous = window_positions.insert(window, position);
            debug_assert!(previous.is_none(), "window close orders must be unique");
        }

        let mut active_count = 0_usize;
        for (key, entry) in &self.entries {
            if !window_positions.contains_key(&key.window) {
                continue;
            }
            self.ensure_viewport_binding(*key)?;
            match entry {
                SurfaceLeaseEntry::Preparing { .. } => {
                    return Err(SurfaceLeaseError::WindowHasPreparedLease { window: key.window });
                }
                SurfaceLeaseEntry::Retiring { lease } => {
                    return Err(SurfaceLeaseError::LeaseRetiring {
                        lease: lease.clone(),
                    });
                }
                SurfaceLeaseEntry::Active { .. } => {
                    active_count = active_count
                        .checked_add(1)
                        .ok_or(SurfaceLeaseError::CapacityExhausted)?;
                }
            }
        }
        for (viewport, bound_window) in &self.viewport_to_window {
            if window_positions.contains_key(bound_window)
                && !self.entries.contains_key(&SurfaceLeaseKey {
                    window: *bound_window,
                    viewport: *viewport,
                })
            {
                return Err(SurfaceLeaseError::InconsistentViewportBinding {
                    window: *bound_window,
                    viewport: *viewport,
                });
            }
        }

        let mut retiring_leases = Vec::new();
        retiring_leases
            .try_reserve(active_count)
            .map_err(|_| SurfaceLeaseError::CapacityExhausted)?;
        let mut registry_leases = Vec::new();
        registry_leases
            .try_reserve(active_count)
            .map_err(|_| SurfaceLeaseError::CapacityExhausted)?;
        for (key, entry) in &self.entries {
            if !window_positions.contains_key(&key.window) {
                continue;
            }
            if let SurfaceLeaseEntry::Active { lease } = entry {
                retiring_leases.push(lease.clone());
                registry_leases.push(lease.clone());
            }
        }
        let retirement_order = |lease: &SurfaceLease| {
            (
                window_positions.get(&lease.window()).copied(),
                lease.viewport().raw(),
            )
        };
        retiring_leases.sort_by_key(|lease| retirement_order(lease));
        registry_leases.sort_by_key(|lease| retirement_order(lease));
        Ok(SurfaceLeaseRetirementPlan::new(
            retiring_leases,
            registry_leases,
        ))
    }

    pub(crate) fn commit_retirement(
        &mut self,
        plan: SurfaceLeaseRetirementPlan,
    ) -> Vec<SurfaceLease> {
        let (retiring_leases, registry_leases) = plan.into_parts();
        debug_assert_eq!(retiring_leases.len(), registry_leases.len());
        for (retiring_lease, registry_lease) in retiring_leases.iter().zip(&registry_leases) {
            debug_assert_eq!(retiring_lease, registry_lease);
            let key = SurfaceLeaseKey::from_lease(retiring_lease);
            debug_assert!(self.ensure_viewport_binding(key).is_ok());
            debug_assert!(matches!(
                self.entries.get(&key),
                Some(SurfaceLeaseEntry::Active { lease: active }) if active == retiring_lease
            ));
        }
        for registry_lease in registry_leases {
            let key = SurfaceLeaseKey::from_lease(&registry_lease);
            let Some(entry) = self.entries.get_mut(&key) else {
                unreachable!("a locked retirement plan cannot lose a surface lease");
            };
            match entry {
                SurfaceLeaseEntry::Active { lease: active } if active == &registry_lease => {
                    *entry = SurfaceLeaseEntry::Retiring {
                        lease: registry_lease,
                    };
                }
                _ => unreachable!("a locked retirement plan cannot change lease state"),
            }
        }
        retiring_leases
    }

    /// Removes a lease only after the graphics owner has dropped its native
    /// surface and no future submission can use it.
    pub(crate) fn complete_retirement(
        &mut self,
        lease: &SurfaceLease,
    ) -> Result<(), SurfaceLeaseError> {
        let key = SurfaceLeaseKey::from_lease(lease);
        self.ensure_viewport_binding(key)?;
        let Some(entry) = self.entries.get(&key) else {
            return Err(SurfaceLeaseError::StaleLease {
                lease: lease.clone(),
            });
        };
        if matches!(entry, SurfaceLeaseEntry::Retiring { lease: retiring } if retiring == lease) {
            let removed_entry = self.entries.remove(&key);
            debug_assert!(removed_entry.is_some());
            let removed_window = self.viewport_to_window.remove(&lease.viewport());
            debug_assert_eq!(removed_window, Some(lease.window()));
            return Ok(());
        }
        if matches!(entry, SurfaceLeaseEntry::Active { lease: active } if active == lease) {
            return Err(SurfaceLeaseError::LeaseNotRetiring {
                lease: lease.clone(),
            });
        }
        Err(SurfaceLeaseError::StaleLease {
            lease: lease.clone(),
        })
    }

    pub(crate) fn active(&self, lease: &SurfaceLease) -> Result<(), SurfaceLeaseError> {
        let key = SurfaceLeaseKey::from_lease(lease);
        self.ensure_viewport_binding(key)?;
        let Some(entry) = self.entries.get(&key) else {
            return Err(SurfaceLeaseError::StaleLease {
                lease: lease.clone(),
            });
        };
        match entry {
            SurfaceLeaseEntry::Active { lease: active } if active == lease => Ok(()),
            SurfaceLeaseEntry::Preparing {
                active: Some(active),
                ..
            } if active == lease => Ok(()),
            SurfaceLeaseEntry::Preparing { candidate, .. } if candidate == lease => {
                Err(SurfaceLeaseError::ReplacementInFlight {
                    window: lease.window(),
                    viewport: lease.viewport(),
                })
            }
            SurfaceLeaseEntry::Retiring { lease: retiring } if retiring == lease => {
                Err(SurfaceLeaseError::LeaseRetiring {
                    lease: lease.clone(),
                })
            }
            _ => Err(SurfaceLeaseError::StaleLease {
                lease: lease.clone(),
            }),
        }
    }

    pub(crate) fn active_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| entry.active().is_some())
            .count()
    }

    pub(crate) fn preparing_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| matches!(entry, SurfaceLeaseEntry::Preparing { .. }))
            .count()
    }

    pub(crate) fn retiring_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| matches!(entry, SurfaceLeaseEntry::Retiring { .. }))
            .count()
    }

    fn next_generation(&mut self) -> Result<SurfaceLeaseGeneration, SurfaceLeaseError> {
        let next = self
            .last_generation
            .checked_add(1)
            .ok_or(SurfaceLeaseError::GenerationExhausted)?;
        let generation =
            SurfaceLeaseGeneration::new(next).ok_or(SurfaceLeaseError::GenerationExhausted)?;
        self.last_generation = next;
        Ok(generation)
    }

    fn ensure_viewport_binding(&self, key: SurfaceLeaseKey) -> Result<(), SurfaceLeaseError> {
        if self.viewport_to_window.get(&key.viewport).copied() != Some(key.window) {
            return Err(SurfaceLeaseError::InconsistentViewportBinding {
                window: key.window,
                viewport: key.viewport,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SurfaceLeaseKey {
    window: WindowId,
    viewport: ZrRuntimeViewportHandle,
}

impl SurfaceLeaseKey {
    const fn from_request(request: &SurfaceLeaseRequest) -> Self {
        Self {
            window: request.window(),
            viewport: request.viewport(),
        }
    }

    const fn from_lease(lease: &SurfaceLease) -> Self {
        Self {
            window: lease.window(),
            viewport: lease.viewport(),
        }
    }
}

enum SurfaceLeaseEntry {
    Active {
        lease: SurfaceLease,
    },
    Preparing {
        active: Option<SurfaceLease>,
        candidate: SurfaceLease,
    },
    Retiring {
        lease: SurfaceLease,
    },
}

impl SurfaceLeaseEntry {
    const fn active(&self) -> Option<&SurfaceLease> {
        match self {
            Self::Active { lease } => Some(lease),
            Self::Preparing { active, .. } => active.as_ref(),
            Self::Retiring { .. } => None,
        }
    }
}
