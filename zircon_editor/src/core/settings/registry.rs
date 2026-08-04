use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use arc_swap::ArcSwap;
use thiserror::Error;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use crate::core::editor_operation::EditorOperationPath;

use super::io::PreparedSettingsWrite;
use super::{
    EDITOR_COMMAND_PALETTE_MRU_KEY, EDITOR_DESIGN_TOKENS_KEY, EDITOR_KEYMAP_OVERRIDES_KEY,
    EDITOR_LOCALE_KEY, EditorCommandPaletteMru, EditorKeymapOverrides, SettingChange,
    SettingDefinition, SettingValue, SettingsChangeCursor, SettingsChangeDelta, SettingsChangeLog,
    SettingsChangeLogPolicy, SettingsKey, SettingsLoad, SettingsPaths, SettingsScope,
    SettingsStore, SettingsStoreError, VIEWPORT_ROTATE_STEP_DEGREES_KEY, VIEWPORT_SCALE_STEP_KEY,
    VIEWPORT_TRANSLATE_STEP_KEY,
};

#[derive(Debug, Error, PartialEq)]
pub enum SettingsError {
    #[error("setting `{0}` is not registered")]
    UnknownKey(String),
    #[error("setting `{0}` is registered more than once")]
    DuplicateDefinition(String),
    #[error("setting `{key}` has an invalid definition: {reason}")]
    InvalidDefinition { key: String, reason: String },
    #[error(
        "setting `{key}` cannot be written at {requested:?}; its definition permits {defined:?}"
    )]
    ScopeNotAllowed {
        key: String,
        requested: SettingsScope,
        defined: SettingsScope,
    },
    #[error("setting `{key}` has an invalid value: {reason}")]
    InvalidValue { key: String, reason: String },
    #[error("{0:?} settings are session-only and cannot be persisted")]
    NonPersistentScope(SettingsScope),
}

#[derive(Clone, Default)]
struct SettingsLayers {
    user: BTreeMap<SettingsKey, SettingValue>,
    project: BTreeMap<SettingsKey, SettingValue>,
    session: BTreeMap<SettingsKey, SettingValue>,
}

impl SettingsLayers {
    fn get(&self, scope: SettingsScope, key: &SettingsKey) -> Option<&SettingValue> {
        match scope {
            SettingsScope::User => self.user.get(key),
            SettingsScope::Project => self.project.get(key),
            SettingsScope::Session => self.session.get(key),
        }
    }

    fn get_mut(&mut self, scope: SettingsScope) -> &mut BTreeMap<SettingsKey, SettingValue> {
        match scope {
            SettingsScope::User => &mut self.user,
            SettingsScope::Project => &mut self.project,
            SettingsScope::Session => &mut self.session,
        }
    }
}

/// Owns setting definitions and the three precedence layers without performing I/O.
#[derive(Clone, Default)]
pub struct SettingsRegistry {
    definitions: BTreeMap<SettingsKey, SettingDefinition>,
    built_in_slots: BuiltInSettingsSlots,
    layers: SettingsLayers,
    revision: u64,
    changes: SettingsChangeLog,
}

impl SettingsRegistry {
    pub fn with_change_log_policy(policy: SettingsChangeLogPolicy) -> Self {
        Self {
            changes: SettingsChangeLog::with_policy(policy),
            ..Self::default()
        }
    }

    pub fn register(&mut self, definition: SettingDefinition) -> Result<(), SettingsError> {
        let key = definition.key.clone();
        if self.definitions.contains_key(&key) {
            return Err(SettingsError::DuplicateDefinition(key.as_str().to_string()));
        }
        definition
            .validate()
            .map_err(|reason| SettingsError::InvalidDefinition {
                key: key.as_str().to_string(),
                reason,
            })?;
        self.built_in_slots.record(&key);
        self.definitions.insert(key, definition);
        Ok(())
    }

    pub fn definition(&self, key: &SettingsKey) -> Option<&SettingDefinition> {
        self.definitions.get(key)
    }

    pub fn resolve(&self, key: &SettingsKey) -> Result<&SettingValue, SettingsError> {
        let definition = self.definition_or_error(key)?;
        for scope in [
            SettingsScope::Session,
            SettingsScope::Project,
            SettingsScope::User,
        ] {
            if let Some(value) = self.layers.get(scope, key) {
                return Ok(value);
            }
        }
        Ok(&definition.default)
    }

