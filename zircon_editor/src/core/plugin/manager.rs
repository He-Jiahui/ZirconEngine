//! Single owner for editor-plugin catalog publication and stable reads.

use std::sync::{Arc, Mutex, OnceLock, RwLock};

use zircon_runtime::{
    core::framework::project::ProjectPluginManifest,
    plugin::{PluginPackageManifest, RuntimePluginCatalog},
};

use super::admission::validate_catalog_admission;
use super::catalog::{EditorPluginCatalog, EditorPluginHandle};
use super::catalog_snapshot::EditorPluginCatalogSnapshot;
use super::catalog_store::EditorPluginCatalogStore;
use super::descriptor::EditorPluginDescriptor;
use super::phases::EditorPluginLoadingPhase;
use super::sdk::lifecycle::{EditorPluginLifecycleEvent, EditorPluginLifecycleReport};

mod discovery;
mod lifecycle_replacement;
mod project_registration;
mod project_selection;
mod publication;
mod snapshot;
mod state;

use discovery::discovery_index;
pub use discovery::{EditorPluginDiscovery, EditorPluginDiscoveryError, EditorPluginSource};
use lifecycle_replacement::{
    dispatch_hot_reloaded_replacements, replaced_live_package_ids, reset_replaced_active_entries,
    retire_replaced_active_entries,
};
pub(crate) use project_registration::ProjectPluginRegistrationCloseReceipt;
pub use snapshot::{EditorPluginManagerEntry, EditorPluginManagerSnapshot};
use state::{
    activate_eligible_entries, apply_enablement_request, has_failed_disabled_lifecycle,
    is_manager_owned_activation_stage, is_phase_gated_state, normalize_entries_for_loading_phase,
    phase_is_reached, state_after_enablement_request,
};
pub use state::{EditorPluginState, EditorPluginTransitionError};

static BUILTIN_EDITOR_PLUGIN_MANAGER: OnceLock<EditorPluginManager> = OnceLock::new();
static BUILTIN_EDITOR_PLUGIN_MANAGER_INIT: Mutex<()> = Mutex::new(());

fn initialize_once<'a, T, E>(
    slot: &'a OnceLock<T>,
    initialization: &Mutex<()>,
    create: impl FnOnce() -> Result<T, E>,
) -> Result<&'a T, E> {
    if let Some(value) = slot.get() {
        return Ok(value);
    }
    let _initialization = initialization
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(value) = slot.get() {
        return Ok(value);
    }
    let _ = slot.set(create()?);
    Ok(slot
        .get()
        .expect("serialized once initialization must publish its value"))
}

/// Owns the mutable publication slot for the editor-plugin catalog and its lifecycle rows.
///
/// Consumers retain an [`EditorPluginManagerSnapshot`] for the complete read operation.
/// Replacing the catalog or toggling a plugin replaces that read model atomically, so no
/// consumer observes manager state paired with a different catalog generation.
#[derive(Debug)]
pub struct EditorPluginManager {
    catalog_store: EditorPluginCatalogStore,
    snapshot: RwLock<Arc<EditorPluginManagerSnapshot>>,
    lifecycle_mutation: Mutex<()>,
}

impl EditorPluginManager {
    pub fn builtin(
        runtime_manifests: impl IntoIterator<Item = PluginPackageManifest>,
    ) -> Result<Self, EditorPluginDiscoveryError> {
        let manager = Self::new(EditorPluginCatalog::builtin(runtime_manifests))?;
        manager
            .advance_loading_phase(EditorPluginLoadingPhase::Default)
            .map_err(|_| EditorPluginDiscoveryError::BuiltinInitialization)?;
        Ok(manager)
    }

    /// Creates the manager-owned catalog from lifecycle-capable plugin handles.
    pub fn from_plugins(
        plugins: impl IntoIterator<Item = (EditorPluginHandle, PluginPackageManifest)>,
    ) -> Result<Self, EditorPluginDiscoveryError> {
        Self::new(EditorPluginCatalog::from_plugins(plugins))
    }

