use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use zircon_runtime_interface::reflect::ReflectTypeRegistration;

use crate::core::{CoreHandle, CoreWeak};
use crate::scene::{
    TypeRegistry, VmTypeBacking, World, WorldRuntimeExtensionError, WorldRuntimeExtensionPlan,
    WorldRuntimeExtensionRegistration,
};
use crate::script::{PluginSlotId, VmStateSchema};

use super::{VmReflectionError, VmReflectionSchema};

/// Stable scene-extension key installed by the VM plugin manager exactly once.
pub const VM_REFLECTION_WORLD_EXTENSION_NAME: &str = "script.vm.reflection.catalog";

#[derive(Clone, Debug, PartialEq)]
struct OwnedVmRegistration {
    slot: PluginSlotId,
    registration: ReflectTypeRegistration,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct CatalogState {
    registrations: BTreeMap<String, OwnedVmRegistration>,
    generations: BTreeMap<PluginSlotId, u32>,
    owners: BTreeMap<PluginSlotId, String>,
    revision: u64,
}

#[derive(Debug)]
pub(crate) struct PreparedVmReflectionGeneration {
    candidate: CatalogState,
    snapshot: VmReflectionRegistrySnapshot,
    registrations: Arc<[ReflectTypeRegistration]>,
    base_epoch: u64,
    candidate_epoch: u64,
    registration_count: usize,
}

impl PreparedVmReflectionGeneration {
    pub(crate) fn snapshot(&self) -> &VmReflectionRegistrySnapshot {
        &self.snapshot
    }
}

/// Immutable registry candidate paired with the catalog revision that owns its dense slots.
#[derive(Clone)]
pub struct VmReflectionRegistrySnapshot {
    registry: Arc<TypeRegistry>,
    revision: u64,
    base_committed_epoch: u64,
    candidate_epoch: u64,
    committed_epoch: Arc<AtomicU64>,
    current_revision: Arc<AtomicU64>,
}

impl VmReflectionRegistrySnapshot {
    /// Returns the canonical reflected registry captured for this revision.
    pub fn registry(&self) -> &TypeRegistry {
        self.registry.as_ref()
    }

    /// Returns the immutable catalog revision assigned to this registry.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the latest committed catalog revision.
    pub fn current_revision(&self) -> u64 {
        self.current_revision.load(Ordering::Acquire)
    }

    /// Returns whether dense slots compiled from this snapshot are still authoritative.
    pub fn is_current(&self) -> bool {
        self.candidate_epoch == self.committed_epoch.load(Ordering::Acquire)
    }

    /// Returns whether this exact prepared candidate may still resolve package-loading names.
    /// Runtime dispatch remains closed until `is_current` becomes true after commit.
    pub fn can_resolve_names(&self) -> bool {
        self.is_current()
            || self.base_committed_epoch == self.committed_epoch.load(Ordering::Acquire)
    }
}

/// Process-wide catalog that projects active VM schemas into existing and future Worlds.
#[derive(Clone)]
pub struct VmReflectionCatalog {
    state: Arc<RwLock<CatalogState>>,
    committed_snapshot: Arc<RwLock<VmReflectionRegistrySnapshot>>,
    mutation: Arc<Mutex<()>>,
    core: Arc<RwLock<Option<CoreWeak>>>,
    next_candidate_epoch: Arc<AtomicU64>,
    committed_epoch: Arc<AtomicU64>,
    current_revision: Arc<AtomicU64>,
}

impl Default for VmReflectionCatalog {
    fn default() -> Self {
        let committed_epoch = Arc::new(AtomicU64::new(0));
        let current_revision = Arc::new(AtomicU64::new(0));
        let committed_snapshot = VmReflectionRegistrySnapshot {
            registry: Arc::new(crate::scene::reflect::builtin_type_registry()),
            revision: 0,
            base_committed_epoch: 0,
            candidate_epoch: 0,
            committed_epoch: Arc::clone(&committed_epoch),
            current_revision: Arc::clone(&current_revision),
        };
        Self {
            state: Arc::new(RwLock::new(CatalogState::default())),
            committed_snapshot: Arc::new(RwLock::new(committed_snapshot)),
            mutation: Arc::new(Mutex::new(())),
            core: Arc::new(RwLock::new(None)),
            next_candidate_epoch: Arc::new(AtomicU64::new(1)),
            committed_epoch,
            current_revision,
        }
    }
}

impl VmReflectionCatalog {
    /// Publishes one package generation and updates every existing managed World atomically per World.
    #[cfg(test)]
    pub(crate) fn publish_generation(
        &self,
        slot: PluginSlotId,
        generation: u32,
        expected_owner: &str,
        state_schema: &VmStateSchema,
    ) -> Result<usize, VmReflectionError> {
        self.publish_optional_generation(slot, generation, expected_owner, Some(state_schema))
    }