    pub fn set(
        &mut self,
        scope: SettingsScope,
        key: &SettingsKey,
        value: SettingValue,
    ) -> Result<Option<SettingChange>, SettingsError> {
        let (defined_scope, schema, requires_restart) = {
            let definition = self.definition_or_error(key)?;
            (
                definition.scope,
                definition.schema.clone(),
                definition.requires_restart,
            )
        };
        if !defined_scope.allows_write(scope) {
            return Err(SettingsError::ScopeNotAllowed {
                key: key.as_str().to_string(),
                requested: scope,
                defined: defined_scope,
            });
        }
        schema
            .validate(&value)
            .map_err(|reason| SettingsError::InvalidValue {
                key: key.as_str().to_string(),
                reason,
            })?;
        if self.layers.get(scope, key) == Some(&value) {
            return Ok(None);
        }
        self.layers.get_mut(scope).insert(key.clone(), value);
        self.revision = self.revision.saturating_add(1);
        let change = SettingChange {
            key: key.clone(),
            scope,
            revision: self.revision,
            requires_restart,
        };
        self.changes.record(change.clone());
        Ok(Some(change))
    }

    pub fn clear(
        &mut self,
        scope: SettingsScope,
        key: &SettingsKey,
    ) -> Result<Option<SettingChange>, SettingsError> {
        let (defined_scope, requires_restart) = {
            let definition = self.definition_or_error(key)?;
            (definition.scope, definition.requires_restart)
        };
        if !defined_scope.allows_write(scope) {
            return Err(SettingsError::ScopeNotAllowed {
                key: key.as_str().to_string(),
                requested: scope,
                defined: defined_scope,
            });
        }
        if self.layers.get_mut(scope).remove(key).is_none() {
            return Ok(None);
        }
        self.revision = self.revision.saturating_add(1);
        let change = SettingChange {
            key: key.clone(),
            scope,
            revision: self.revision,
            requires_restart,
        };
        self.changes.record(change.clone());
        Ok(Some(change))
    }

    pub fn change_cursor(&self) -> SettingsChangeCursor {
        SettingsChangeCursor::at(self.revision)
    }

    pub fn changes_since(&mut self, cursor: SettingsChangeCursor) -> SettingsChangeDelta {
        self.changes.delta_since(cursor, self.revision)
    }

    pub(crate) fn persistent_values(
        &self,
        scope: SettingsScope,
    ) -> Result<&BTreeMap<SettingsKey, SettingValue>, SettingsError> {
        match scope {
            SettingsScope::User => Ok(&self.layers.user),
            SettingsScope::Project => Ok(&self.layers.project),
            SettingsScope::Session => Err(SettingsError::NonPersistentScope(scope)),
        }
    }

    /// Replaces one durable layer only after every persisted entry has passed the
    /// registered key, scope, and value checks.
    /// Atomically replaces a durable layer and publishes only when its effective values changed.
    pub(crate) fn replace_persistent_layer(
        &mut self,
        scope: SettingsScope,
        values: BTreeMap<SettingsKey, SettingValue>,
    ) -> Result<Vec<SettingChange>, SettingsError> {
        if !scope.is_persistent() {
            return Err(SettingsError::NonPersistentScope(scope));
        }
        for (key, value) in &values {
            let definition = self.definition_or_error(key)?;
            if !definition.scope.allows_write(scope) {
                return Err(SettingsError::ScopeNotAllowed {
                    key: key.as_str().to_string(),
                    requested: scope,
                    defined: definition.scope,
                });
            }
            definition
                .schema
                .validate(value)
                .map_err(|reason| SettingsError::InvalidValue {
                    key: key.as_str().to_string(),
                    reason,
                })?;
        }

        let previous = self.persistent_values(scope)?.clone();
        let changed_keys: BTreeSet<_> = previous
            .keys()
            .chain(values.keys())
            .filter(|key| previous.get(*key) != values.get(*key))
            .cloned()
            .collect();
        if changed_keys.is_empty() {
            return Ok(Vec::new());
        }

        *self.layers.get_mut(scope) = values;
        let mut changes = Vec::with_capacity(changed_keys.len());
        for key in changed_keys {
            let requires_restart = self
                .definition_or_error(&key)
                .expect("validated persisted keys remain registered")
                .requires_restart;
            self.revision = self.revision.saturating_add(1);
            let change = SettingChange {
                key,
                scope,
                revision: self.revision,
                requires_restart,
            };
            self.changes.record(change.clone());
            changes.push(change);
        }
        Ok(changes)
    }

