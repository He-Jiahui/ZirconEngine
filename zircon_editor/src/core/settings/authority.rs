use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use arc_swap::ArcSwap;

use crate::core::editor_operation::EditorOperationPath;

use super::io::PreparedSettingsWrite;
use super::{
    ResolvedSettingSnapshot, ResolvedSettingsBatch, SettingChange, SettingValue,
    SettingsChangeCursor, SettingsChangeDelta, SettingsError, SettingsKey, SettingsLoad,
    SettingsRegistry, SettingsScope, SettingsSnapshot, SettingsStore, SettingsStoreError,
    SettingsUserLayerLoad,
};

/// Consumes published settings changes after the authority has released its mutable state lock.
///
/// Subscribers must treat the snapshot as the source of truth and must not synchronously write
/// back into the authority from this callback.
pub(crate) trait SettingsChangeSubscriber: Send + Sync {
    fn settings_changed(&self, changes: &[SettingChange], snapshot: &SettingsSnapshot);
}

struct SettingsAuthorityState {
    registry: SettingsRegistry,
}

/// The durable project-layer result for the active editor project generation.
///
/// The authority retains this result instead of asking each UI consumer to reopen the same
/// settings file. A missing or invalid source is cached just like a successful load so the
/// generation has one unambiguous startup provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsProjectLayerLoad {
    Persisted { path: PathBuf, schema_version: u32 },
    Missing { path: PathBuf },
    Invalid { path: PathBuf, message: String },
}

#[derive(Clone, Debug)]
struct CachedProjectLayerLoad {
    path: PathBuf,
    result: SettingsProjectLayerLoad,
}

#[derive(Default)]
struct ProjectLayerState {
    cached: Option<CachedProjectLayerLoad>,
    transition_in_progress: bool,
}

/// The sole mutable owner for registered settings and their published generation snapshots.
pub struct SettingsAuthority {
    state: Mutex<SettingsAuthorityState>,
    snapshot: ArcSwap<SettingsSnapshot>,
    user_layer_load: Option<SettingsUserLayerLoad>,
    change_subscriber: Mutex<Option<Arc<dyn SettingsChangeSubscriber>>>,
    project_layer: Mutex<ProjectLayerState>,
    project_layer_operation: Mutex<()>,
}

impl SettingsAuthority {
    pub fn with_defaults() -> Self {
        Self::from_registry(super::settings_registry_with_defaults())
    }

    fn from_registry(registry: SettingsRegistry) -> Self {
        Self::from_registry_and_user_layer(registry, None)
    }

    pub(super) fn from_startup(
        registry: SettingsRegistry,
        user_layer_load: SettingsUserLayerLoad,
    ) -> Self {
        Self::from_registry_and_user_layer(registry, Some(user_layer_load))
    }

    fn from_registry_and_user_layer(
        registry: SettingsRegistry,
        user_layer_load: Option<SettingsUserLayerLoad>,
    ) -> Self {
        let snapshot = Arc::new(SettingsSnapshot::from_registry(&registry));
        Self {
            state: Mutex::new(SettingsAuthorityState { registry }),
            snapshot: ArcSwap::from(snapshot),
            user_layer_load,
            change_subscriber: Mutex::new(None),
            project_layer: Mutex::new(ProjectLayerState::default()),
            project_layer_operation: Mutex::new(()),
        }
    }

    pub fn snapshot(&self) -> Arc<SettingsSnapshot> {
        self.snapshot.load_full()
    }

    pub fn user_layer_load(&self) -> Option<&SettingsUserLayerLoad> {
        self.user_layer_load.as_ref()
    }

    /// Reads one effective setting without cloning the registry or its immutable catalog.
    pub fn resolved_setting(
        &self,
        key: &SettingsKey,
    ) -> Result<ResolvedSettingSnapshot, SettingsError> {
        let state = self.lock_state();
        let (value, source) = state.registry.resolve_with_source(key)?;
        Ok(ResolvedSettingSnapshot::new(
            state.registry.revision,
            value.clone(),
            source,
        ))
    }

    /// Reads a selected set of effective values under one lock and one exact generation.
    pub fn resolved_settings(
        &self,
        keys: &[SettingsKey],
    ) -> Result<ResolvedSettingsBatch, SettingsError> {
        let state = self.lock_state();
        ResolvedSettingsBatch::from_registry(&state.registry, keys)
    }

    /// Configures the one contextual hot-apply subscriber for this authority.
    ///
    /// The editor installs it before exposing the context, after it has synchronized the startup
    /// snapshot, so every later mutation observes a fully initialized service graph.
    pub(crate) fn configure_change_subscriber(
        &self,
        subscriber: Arc<dyn SettingsChangeSubscriber>,
    ) {
        *self.lock_change_subscriber() = Some(subscriber);
    }

