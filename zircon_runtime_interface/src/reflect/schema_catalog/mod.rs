use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::reflect::{ReflectError, ReflectFieldId, ReflectTypeRegistration};

mod admission;
mod entry;
mod field_index;
mod fingerprint;

use admission::{
    admit_entries, build_short_path_index, canonicalize_and_validate_entry, dependency_order,
    MAX_REFLECT_SCHEMA_DEPENDENCIES, MAX_REFLECT_SCHEMA_TYPES,
};
use field_index::ReflectSchemaFieldIndex;
use fingerprint::fingerprint;

pub use entry::ReflectSchemaCatalogEntry;
pub use fingerprint::{ReflectSchemaFingerprint, REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION};

/// Admitted neutral reflection catalog shared by runtime, tooling, editor, and script hosts.
#[derive(Clone, Debug)]
pub struct ReflectSchemaCatalog {
    entries: BTreeMap<String, ReflectSchemaCatalogEntry>,
    short_paths: BTreeMap<String, String>,
    ambiguous_short_paths: BTreeSet<String>,
    field_indexes: BTreeMap<String, ReflectSchemaFieldIndex>,
    field_id_owners: HashMap<ReflectFieldId, (String, String)>,
    dependency_edge_count: usize,
    dependency_order: OnceLock<Vec<String>>,
    fingerprint: OnceLock<ReflectSchemaFingerprint>,
}

/// Serializable immutable projection of one admitted catalog generation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectSchemaCatalogSnapshot {
    pub algorithm_version: u32,
    pub fingerprint: ReflectSchemaFingerprint,
    pub entries: Vec<ReflectSchemaCatalogEntry>,
    pub ambiguous_short_type_paths: Vec<String>,
    pub dependency_order: Vec<String>,
}

impl ReflectSchemaCatalog {
    pub fn try_new(entries: Vec<ReflectSchemaCatalogEntry>) -> Result<Self, ReflectError> {
        let admitted = admit_entries(entries)?;
        Ok(Self {
            entries: admitted.entries,
            short_paths: admitted.short_paths,
            ambiguous_short_paths: admitted.ambiguous_short_paths,
            field_indexes: admitted.field_indexes,
            field_id_owners: admitted.field_id_owners,
            dependency_edge_count: admitted.dependency_edge_count,
            dependency_order: OnceLock::new(),
            fingerprint: OnceLock::new(),
        })
    }

    pub fn try_from_snapshot(snapshot: ReflectSchemaCatalogSnapshot) -> Result<Self, ReflectError> {
        if snapshot.algorithm_version != REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION {
            return Err(ReflectError::InvalidRegistration {
                type_path: "<schema-catalog>".to_string(),
                reason: format!(
                    "unsupported reflection schema catalog algorithm version {}; expected {}",
                    snapshot.algorithm_version, REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION
                ),
            });
        }
        let expected_fingerprint = snapshot.fingerprint;
        let catalog = Self::try_new(snapshot.entries)?;
        if catalog.fingerprint() != expected_fingerprint {
            return Err(ReflectError::InvalidRegistration {
                type_path: "<schema-catalog>".to_string(),
                reason: format!(
                    "reflection schema catalog fingerprint mismatch: expected {expected_fingerprint}, computed {}",
                    catalog.fingerprint()
                ),
            });
        }
        if snapshot.ambiguous_short_type_paths
            != catalog
                .ambiguous_short_paths
                .iter()
                .cloned()
                .collect::<Vec<_>>()
        {
            return Err(ReflectError::InvalidRegistration {
                type_path: "<schema-catalog>".to_string(),
                reason: "reflection schema catalog ambiguous short-path projection is stale"
                    .to_string(),
            });
        }
        if !snapshot
            .dependency_order
            .iter()
            .map(String::as_str)
            .eq(catalog.dependency_order())
        {
            return Err(ReflectError::InvalidRegistration {
                type_path: "<schema-catalog>".to_string(),
                reason: "reflection schema catalog dependency order projection is stale"
                    .to_string(),
            });
        }
        Ok(catalog)
    }

    pub fn fingerprint(&self) -> ReflectSchemaFingerprint {
        *self.fingerprint.get_or_init(|| fingerprint(&self.entries))
    }