    fn definition_or_error(&self, key: &SettingsKey) -> Result<&SettingDefinition, SettingsError> {
        self.definition(key)
            .ok_or_else(|| SettingsError::UnknownKey(key.as_str().to_string()))
    }
}

/// Parsed built-in keys retained by the registry at registration time.
///
/// Snapshot publication runs on every real settings mutation. Keeping the core keys here
/// prevents that hot path from allocating and reparsing static strings before looking up the
/// effective value.
#[derive(Clone, Default)]
struct BuiltInSettingsSlots {
    design_tokens: Option<SettingsKey>,
    keymap_overrides: Option<SettingsKey>,
    command_palette_mru: Option<SettingsKey>,
    locale: Option<SettingsKey>,
    viewport_translate_step: Option<SettingsKey>,
    viewport_rotate_step_degrees: Option<SettingsKey>,
    viewport_scale_step: Option<SettingsKey>,
}

impl BuiltInSettingsSlots {
    fn record(&mut self, key: &SettingsKey) {
        let slot = match key.as_str() {
            EDITOR_DESIGN_TOKENS_KEY => &mut self.design_tokens,
            EDITOR_KEYMAP_OVERRIDES_KEY => &mut self.keymap_overrides,
            EDITOR_COMMAND_PALETTE_MRU_KEY => &mut self.command_palette_mru,
            EDITOR_LOCALE_KEY => &mut self.locale,
            VIEWPORT_TRANSLATE_STEP_KEY => &mut self.viewport_translate_step,
            VIEWPORT_ROTATE_STEP_DEGREES_KEY => &mut self.viewport_rotate_step_degrees,
            VIEWPORT_SCALE_STEP_KEY => &mut self.viewport_scale_step,
            _ => return,
        };
        *slot = Some(key.clone());
    }

    fn design_tokens(&self) -> &SettingsKey {
        self.design_tokens
            .as_ref()
            .expect("the settings authority requires the design-token slot")
    }

    fn keymap_overrides(&self) -> &SettingsKey {
        self.keymap_overrides
            .as_ref()
            .expect("the settings authority requires the keymap-overrides slot")
    }

    fn command_palette_mru(&self) -> &SettingsKey {
        self.command_palette_mru
            .as_ref()
            .expect("the settings authority requires the command-palette MRU slot")
    }

    fn locale(&self) -> &SettingsKey {
        self.locale
            .as_ref()
            .expect("the settings authority requires the locale slot")
    }

    fn viewport_translate_step(&self) -> &SettingsKey {
        self.viewport_translate_step
            .as_ref()
            .expect("the settings authority requires the viewport translate-step slot")
    }

    fn viewport_rotate_step_degrees(&self) -> &SettingsKey {
        self.viewport_rotate_step_degrees
            .as_ref()
            .expect("the settings authority requires the viewport rotate-step slot")
    }

    fn viewport_scale_step(&self) -> &SettingsKey {
        self.viewport_scale_step
            .as_ref()
            .expect("the settings authority requires the viewport scale-step slot")
    }
}

/// Typed viewport snap values resolved once for an immutable settings generation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportSnapSettings {
    translate_step: f64,
    rotate_step_degrees: f64,
    scale_step: f64,
}

impl ViewportSnapSettings {
    pub const fn translate_step(self) -> f64 {
        self.translate_step
    }

    pub const fn rotate_step_degrees(self) -> f64 {
        self.rotate_step_degrees
    }

    pub const fn scale_step(self) -> f64 {
        self.scale_step
    }
}

/// Immutable, typed values published by the sole settings authority for one generation.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingsSnapshot {
    generation: u64,
    design_tokens: Arc<EditorDesignTokens>,
    keymap_overrides: Arc<EditorKeymapOverrides>,
    command_palette_mru: Arc<EditorCommandPaletteMru>,
    locale: Arc<str>,
    viewport_snap: ViewportSnapSettings,
}

/// Consumes published settings changes after the authority has released its mutable state lock.
///
/// Subscribers must treat the snapshot as the source of truth and must not synchronously write
/// back into the authority from this callback.
pub(crate) trait SettingsChangeSubscriber: Send + Sync {
    fn settings_changed(&self, changes: &[SettingChange], snapshot: &SettingsSnapshot);
}

impl SettingsSnapshot {
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn design_tokens(&self) -> &EditorDesignTokens {
        self.design_tokens.as_ref()
    }