    #[cfg(test)]
    pub(crate) fn publish_optional_generation(
        &self,
        slot: PluginSlotId,
        generation: u32,
        expected_owner: &str,
        state_schema: Option<&VmStateSchema>,
    ) -> Result<usize, VmReflectionError> {
        let prepared =
            self.prepare_optional_generation(slot, generation, expected_owner, state_schema)?;
        self.commit_prepared(prepared)
    }

    pub(crate) fn prepare_optional_generation(
        &self,
        slot: PluginSlotId,
        generation: u32,
        expected_owner: &str,
        state_schema: Option<&VmStateSchema>,
    ) -> Result<PreparedVmReflectionGeneration, VmReflectionError> {
        let _mutation = self.mutation_lock();
        let current = self.state_read().clone();
        let base_epoch = self.committed_epoch.load(Ordering::Acquire);
        let projected = state_schema
            .map(VmReflectionSchema::from_state_schema)
            .transpose()?
            .unwrap_or_default();
        let candidate = candidate_for_generation(
            &current,
            slot,
            generation,
            expected_owner,
            projected.registrations(),
        )?;
        let registrations: Arc<[ReflectTypeRegistration]> =
            registrations_from_state(&candidate).into();
        let (snapshot, candidate_epoch) = if candidate == current {
            (self.committed_snapshot_read().clone(), base_epoch)
        } else {
            let candidate_epoch = self.allocate_candidate_epoch()?;
            let registry = Arc::new(self.registry_for_state(&candidate)?);
            self.validate_existing_worlds(registrations.as_ref())?;
            (
                VmReflectionRegistrySnapshot {
                    registry,
                    revision: candidate.revision,
                    base_committed_epoch: base_epoch,
                    candidate_epoch,
                    committed_epoch: Arc::clone(&self.committed_epoch),
                    current_revision: Arc::clone(&self.current_revision),
                },
                candidate_epoch,
            )
        };
        Ok(PreparedVmReflectionGeneration {
            candidate,
            snapshot,
            registrations,
            base_epoch,
            candidate_epoch,
            registration_count: projected.registrations().len(),
        })
    }

    pub(crate) fn commit_prepared(
        &self,
        prepared: PreparedVmReflectionGeneration,
    ) -> Result<usize, VmReflectionError> {
        let _mutation = self.mutation_lock();
        if !Arc::ptr_eq(&prepared.snapshot.committed_epoch, &self.committed_epoch) {
            return Err(VmReflectionError::ForeignPreparedGeneration);
        }
        let committed_epoch = self.committed_epoch.load(Ordering::Acquire);
        if prepared.candidate_epoch == committed_epoch && prepared.base_epoch == committed_epoch {
            return Ok(prepared.registration_count);
        }
        if prepared.base_epoch != committed_epoch {
            return Err(VmReflectionError::PreparedGenerationStale {
                base_epoch: prepared.base_epoch,
                committed_epoch,
            });
        }
        self.publish_candidate(
            prepared.candidate,
            prepared.snapshot,
            prepared.registrations,
        )?;
        Ok(prepared.registration_count)
    }

    /// Removes a package slot and synchronizes the remaining authoritative registrations.
    pub(crate) fn discard_slot(&self, slot: PluginSlotId) -> Result<usize, VmReflectionError> {
        let _mutation = self.mutation_lock();
        let current = self.state_read().clone();
        let (candidate, removed) = self.candidate_without_slot(slot)?;
        if candidate == current {
            return Ok(removed);
        }
        let candidate_epoch = self.allocate_candidate_epoch()?;
        self.commit_candidate(candidate, candidate_epoch)?;
        Ok(removed)
    }

    pub(crate) fn validate_slot_discard(
        &self,
        slot: PluginSlotId,
    ) -> Result<(), VmReflectionError> {
        let _mutation = self.mutation_lock();
        let (candidate, _) = self.candidate_without_slot(slot)?;
        self.validate_candidate(&candidate)
    }

    fn commit_candidate(
        &self,
        candidate: CatalogState,
        candidate_epoch: u64,
    ) -> Result<(), VmReflectionError> {
        let base_epoch = self.committed_epoch.load(Ordering::Acquire);
        let registrations: Arc<[ReflectTypeRegistration]> =
            registrations_from_state(&candidate).into();
        let registry = Arc::new(self.registry_for_state(&candidate)?);
        self.validate_existing_worlds(registrations.as_ref())?;
        let snapshot = VmReflectionRegistrySnapshot {
            registry,
            revision: candidate.revision,
            base_committed_epoch: base_epoch,
            candidate_epoch,
            committed_epoch: Arc::clone(&self.committed_epoch),
            current_revision: Arc::clone(&self.current_revision),
        };
        self.publish_candidate(candidate, snapshot, registrations)
    }

