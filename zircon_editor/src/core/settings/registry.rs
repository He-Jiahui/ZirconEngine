use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use super::change_log::SettingsChangeLog;
use super::snapshot::BuiltInSettingsSlots;
use super::{
    SettingChange, SettingDefinition, SettingValue, SettingValueSource, SettingsChangeCursor,
    SettingsChangeDelta, SettingsChangeLogPolicy, SettingsKey, SettingsScope,
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
    pub(super) built_in_slots: BuiltInSettingsSlots,
    layers: SettingsLayers,
    pub(super) revision: u64,
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

    pub(super) fn definitions(&self) -> impl ExactSizeIterator<Item = &SettingDefinition> {
        self.definitions.values()
    }

    pub fn resolve(&self, key: &SettingsKey) -> Result<&SettingValue, SettingsError> {
        self.resolve_with_source(key).map(|(value, _)| value)
    }

    pub(super) fn resolve_with_source(
        &self,
        key: &SettingsKey,
    ) -> Result<(&SettingValue, SettingValueSource), SettingsError> {
        let definition = self.definition_or_error(key)?;
        for scope in [
            SettingsScope::Session,
            SettingsScope::Project,
            SettingsScope::User,
        ] {
            if let Some(value) = self.layers.get(scope, key) {
                return Ok((value, SettingValueSource::Scope(scope)));
            }
        }
        Ok((&definition.default, SettingValueSource::Default))
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

        let previous = self.persistent_values(scope)?;
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