    /// Creates the manager-owned catalog from serialized editor descriptors.
    pub fn from_descriptors(
        descriptors: impl IntoIterator<Item = EditorPluginDescriptor>,
        runtime_manifests: impl IntoIterator<Item = PluginPackageManifest>,
    ) -> Result<Self, EditorPluginDiscoveryError> {
        Self::new(EditorPluginCatalog::from_descriptors(
            descriptors,
            runtime_manifests,
        ))
    }

    /// Builds the first immutable manager generation without additional discovery metadata.
    pub(crate) fn new(catalog: EditorPluginCatalog) -> Result<Self, EditorPluginDiscoveryError> {
        Self::new_with_discoveries(catalog, std::iter::empty())
    }

    /// Builds the first immutable manager generation from one validated discovery per package.
    pub(crate) fn new_with_discoveries(
        catalog: EditorPluginCatalog,
        discoveries: impl IntoIterator<Item = EditorPluginDiscovery>,
    ) -> Result<Self, EditorPluginDiscoveryError> {
        validate_catalog_admission(&catalog)?;
        let discoveries = discovery_index(&catalog, discoveries)?;
        let catalog_store = EditorPluginCatalogStore::new(catalog);
        let catalog = catalog_store.snapshot();
        let snapshot =
            EditorPluginManagerSnapshot::from_catalog(1, catalog, &[], &discoveries, None);
        Ok(Self {
            catalog_store,
            snapshot: RwLock::new(Arc::new(snapshot)),
            lifecycle_mutation: Mutex::new(()),
        })
    }