    fn publish_candidate(
        &self,
        candidate: CatalogState,
        snapshot: VmReflectionRegistrySnapshot,
        registrations: Arc<[ReflectTypeRegistration]>,
    ) -> Result<(), VmReflectionError> {
        let revision = candidate.revision;
        let candidate_epoch = snapshot.candidate_epoch;
        if let Some(core) = self.bound_core() {
            let manager = crate::scene::resolve_default_level_manager(&core)?;
            manager.sync_vm_types_atomically(registrations.as_ref(), || {
                *self.state_write() = candidate;
                *self.committed_snapshot_write() = snapshot;
                self.current_revision.store(revision, Ordering::Release);
                self.committed_epoch
                    .store(candidate_epoch, Ordering::Release);
            })?;
        } else {
            *self.state_write() = candidate;
            *self.committed_snapshot_write() = snapshot;
            self.current_revision.store(revision, Ordering::Release);
            self.committed_epoch
                .store(candidate_epoch, Ordering::Release);
        }
        Ok(())
    }

    /// Returns the latest committed canonical registry snapshot.
    pub fn current_snapshot(&self) -> Result<VmReflectionRegistrySnapshot, VmReflectionError> {
        let _mutation = self.mutation_lock();
        Ok(self.committed_snapshot_read().clone())
    }

    /// Returns the latest committed reflection revision.
    pub fn revision(&self) -> u64 {
        self.current_revision.load(Ordering::Acquire)
    }

    /// Applies the latest catalog snapshot to a newly-created World.
    pub fn apply_to_world(&self, world: &mut World) -> Result<(), VmReflectionError> {
        let registrations = registrations_from_state(&self.state_read());
        world.sync_vm_types(&registrations)?;
        Ok(())
    }

    /// Builds the single World extension installed by the script module.
    pub fn world_runtime_extension_plan(
        &self,
    ) -> Result<WorldRuntimeExtensionPlan, VmReflectionError> {
        let catalog = self.clone();
        Ok(WorldRuntimeExtensionPlan::from_registrations([
            WorldRuntimeExtensionRegistration::new(
                VM_REFLECTION_WORLD_EXTENSION_NAME,
                move |world| {
                    catalog.apply_to_world(world).map_err(|error| {
                        WorldRuntimeExtensionError::registration_failed(
                            VM_REFLECTION_WORLD_EXTENSION_NAME,
                            error,
                        )
                    })
                },
            ),
        ])?)
    }

    pub(crate) fn bind_core(&self, core: &CoreHandle) {
        *self.core_write() = Some(core.downgrade());
    }

    fn validate_existing_worlds(
        &self,
        registrations: &[ReflectTypeRegistration],
    ) -> Result<(), VmReflectionError> {
        let Some(core) = self.bound_core() else {
            return Ok(());
        };
        let manager = crate::scene::resolve_default_level_manager(&core)?;
        manager.try_for_each_world(|world| world.validate_vm_type_sync(registrations))?;
        Ok(())
    }

    fn validate_candidate(&self, candidate: &CatalogState) -> Result<(), VmReflectionError> {
        self.registry_for_state(candidate)?;
        self.validate_existing_worlds(&registrations_from_state(candidate))
    }

    fn registry_for_state(
        &self,
        candidate: &CatalogState,
    ) -> Result<TypeRegistry, VmReflectionError> {
        let mut registry = crate::scene::reflect::builtin_type_registry();
        let current_paths = self
            .state_read()
            .registrations
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for type_path in current_paths {
            registry.remove_vm_type(&type_path)?;
        }
        for registration in candidate.registrations.values() {
            registry.register_vm_type(
                registration.registration.clone(),
                VmTypeBacking::DynamicComponent,
            )?;
        }
        Ok(registry)
    }

    fn bound_core(&self) -> Option<CoreHandle> {
        self.core_read().as_ref().and_then(CoreWeak::upgrade)
    }

    fn candidate_without_slot(
        &self,
        slot: PluginSlotId,
    ) -> Result<(CatalogState, usize), VmReflectionError> {
        let mut candidate = self.state_read().clone();
        let previous_len = candidate.registrations.len();
        candidate
            .registrations
            .retain(|_, registration| registration.slot != slot);
        let removed_generation = candidate.generations.remove(&slot).is_some();
        let removed_owner = candidate.owners.remove(&slot).is_some();
        let removed = previous_len.saturating_sub(candidate.registrations.len());
        if removed > 0 || removed_generation || removed_owner {
            candidate.revision = next_revision(candidate.revision)?;
        }
        Ok((candidate, removed))
    }