    pub fn snapshot(&self) -> ReflectSchemaCatalogSnapshot {
        ReflectSchemaCatalogSnapshot {
            algorithm_version: REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION,
            fingerprint: self.fingerprint(),
            entries: self.entries.values().cloned().collect(),
            ambiguous_short_type_paths: self.ambiguous_short_paths.iter().cloned().collect(),
            dependency_order: self.dependency_order().map(str::to_string).collect(),
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = &ReflectSchemaCatalogEntry> {
        self.entries.values()
    }

    pub fn registrations(&self) -> impl Iterator<Item = &ReflectTypeRegistration> {
        self.entries.values().map(|entry| &entry.registration)
    }

    pub fn registration(&self, type_path: &str) -> Result<&ReflectTypeRegistration, ReflectError> {
        let type_path = self.resolve_type_path(type_path)?;
        Ok(&self
            .entries
            .get(type_path)
            .expect("resolved schema type path must have an entry")
            .registration)
    }

    pub fn resolve_type_path(&self, type_path: &str) -> Result<&str, ReflectError> {
        if let Some((canonical, _)) = self.entries.get_key_value(type_path) {
            return Ok(canonical.as_str());
        }
        if let Some(canonical) = self.short_paths.get(type_path) {
            return Ok(canonical.as_str());
        }
        if self.ambiguous_short_paths.contains(type_path) {
            return Err(ReflectError::AmbiguousShortTypePath {
                short_type_path: type_path.to_string(),
            });
        }
        Err(ReflectError::UnknownType {
            type_path: type_path.to_string(),
        })
    }

    pub fn field_slot_by_id(
        &self,
        type_path: &str,
        field_id: ReflectFieldId,
    ) -> Result<u32, ReflectError> {
        let type_path = self.resolve_type_path(type_path)?;
        self.field_indexes[type_path]
            .field_slot(field_id)
            .ok_or_else(|| ReflectError::UnknownField {
                type_path: type_path.to_string(),
                field_name: field_id.to_string(),
            })
    }

    /// Resolves current or historical field names for one explicitly scoped legacy import.
    pub fn resolve_legacy_field_id(
        &self,
        type_path: &str,
        field_name: &str,
    ) -> Result<ReflectFieldId, ReflectError> {
        let type_path = self.resolve_type_path(type_path)?;
        self.field_indexes[type_path]
            .legacy_field_id(field_name)
            .ok_or_else(|| ReflectError::UnknownField {
                type_path: type_path.to_string(),
                field_name: field_name.to_string(),
            })
    }

    pub fn dependency_order(&self) -> impl Iterator<Item = &str> {
        self.dependency_order
            .get_or_init(|| {
                dependency_order(&self.entries)
                    .expect("an admitted schema catalog has an acyclic dependency graph")
            })
            .iter()
            .map(String::as_str)
    }

    pub fn ambiguous_short_type_paths(&self) -> impl Iterator<Item = &str> {
        self.ambiguous_short_paths.iter().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn contains(&self, type_path: &str) -> bool {
        self.entries.contains_key(type_path) || self.short_paths.contains_key(type_path)
    }

    pub fn contains_type_path(&self, type_path: &str) -> bool {
        self.entries.contains_key(type_path)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn validate_entry(entry: &ReflectSchemaCatalogEntry) -> Result<(), ReflectError> {
        let mut entry = entry.clone();
        canonicalize_and_validate_entry(&mut entry)
    }

    pub fn validate_insert(&self, entry: &ReflectSchemaCatalogEntry) -> Result<(), ReflectError> {
        self.prepare_insert(entry.clone()).map(|_| ())
    }

    pub fn try_insert(&mut self, entry: ReflectSchemaCatalogEntry) -> Result<(), ReflectError> {
        let (type_path, entry, dependency_edge_count) = self.prepare_insert(entry)?;
        let short_path = entry.registration.type_path.short_type_path().to_string();
        for field in &entry.registration.type_info.fields {
            self.field_id_owners
                .insert(field.id, (type_path.clone(), field.name.clone()));
        }
        self.field_indexes.insert(
            type_path.clone(),
            ReflectSchemaFieldIndex::from_fields(&entry.registration.type_info.fields),
        );
        self.entries.insert(type_path.clone(), entry);
        self.dependency_edge_count = dependency_edge_count;
        self.update_short_path_index(&type_path, &short_path);
        self.invalidate_derived_products();
        Ok(())
    }

    fn prepare_insert(
        &self,
        mut entry: ReflectSchemaCatalogEntry,
    ) -> Result<(String, ReflectSchemaCatalogEntry, usize), ReflectError> {
        let type_path = entry.registration.type_path.type_path().to_string();
        if self.entries.contains_key(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }
        if self.entries.len() >= MAX_REFLECT_SCHEMA_TYPES {
            return Err(ReflectError::InvalidRegistration {
                type_path,
                reason: format!(
                    "reflection catalog must not contain more than {MAX_REFLECT_SCHEMA_TYPES} types"
                ),
            });
        }
        canonicalize_and_validate_entry(&mut entry)?;
        let dependency_edge_count = self
            .dependency_edge_count
            .checked_add(entry.dependencies.len())
            .ok_or_else(|| ReflectError::InvalidRegistration {
                type_path: type_path.clone(),
                reason: "schema dependency count overflowed".to_string(),
            })?;
        if dependency_edge_count > MAX_REFLECT_SCHEMA_DEPENDENCIES {
            return Err(ReflectError::InvalidRegistration {
                type_path: type_path.clone(),
                reason: format!(
                    "reflection catalog must not contain more than {MAX_REFLECT_SCHEMA_DEPENDENCIES} dependencies"
                ),
            });
        }
        for dependency in &entry.dependencies {
            if !self.entries.contains_key(dependency) {
                return Err(ReflectError::InvalidRegistration {
                    type_path: type_path.clone(),
                    reason: format!("missing schema dependency `{dependency}`"),
                });
            }
        }
        for field in &entry.registration.type_info.fields {
            if let Some((owner_type, owner_field)) = self.field_id_owners.get(&field.id) {
                return Err(ReflectError::InvalidFieldRegistration {
                    type_path: type_path.clone(),
                    field_name: field.name.clone(),
                    reason: format!(
                        "field ID `{}` is already owned by `{owner_type}.{owner_field}`",
                        field.id
                    ),
                });
            }
        }

        Ok((type_path, entry, dependency_edge_count))
    }

    pub fn try_replace(&mut self, entry: ReflectSchemaCatalogEntry) -> Result<(), ReflectError> {
        let type_path = entry.registration.type_path.type_path().to_string();
        if !self.entries.contains_key(&type_path) {
            return Err(ReflectError::UnknownType { type_path });
        }
        let entries = self
            .entries
            .values()
            .filter(|candidate| candidate.registration.type_path.type_path() != type_path.as_str())
            .cloned()
            .chain(std::iter::once(entry))
            .collect();
        *self = Self::try_new(entries)?;
        Ok(())
    }

    pub fn try_remove(
        &mut self,
        type_path: &str,
    ) -> Result<Option<ReflectSchemaCatalogEntry>, ReflectError> {
        let canonical = match self.resolve_type_path(type_path) {
            Ok(type_path) => type_path.to_string(),
            Err(ReflectError::UnknownType { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
        if let Some(dependent) = self.entries.values().find(|entry| {
            entry
                .dependencies
                .iter()
                .any(|dependency| dependency == &canonical)
        }) {
            return Err(ReflectError::InvalidRegistration {
                type_path: canonical,
                reason: format!(
                    "schema type is still required by `{}`",
                    dependent.registration.type_path.type_path()
                ),
            });
        }
        let removed = self
            .entries
            .remove(&canonical)
            .expect("resolved schema type path must have an entry");
        self.field_indexes.remove(&canonical);
        self.dependency_edge_count -= removed.dependencies.len();
        for field in &removed.registration.type_info.fields {
            self.field_id_owners.remove(&field.id);
        }
        (self.short_paths, self.ambiguous_short_paths) = build_short_path_index(&self.entries);
        self.invalidate_derived_products();
        Ok(Some(removed))
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.short_paths.clear();
        self.ambiguous_short_paths.clear();
        self.field_indexes.clear();
        self.field_id_owners.clear();
        self.dependency_edge_count = 0;
        self.invalidate_derived_products();
    }

    fn update_short_path_index(&mut self, type_path: &str, short_path: &str) {
        if self.ambiguous_short_paths.contains(short_path) {
            return;
        }
        match self.short_paths.get(short_path) {
            None => {
                self.short_paths
                    .insert(short_path.to_string(), type_path.to_string());
            }
            Some(existing) if existing == type_path => {}
            Some(_) => {
                self.short_paths.remove(short_path);
                self.ambiguous_short_paths.insert(short_path.to_string());
            }
        }
    }

    fn invalidate_derived_products(&mut self) {
        self.dependency_order = OnceLock::new();
        self.fingerprint = OnceLock::new();
    }
}

impl PartialEq for ReflectSchemaCatalog {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl Default for ReflectSchemaCatalog {
    fn default() -> Self {
        Self::try_new(Vec::new()).expect("an empty reflection schema catalog is valid")
    }
}

#[cfg(test)]
mod tests;