    /// Shares the immutable token payload with UI projections that only refresh on token changes.
    pub(crate) fn design_tokens_handle(&self) -> Arc<EditorDesignTokens> {
        Arc::clone(&self.design_tokens)
    }

    pub fn keymap_overrides(&self) -> &EditorKeymapOverrides {
        self.keymap_overrides.as_ref()
    }

    /// Shares the immutable override payload with projections that must avoid rebuilding on
    /// unrelated settings generations.
    pub(crate) fn keymap_overrides_handle(&self) -> Arc<EditorKeymapOverrides> {
        Arc::clone(&self.keymap_overrides)
    }

    pub fn command_palette_mru(&self) -> &EditorCommandPaletteMru {
        self.command_palette_mru.as_ref()
    }

    /// The resolved User language setting for consumers that render localizable editor content.
    pub fn locale(&self) -> &str {
        self.locale.as_ref()
    }

    pub const fn viewport_snap(&self) -> ViewportSnapSettings {
        self.viewport_snap
    }

    fn from_registry(registry: &SettingsRegistry) -> Self {
        let slots = &registry.built_in_slots;
        Self {
            generation: registry.revision,
            design_tokens: Arc::new(built_in_design_tokens(registry, slots.design_tokens())),
            keymap_overrides: Arc::new(built_in_keymap_overrides(
                registry,
                slots.keymap_overrides(),
            )),
            command_palette_mru: Arc::new(built_in_command_palette_mru(
                registry,
                slots.command_palette_mru(),
            )),
            locale: Arc::from(built_in_locale(registry, slots.locale())),
            viewport_snap: ViewportSnapSettings {
                translate_step: built_in_float(registry, slots.viewport_translate_step()),
                rotate_step_degrees: built_in_float(registry, slots.viewport_rotate_step_degrees()),
                scale_step: built_in_float(registry, slots.viewport_scale_step()),
            },
        }
    }

    fn after_change(
        previous: &SettingsSnapshot,
        registry: &SettingsRegistry,
        change: &SettingChange,
    ) -> Self {
        let slots = &registry.built_in_slots;
        let mut snapshot = Self {
            generation: registry.revision,
            design_tokens: Arc::clone(&previous.design_tokens),
            keymap_overrides: Arc::clone(&previous.keymap_overrides),
            command_palette_mru: Arc::clone(&previous.command_palette_mru),
            locale: Arc::clone(&previous.locale),
            viewport_snap: previous.viewport_snap,
        };
        if &change.key == slots.design_tokens() {
            snapshot.design_tokens =
                Arc::new(built_in_design_tokens(registry, slots.design_tokens()));
        } else if &change.key == slots.keymap_overrides() {
            snapshot.keymap_overrides = Arc::new(built_in_keymap_overrides(
                registry,
                slots.keymap_overrides(),
            ));
        } else if &change.key == slots.command_palette_mru() {
            snapshot.command_palette_mru = Arc::new(built_in_command_palette_mru(
                registry,
                slots.command_palette_mru(),
            ));
        } else if &change.key == slots.locale() {
            snapshot.locale = Arc::from(built_in_locale(registry, slots.locale()));
        } else if &change.key == slots.viewport_translate_step() {
            snapshot.viewport_snap.translate_step =
                built_in_float(registry, slots.viewport_translate_step());
        } else if &change.key == slots.viewport_rotate_step_degrees() {
            snapshot.viewport_snap.rotate_step_degrees =
                built_in_float(registry, slots.viewport_rotate_step_degrees());
        } else if &change.key == slots.viewport_scale_step() {
            snapshot.viewport_snap.scale_step =
                built_in_float(registry, slots.viewport_scale_step());
        }
        snapshot
    }
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

/// The sole mutable owner for registered settings and their published generation snapshots.
pub struct SettingsAuthority {
    state: Mutex<SettingsAuthorityState>,
    snapshot: ArcSwap<SettingsSnapshot>,
    change_subscriber: Mutex<Option<Arc<dyn SettingsChangeSubscriber>>>,
    project_layer: Mutex<Option<CachedProjectLayerLoad>>,
}

impl SettingsAuthority {
    pub fn with_defaults() -> Self {
        Self::from_registry(super::settings_registry_with_defaults())
    }

    pub fn at_startup() -> Self {
        let authority = Self::from_registry(super::settings_registry_with_defaults());
        authority.load_user_layer_at_startup();
        authority
    }