    fn allocate_candidate_epoch(&self) -> Result<u64, VmReflectionError> {
        self.next_candidate_epoch
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| VmReflectionError::CandidateEpochExhausted)
    }

    fn state_read(&self) -> RwLockReadGuard<'_, CatalogState> {
        self.state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn mutation_lock(&self) -> MutexGuard<'_, ()> {
        self.mutation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn state_write(&self) -> RwLockWriteGuard<'_, CatalogState> {
        self.state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn committed_snapshot_read(&self) -> RwLockReadGuard<'_, VmReflectionRegistrySnapshot> {
        self.committed_snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn committed_snapshot_write(&self) -> RwLockWriteGuard<'_, VmReflectionRegistrySnapshot> {
        self.committed_snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn core_read(&self) -> RwLockReadGuard<'_, Option<CoreWeak>> {
        self.core
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn core_write(&self) -> RwLockWriteGuard<'_, Option<CoreWeak>> {
        self.core
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn candidate_for_generation(
    current: &CatalogState,
    slot: PluginSlotId,
    generation: u32,
    expected_owner: &str,
    registrations: &[ReflectTypeRegistration],
) -> Result<CatalogState, VmReflectionError> {
    validate_package_owner(expected_owner, registrations)?;
    let mut candidate = current.clone();
    if let Some(current_owner) = candidate.owners.get(&slot) {
        if current_owner != expected_owner {
            return Err(VmReflectionError::SlotOwnerConflict {
                slot,
                current_owner: current_owner.clone(),
                requested_owner: expected_owner.to_string(),
            });
        }
    }
    if let Some(current_generation) = candidate.generations.get(&slot).copied() {
        if generation < current_generation {
            return Err(VmReflectionError::GenerationRegression {
                slot,
                current_generation,
                requested_generation: generation,
            });
        }
        if generation == current_generation {
            let current_registrations = candidate
                .registrations
                .iter()
                .filter(|(_, registration)| registration.slot == slot)
                .map(|(type_path, registration)| {
                    (type_path.clone(), registration.registration.clone())
                })
                .collect::<BTreeMap<_, _>>();
            let requested_registrations = registrations
                .iter()
                .map(|registration| {
                    (
                        registration.type_path.type_path().to_string(),
                        registration.clone(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            if current_registrations == requested_registrations {
                return Ok(candidate);
            }
            return Err(VmReflectionError::GenerationConflict { slot, generation });
        }
    }
    candidate
        .registrations
        .retain(|_, registration| registration.slot != slot);
    for registration in registrations {
        let type_path = registration.type_path.type_path().to_string();
        if let Some(existing) = candidate.registrations.get(&type_path) {
            return Err(VmReflectionError::TypePathOwnedByAnotherSlot {
                type_path,
                owner_slot: existing.slot,
                requesting_slot: slot,
            });
        }
        candidate.registrations.insert(
            type_path,
            OwnedVmRegistration {
                slot,
                registration: registration.clone(),
            },
        );
    }
    candidate.generations.insert(slot, generation);
    candidate.owners.insert(slot, expected_owner.to_string());
    candidate.revision = next_revision(candidate.revision)?;
    Ok(candidate)
}

fn validate_package_owner(
    expected_owner: &str,
    registrations: &[ReflectTypeRegistration],
) -> Result<(), VmReflectionError> {
    for registration in registrations {
        let type_path_owner = registration.type_path.plugin_id().unwrap_or("<missing>");
        if type_path_owner != expected_owner {
            return Err(VmReflectionError::PackageOwnerMismatch {
                type_path: registration.type_path.type_path().to_string(),
                expected_owner: expected_owner.to_string(),
                declared_owner: type_path_owner.to_string(),
            });
        }
    }
    Ok(())
}

fn next_revision(current: u64) -> Result<u64, VmReflectionError> {
    current
        .checked_add(1)
        .ok_or(VmReflectionError::RevisionExhausted)
}

fn registrations_from_state(state: &CatalogState) -> Vec<ReflectTypeRegistration> {
    state
        .registrations
        .values()
        .map(|entry| entry.registration.clone())
        .collect()
}

impl fmt::Debug for VmReflectionCatalog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VmReflectionCatalog")
            .field("registration_count", &self.state_read().registrations.len())
            .field("revision", &self.revision())
            .field(
                "committed_epoch",
                &self.committed_epoch.load(Ordering::Acquire),
            )
            .field("bound_to_core", &self.bound_core().is_some())
            .finish()
    }
}

impl fmt::Debug for VmReflectionRegistrySnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VmReflectionRegistrySnapshot")
            .field("revision", &self.revision)
            .field("candidate_epoch", &self.candidate_epoch)
            .field("current", &self.is_current())
            .finish()
    }
}