    /// Returns the process-wide builtin catalog owner shared by UI and headless consumers.
    pub fn builtin_shared() -> Result<&'static Self, EditorPluginDiscoveryError> {
        initialize_once(
            &BUILTIN_EDITOR_PLUGIN_MANAGER,
            &BUILTIN_EDITOR_PLUGIN_MANAGER_INIT,
            || Self::builtin(RuntimePluginCatalog::builtin().package_manifests().cloned()),
        )
    }

    /// Returns the manager state paired with the catalog generation it references.
    pub fn state_snapshot(&self) -> Arc<EditorPluginManagerSnapshot> {
        Arc::clone(
            &self
                .snapshot
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    /// Returns the current catalog generation without rebuilding a projection.
    pub fn catalog_snapshot(&self) -> Arc<EditorPluginCatalogSnapshot> {
        Arc::clone(self.state_snapshot().catalog_snapshot())
    }

    /// Publishes exactly one legal lifecycle state transition for a known plugin.
    pub fn transition_state(
        &self,
        package_id: &str,
        next_state: EditorPluginState,
    ) -> Result<Arc<EditorPluginManagerSnapshot>, EditorPluginTransitionError> {
        let _mutation = self
            .lifecycle_mutation
            .try_lock()
            .map_err(|_| EditorPluginTransitionError::MutationInProgress)?;
        let previous = self.state_snapshot();
        let index = previous
            .entries()
            .binary_search_by(|entry| entry.package_id.as_str().cmp(package_id))
            .map_err(|_| EditorPluginTransitionError::UnknownPlugin {
                package_id: package_id.to_string(),
            })?;
        let entry = &previous.entries()[index];
        let from = entry.state;
        if next_state == EditorPluginState::Validated
            && has_failed_disabled_lifecycle(previous.catalog_snapshot(), entry)
        {
            return Err(
                EditorPluginTransitionError::DisabledLifecycleRetryRequired {
                    package_id: package_id.to_string(),
                },
            );
        }
        if is_phase_gated_state(next_state)
            && !phase_is_reached(entry.loading_phase, previous.reached_loading_phase)
        {
            return Err(EditorPluginTransitionError::LoadingPhaseUnavailable {
                package_id: package_id.to_string(),
                loading_phase: entry.loading_phase,
                reached: previous.reached_loading_phase,
            });
        }
        if matches!(
            next_state,
            EditorPluginState::Loading
                | EditorPluginState::Active
                | EditorPluginState::Revoking
                | EditorPluginState::Disabled
        ) {
            return Err(
                EditorPluginTransitionError::ManagedLifecycleTransitionRequired {
                    package_id: package_id.to_string(),
                    requested: next_state,
                },
            );
        }
        if !from.can_transition_to(next_state) {
            return Err(EditorPluginTransitionError::InvalidTransition {
                package_id: package_id.to_string(),
                from,
                to: next_state,
            });
        }

        let mut entries = previous.entries().to_vec();
        entries[index].state = next_state;
        Ok(self.publish_manager_snapshot(&previous, None, entries, previous.reached_loading_phase))
    }

    /// Advances startup scheduling once and atomically publishes the matching active extensions.
    pub fn advance_loading_phase(
        &self,
        requested: EditorPluginLoadingPhase,
    ) -> Result<Arc<EditorPluginManagerSnapshot>, EditorPluginTransitionError> {
        let _mutation = self
            .lifecycle_mutation
            .try_lock()
            .map_err(|_| EditorPluginTransitionError::MutationInProgress)?;
        let previous = self.state_snapshot();
        if let Some(reached) = previous.reached_loading_phase {
            if requested < reached {
                return Err(EditorPluginTransitionError::InvalidLoadingPhaseAdvance {
                    reached,
                    requested,
                });
            }
            if requested == reached {
                return Ok(previous);
            }
        }

        let mut entries = previous.entries().to_vec();
        normalize_entries_for_loading_phase(&mut entries, Some(requested));
        let mut catalog = previous.catalog_snapshot().clone_catalog();
        let catalog_changed =
            activate_eligible_entries(&mut catalog, &mut entries, Some(requested));
        Ok(self.publish_manager_snapshot(
            &previous,
            catalog_changed.then_some(catalog),
            entries,
            Some(requested),
        ))
    }

    /// Checks whether an enablement request can publish without replacing the current snapshot.
    pub fn validate_enablement(
        &self,
        package_id: &str,
        enabled: bool,
    ) -> Result<(), EditorPluginTransitionError> {
        let snapshot = self
            .snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = snapshot.entry(package_id).ok_or_else(|| {
            EditorPluginTransitionError::UnknownPlugin {
                package_id: package_id.to_string(),
            }
        })?;
        if has_failed_disabled_lifecycle(snapshot.catalog_snapshot(), entry) {
            return if enabled {
                Err(
                    EditorPluginTransitionError::DisabledLifecycleRetryRequired {
                        package_id: package_id.to_string(),
                    },
                )
            } else {
                Ok(())
            };
        }
        let _ = state_after_enablement_request(
            package_id,
            entry.state,
            enabled,
            entry.loading_phase,
            snapshot.reached_loading_phase,
        )?;
        Ok(())
    }

    /// Changes the desired activation state without rebuilding descriptor data.
    pub fn set_enabled(
        &self,
        package_id: &str,
        enabled: bool,
    ) -> Result<Arc<EditorPluginManagerSnapshot>, EditorPluginTransitionError> {
        let _mutation = self
            .lifecycle_mutation
            .try_lock()
            .map_err(|_| EditorPluginTransitionError::MutationInProgress)?;
        let previous = self.state_snapshot();
        let index = previous
            .entries()
            .binary_search_by(|entry| entry.package_id.as_str().cmp(package_id))
            .map_err(|_| EditorPluginTransitionError::UnknownPlugin {
                package_id: package_id.to_string(),
            })?;
        let mut entries = previous.entries().to_vec();
        let mut catalog = previous.catalog_snapshot().clone_catalog();
        let previous_state = entries[index].state;
        let catalog_changed = apply_enablement_request(
            &mut catalog,
            &mut entries[index],
            enabled,
            previous.reached_loading_phase,
            has_failed_disabled_lifecycle(previous.catalog_snapshot(), &previous.entries()[index]),
        )?;
        if entries[index].state == previous_state && !catalog_changed {
            return Ok(previous);
        }
        Ok(self.publish_manager_snapshot(
            &previous,
            catalog_changed.then_some(catalog),
            entries,
            previous.reached_loading_phase,
        ))
    }

    /// Applies the project selection contract to every editor package in one publication batch.
    ///
    /// The manifest is only desired enablement input. The catalog remains the sole descriptor
    /// owner, and callers retain the previous snapshot if the request changes no editor state.
    pub fn apply_project_manifest(
        &self,
        manifest: &ProjectPluginManifest,
    ) -> Result<Arc<EditorPluginManagerSnapshot>, EditorPluginTransitionError> {
        project_selection::apply_project_manifest(self, manifest)
    }

    /// Dispatches a non-activation lifecycle event through the manager-owned plugin handle.
    pub fn dispatch_lifecycle_event(
        &self,
        package_id: &str,
        event: EditorPluginLifecycleEvent,
    ) -> Result<EditorPluginLifecycleReport, EditorPluginTransitionError> {
        if is_manager_owned_activation_stage(event.stage()) {
            return Err(EditorPluginTransitionError::ManagedLifecycleEventReserved {
                package_id: package_id.to_string(),
                stage: event.stage().clone(),
            });
        }
        let _mutation = self
            .lifecycle_mutation
            .try_lock()
            .map_err(|_| EditorPluginTransitionError::MutationInProgress)?;
        let previous = self.state_snapshot();
        let index = previous
            .entries()
            .binary_search_by(|entry| entry.package_id.as_str().cmp(package_id))
            .map_err(|_| EditorPluginTransitionError::UnknownPlugin {
                package_id: package_id.to_string(),
            })?;
        let mut entries = previous.entries().to_vec();
        let mut catalog = previous.catalog_snapshot().clone_catalog();
        let report = catalog.record_lifecycle_event(package_id, event);
        if !report.is_success() {
            entries[index].state = EditorPluginState::Faulted;
        }
        self.publish_manager_snapshot(
            &previous,
            Some(catalog),
            entries,
            previous.reached_loading_phase,
        );
        Ok(report)
    }

    /// Delivers one external lifecycle event to the current active set in one publication batch.
    ///
    /// Activation stages remain manager-owned; every other stage observes the same immutable
    /// active set, so a failing callback cannot prevent later active plugins from receiving it.
    pub fn dispatch_lifecycle_event_to_active(
        &self,
        event: EditorPluginLifecycleEvent,
    ) -> Result<EditorPluginLifecycleReport, EditorPluginTransitionError> {
        if is_manager_owned_activation_stage(event.stage()) {
            return Err(
                EditorPluginTransitionError::ManagedLifecycleBroadcastReserved {
                    stage: event.stage().clone(),
                },
            );
        }
        let _mutation = self
            .lifecycle_mutation
            .try_lock()
            .map_err(|_| EditorPluginTransitionError::MutationInProgress)?;
        let previous = self.state_snapshot();
        if !previous
            .entries()
            .iter()
            .any(|entry| entry.state == EditorPluginState::Active)
        {
            return Ok(EditorPluginLifecycleReport::default());
        }

        let mut entries = previous.entries().to_vec();
        let mut catalog = previous.catalog_snapshot().clone_catalog();
        let mut report = EditorPluginLifecycleReport::default();
        for entry in entries
            .iter_mut()
            .filter(|entry| entry.state == EditorPluginState::Active)
        {
            let package_id = entry.package_id.clone();
            let package_report = catalog.record_lifecycle_event(&package_id, event.clone());
            if !package_report.is_success() {
                entry.state = EditorPluginState::Faulted;
            }
            report.extend(package_report);
        }
        self.publish_manager_snapshot(
            &previous,
            Some(catalog),
            entries,
            previous.reached_loading_phase,
        );
        Ok(report)
    }

    fn publish_manager_snapshot(
        &self,
        previous: &EditorPluginManagerSnapshot,
        catalog: Option<EditorPluginCatalog>,
        entries: Vec<EditorPluginManagerEntry>,
        reached_loading_phase: Option<EditorPluginLoadingPhase>,
    ) -> Arc<EditorPluginManagerSnapshot> {
        let catalog = match catalog {
            Some(catalog) => {
                let generation = self.catalog_store.next_generation();
                let catalog = Arc::new(EditorPluginCatalogSnapshot::from_catalog(
                    generation, catalog,
                ));
                self.catalog_store.publish_prepared(catalog)
            }
            None => Arc::clone(previous.catalog_snapshot()),
        };
        let snapshot = Arc::new(EditorPluginManagerSnapshot::from_parts(
            previous.generation().saturating_add(1),
            catalog,
            entries,
            reached_loading_phase,
        ));
        *self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Arc::clone(&snapshot);
        snapshot
    }
}
#[cfg(test)]
mod tests;