    fn from_registry(registry: SettingsRegistry) -> Self {
        let snapshot = Arc::new(SettingsSnapshot::from_registry(&registry));
        Self {
            state: Mutex::new(SettingsAuthorityState { registry }),
            snapshot: ArcSwap::from(snapshot),
            change_subscriber: Mutex::new(None),
            project_layer: Mutex::new(None),
        }
    }

    pub fn snapshot(&self) -> Arc<SettingsSnapshot> {
        self.snapshot.load_full()
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
            let Some(expected_path) = store.paths().project() else {
                return Err(SettingsStoreError::ProjectRootRequired);
            };
            let Some(active) = project_layer
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
        let fallback_store = SettingsPaths::from_roots(project_root, Some(project_root));
        let path = fallback_store
            .project()
            .expect("a project settings path is available for an active project")
            .to_path_buf();
        let user_root = match SettingsPaths::user_root_from_environment() {
            Ok(user_root) => user_root,
            Err(error) => {
                let mut cached = self.lock_project_layer();
                if let Some(previous) = cached.as_ref().filter(|previous| previous.path == path) {
                    return previous.result.clone();
                }
                self.replace_persistent_layer(SettingsScope::Project, BTreeMap::new())
                    .expect("the registered project layer accepts an empty replacement");
                let result = SettingsProjectLayerLoad::Invalid {
                    path: path.clone(),
                    message: error.to_string(),
                };
                *cached = Some(CachedProjectLayerLoad {
                    path,
                    result: result.clone(),
                });
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
        let mut cached = self.lock_project_layer();
        if let Some(previous) = cached.as_ref().filter(|previous| previous.path == path) {
            return previous.result.clone();
        }

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
        *cached = Some(CachedProjectLayerLoad {
            path,
            result: result.clone(),
        });
        result
    }

    pub(crate) fn clear_project_layer(&self) {
        let mut cached = self.lock_project_layer();
        self.replace_persistent_layer(SettingsScope::Project, BTreeMap::new())
            .expect("the registered project layer accepts an empty replacement");
        *cached = None;
    }

    fn load_user_layer_at_startup(&self) {
        match SettingsStore::from_user_environment() {
            Ok(store) => {
                if let Err(error) = store.load_authority_layer(SettingsScope::User, self) {
                    tracing::warn!(error = %error, "failed to load editor user settings; using defaults");
                }
            }
            Err(error) => {
                tracing::warn!(error = %error, "failed to resolve editor user settings root; using defaults");
            }
        }
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

    fn lock_project_layer(&self) -> MutexGuard<'_, Option<CachedProjectLayerLoad>> {
        self.project_layer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn built_in_design_tokens(registry: &SettingsRegistry, key: &SettingsKey) -> EditorDesignTokens {
    match registry
        .resolve(key)
        .expect("the built-in design-token setting is registered")
    {
        SettingValue::DesignTokens(tokens) => tokens.clone(),
        _ => unreachable!("the built-in design-token setting has a design-token schema"),
    }
}

fn built_in_keymap_overrides(
    registry: &SettingsRegistry,
    key: &SettingsKey,
) -> EditorKeymapOverrides {
    match registry
        .resolve(key)
        .expect("the built-in keymap-overrides setting is registered")
    {
        SettingValue::KeymapOverrides(overrides) => overrides.clone(),
        _ => unreachable!("the built-in keymap-overrides setting has a keymap-overrides schema"),
    }
}

fn built_in_command_palette_mru(
    registry: &SettingsRegistry,
    key: &SettingsKey,
) -> EditorCommandPaletteMru {
    match registry
        .resolve(key)
        .expect("the built-in command-palette MRU setting is registered")
    {
        SettingValue::CommandPaletteMru(mru) => mru.clone(),
        _ => unreachable!("the built-in command-palette MRU setting has an MRU schema"),
    }
}

fn built_in_float(registry: &SettingsRegistry, key: &SettingsKey) -> f64 {
    match registry
        .resolve(key)
        .expect("the built-in viewport snap key is registered")
    {
        SettingValue::Float(value) => *value,
        _ => unreachable!("the built-in viewport snap key uses a float schema"),
    }
}

fn built_in_locale(registry: &SettingsRegistry, key: &SettingsKey) -> String {
    match registry
        .resolve(key)
        .expect("the built-in locale setting is registered")
    {
        SettingValue::Enum(value) => value.clone(),
        _ => unreachable!("the built-in locale setting uses an enum schema"),
    }
}