    /// Records palette usage through the registered Session setting without making UI callers
    /// reconstruct the built-in key or a second MRU value outside this authority.
    pub fn record_command_palette_usage(
        &self,
        command: EditorOperationPath,
    ) -> Result<Option<SettingChange>, SettingsError> {
        let mut state = self.lock_state();
        let key = state.registry.built_in_slots.command_palette_mru().clone();
        let mut mru = match state.registry.resolve(&key)? {
            SettingValue::CommandPaletteMru(mru) => mru.clone(),
            _ => unreachable!("the command-palette MRU key has its registered schema"),
        };
        if !mru.record(command) {
            return Ok(None);
        }
        let change = state
            .registry
            .set(
                SettingsScope::Session,
                &key,
                SettingValue::CommandPaletteMru(mru),
            )?
            .expect("a changed command-palette MRU must update its Session layer");
        let previous = self.snapshot.load_full();
        let snapshot = SettingsSnapshot::after_change(&previous, &state.registry, &change);
        let snapshot = Arc::new(snapshot);
        self.snapshot.store(Arc::clone(&snapshot));
        drop(state);
        self.notify_change_subscriber(std::slice::from_ref(&change), snapshot.as_ref());
        Ok(Some(change))
    }

    pub fn changes_since(&self, cursor: SettingsChangeCursor) -> SettingsChangeDelta {
        let mut state = self.lock_state();
        state.registry.changes_since(cursor)
    }

    /// Applies a value only when it changes the scoped layer, then publishes one new snapshot.
    pub fn set(
        &self,
        scope: SettingsScope,
        key: &SettingsKey,
        value: SettingValue,
    ) -> Result<Option<SettingChange>, SettingsError> {
        let mut state = self.lock_state();
        let Some(change) = state.registry.set(scope, key, value)? else {
            return Ok(None);
        };
        let previous = self.snapshot.load_full();
        let snapshot = SettingsSnapshot::after_change(&previous, &state.registry, &change);
        let snapshot = Arc::new(snapshot);
        self.snapshot.store(Arc::clone(&snapshot));
        drop(state);
        self.notify_change_subscriber(std::slice::from_ref(&change), snapshot.as_ref());
        Ok(Some(change))
    }

    pub fn clear(
        &self,
        scope: SettingsScope,
        key: &SettingsKey,
    ) -> Result<Option<SettingChange>, SettingsError> {
        let mut state = self.lock_state();
        let change = state.registry.clear(scope, key)?;
        if let Some(change) = change {
            let previous = self.snapshot.load_full();
            let snapshot = SettingsSnapshot::after_change(&previous, &state.registry, &change);
            let snapshot = Arc::new(snapshot);
            self.snapshot.store(Arc::clone(&snapshot));
            drop(state);
            self.notify_change_subscriber(std::slice::from_ref(&change), snapshot.as_ref());
            return Ok(Some(change));
        }
        Ok(None)
    }

    pub(crate) fn replace_persistent_layer(
        &self,
        scope: SettingsScope,
        values: BTreeMap<SettingsKey, SettingValue>,
    ) -> Result<Vec<SettingChange>, SettingsError> {
        let mut state = self.lock_state();
        let changes = state.registry.replace_persistent_layer(scope, values)?;
        if changes.is_empty() {
            return Ok(changes);
        }
        let mut snapshot = self.snapshot.load_full().as_ref().clone();
        for change in &changes {
            snapshot = SettingsSnapshot::after_change(&snapshot, &state.registry, change);
        }
        let snapshot = Arc::new(snapshot);
        self.snapshot.store(Arc::clone(&snapshot));
        drop(state);
        self.notify_change_subscriber(&changes, snapshot.as_ref());
        Ok(changes)
    }

    /// Serializes the current durable layer only while its project source is still active.
    ///
    /// The worker receives a complete encoded document but never a cloned registry map. Project
    /// source identity is held through serialization, so a queued write for one project cannot
    /// capture another project's authority layer after a switch.
    pub(crate) fn prepare_persistent_layer_for_write(
        &self,
        scope: SettingsScope,
        store: &SettingsStore,
    ) -> Result<Option<PreparedSettingsWrite>, SettingsStoreError> {
        let project_layer = (scope == SettingsScope::Project).then(|| self.lock_project_layer());
        if let Some(project_layer) = project_layer.as_ref() {
            if project_layer.transition_in_progress {
                return Ok(None);
            }
            let Some(expected_path) = store.paths().project() else {
                return Err(SettingsStoreError::ProjectRootRequired);
            };
            let Some(active) = project_layer
                .cached
                .as_ref()
                .filter(|active| active.path.as_path() == expected_path)
            else {
                return Ok(None);
            };
            if matches!(&active.result, SettingsProjectLayerLoad::Invalid { .. }) {
                return Ok(None);
            }
        }

        let state = self.lock_state();
        store
            .prepare_registry_layer(scope, &state.registry)
            .map(Some)
    }

    /// Loads the active project layer exactly once for its settings-file path.
    ///
    /// This is intentionally the only project-settings read path used by production editor
    /// startup. Viewport consumers may bind the same store later, but receive this cached result
    /// rather than opening the source again.
    pub(crate) fn load_project_layer_from_environment(
        &self,
        project_root: &Path,
    ) -> SettingsProjectLayerLoad {
        let fallback_store = super::SettingsPaths::from_roots(project_root, Some(project_root));
        let path = fallback_store
            .project()
            .expect("a project settings path is available for an active project")
            .to_path_buf();
        let user_root = match super::SettingsPaths::user_root_from_environment() {
            Ok(user_root) => user_root,
            Err(error) => {
                let _operation = self.lock_project_layer_operation();
                if let Some(previous) = self.cached_project_layer(&path) {
                    return previous.result.clone();
                }
                self.begin_project_layer_transition();
                self.replace_persistent_layer(SettingsScope::Project, BTreeMap::new())
                    .expect("the registered project layer accepts an empty replacement");
                let result = SettingsProjectLayerLoad::Invalid {
                    path: path.clone(),
                    message: error.to_string(),
                };
                self.finish_project_layer_transition(Some(CachedProjectLayerLoad {
                    path,
                    result: result.clone(),
                }));
                return result;
            }
        };
        let store = SettingsStore::from_roots(user_root, Some(project_root));
        self.load_project_layer_from_store(&store)
    }

    pub(crate) fn load_project_layer_from_store(
        &self,
        store: &SettingsStore,
    ) -> SettingsProjectLayerLoad {
        let path = store
            .paths()
            .project()
            .expect("project-layer loading requires a project settings store")
            .to_path_buf();
        let _operation = self.lock_project_layer_operation();
        if let Some(previous) = self.cached_project_layer(&path) {
            return previous.result.clone();
        }
        self.begin_project_layer_transition();

        self.replace_persistent_layer(SettingsScope::Project, BTreeMap::new())
            .expect("the registered project layer accepts an empty replacement");
        let result = match store.load_authority_layer(SettingsScope::Project, self) {
            Ok(SettingsLoad::Loaded {
                path,
                schema_version,
                ..
            }) => SettingsProjectLayerLoad::Persisted {
                path,
                schema_version,
            },
            Ok(SettingsLoad::Missing { path }) => SettingsProjectLayerLoad::Missing { path },
            Err(error) => SettingsProjectLayerLoad::Invalid {
                path: path.clone(),
                message: error.to_string(),
            },
        };
        self.finish_project_layer_transition(Some(CachedProjectLayerLoad {
            path,
            result: result.clone(),
        }));
        result
    }

    pub(crate) fn clear_project_layer(&self) {
        let _operation = self.lock_project_layer_operation();
        self.begin_project_layer_transition();
        self.replace_persistent_layer(SettingsScope::Project, BTreeMap::new())
            .expect("the registered project layer accepts an empty replacement");
        self.finish_project_layer_transition(None);
    }

    fn lock_state(&self) -> MutexGuard<'_, SettingsAuthorityState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn notify_change_subscriber(&self, changes: &[SettingChange], snapshot: &SettingsSnapshot) {
        let subscriber = self.lock_change_subscriber().clone();
        if let Some(subscriber) = subscriber {
            subscriber.settings_changed(changes, snapshot);
        }
    }

    fn lock_change_subscriber(&self) -> MutexGuard<'_, Option<Arc<dyn SettingsChangeSubscriber>>> {
        self.change_subscriber
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn cached_project_layer(&self, path: &Path) -> Option<CachedProjectLayerLoad> {
        self.lock_project_layer()
            .cached
            .as_ref()
            .filter(|previous| previous.path.as_path() == path)
            .cloned()
    }

    fn begin_project_layer_transition(&self) {
        let mut project_layer = self.lock_project_layer();
        project_layer.cached = None;
        project_layer.transition_in_progress = true;
    }

    fn finish_project_layer_transition(&self, cached: Option<CachedProjectLayerLoad>) {
        let mut project_layer = self.lock_project_layer();
        project_layer.cached = cached;
        project_layer.transition_in_progress = false;
    }

    fn lock_project_layer(&self) -> MutexGuard<'_, ProjectLayerState> {
        self.project_layer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_project_layer_operation(&self) -> MutexGuard<'_, ()> {
        self.project_layer_operation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
