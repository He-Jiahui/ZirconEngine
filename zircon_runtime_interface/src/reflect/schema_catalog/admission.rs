use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::reflect::{ReflectError, ReflectFieldId, ReflectFieldInfo};

use super::{ReflectSchemaCatalogEntry, ReflectSchemaFieldIndex};

pub(super) const MAX_REFLECT_SCHEMA_TYPES: usize = 16_384;
const MAX_REFLECT_FIELDS_PER_TYPE: usize = 4_096;
const MAX_REFLECT_FIELD_NAME_BYTES: usize = 128;
const MAX_REFLECT_FIELD_ALIASES_PER_FIELD: usize = 32;
const MAX_REFLECT_FIELD_ALIASES_PER_TYPE: usize = 4_096;
const MAX_REFLECT_DEPENDENCIES_PER_TYPE: usize = 256;
pub(super) const MAX_REFLECT_SCHEMA_DEPENDENCIES: usize = 65_536;

pub(super) struct AdmittedCatalog {
    pub(super) entries: BTreeMap<String, ReflectSchemaCatalogEntry>,
    pub(super) short_paths: BTreeMap<String, String>,
    pub(super) ambiguous_short_paths: BTreeSet<String>,
    pub(super) field_indexes: BTreeMap<String, ReflectSchemaFieldIndex>,
    pub(super) field_id_owners: HashMap<ReflectFieldId, (String, String)>,
    pub(super) dependency_edge_count: usize,
}

pub(super) fn admit_entries(
    entries: Vec<ReflectSchemaCatalogEntry>,
) -> Result<AdmittedCatalog, ReflectError> {
    if entries.len() > MAX_REFLECT_SCHEMA_TYPES {
        return Err(ReflectError::InvalidRegistration {
            type_path: "<schema-catalog>".to_string(),
            reason: format!(
                "reflection catalog must not contain more than {MAX_REFLECT_SCHEMA_TYPES} types"
            ),
        });
    }

    let mut admitted = BTreeMap::new();
    let mut global_field_ids = HashMap::<ReflectFieldId, (String, String)>::new();
    let mut dependency_count = 0_usize;
    for mut entry in entries {
        let type_path = entry.registration.type_path.type_path().to_string();
        if admitted.contains_key(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }
        canonicalize_and_validate_entry(&mut entry)?;
        dependency_count = dependency_count
            .checked_add(entry.dependencies.len())
            .ok_or_else(|| {
                invalid_registration(&type_path, "schema dependency count overflowed")
            })?;
        if dependency_count > MAX_REFLECT_SCHEMA_DEPENDENCIES {
            return Err(invalid_registration(
                &type_path,
                &format!(
                    "reflection catalog must not contain more than {MAX_REFLECT_SCHEMA_DEPENDENCIES} dependencies"
                ),
            ));
        }
        for field in &entry.registration.type_info.fields {
            if let Some((owner_type, owner_field)) =
                global_field_ids.insert(field.id, (type_path.clone(), field.name.clone()))
            {
                return Err(invalid_field_registration(
                    &type_path,
                    &field.name,
                    &format!(
                        "field ID `{}` is already owned by `{owner_type}.{owner_field}`",
                        field.id
                    ),
                ));
            }
        }
        admitted.insert(type_path, entry);
    }

    let (short_paths, ambiguous_short_paths) = build_short_path_index(&admitted);
    dependency_order(&admitted)?;
    let field_indexes = admitted
        .iter()
        .map(|(type_path, entry)| {
            (
                type_path.clone(),
                ReflectSchemaFieldIndex::from_fields(&entry.registration.type_info.fields),
            )
        })
        .collect();
    Ok(AdmittedCatalog {
        entries: admitted,
        short_paths,
        ambiguous_short_paths,
        field_indexes,
        field_id_owners: global_field_ids,
        dependency_edge_count: dependency_count,
    })
}

pub(super) fn canonicalize_and_validate_entry(
    entry: &mut ReflectSchemaCatalogEntry,
) -> Result<(), ReflectError> {
    let type_path = entry.registration.type_path.type_path().to_string();
    if entry.registration.type_info.fields.len() > MAX_REFLECT_FIELDS_PER_TYPE {
        return Err(invalid_registration(
            &type_path,
            &format!("type must not declare more than {MAX_REFLECT_FIELDS_PER_TYPE} fields"),
        ));
    }
    validate_field_identities(&type_path, &mut entry.registration.type_info.fields)?;

    if entry.dependencies.len() > MAX_REFLECT_DEPENDENCIES_PER_TYPE {
        return Err(invalid_registration(
            &type_path,
            &format!(
                "type must not declare more than {MAX_REFLECT_DEPENDENCIES_PER_TYPE} schema dependencies"
            ),
        ));
    }
    for dependency in &entry.dependencies {
        super::super::type_path::validate_type_path(dependency)?;
        if dependency == &type_path {
            return Err(invalid_registration(
                &type_path,
                "schema types must not depend on themselves",
            ));
        }
    }
    entry.dependencies.sort_unstable();
    if let Some(duplicate) = entry
        .dependencies
        .windows(2)
        .find(|pair| pair[0] == pair[1])
        .map(|pair| pair[0].clone())
    {
        return Err(invalid_registration(
            &type_path,
            &format!("duplicate schema dependency `{duplicate}`"),
        ));
    }
    Ok(())
}

fn validate_field_identities(
    type_path: &str,
    fields: &mut [ReflectFieldInfo],
) -> Result<(), ReflectError> {
    let mut field_ids = HashSet::with_capacity(fields.len());
    let mut field_names = HashSet::with_capacity(fields.len());
    let mut alias_count = 0_usize;
    for field in fields.iter() {
        if !field_ids.insert(field.id) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!("duplicate reflected field ID `{}`", field.id),
            ));
        }
        validate_field_key(type_path, field, "name", &field.name)?;
        if !field_names.insert(field.name.as_str()) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!("duplicate reflected field `{}`", field.name),
            ));
        }
        if field.aliases.len() > MAX_REFLECT_FIELD_ALIASES_PER_FIELD {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!(
                    "field must not declare more than {MAX_REFLECT_FIELD_ALIASES_PER_FIELD} aliases"
                ),
            ));
        }
        alias_count = alias_count
            .checked_add(field.aliases.len())
            .ok_or_else(|| {
                invalid_field_registration(type_path, &field.name, "field alias count overflowed")
            })?;
        if alias_count > MAX_REFLECT_FIELD_ALIASES_PER_TYPE {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!(
                    "type must not declare more than {MAX_REFLECT_FIELD_ALIASES_PER_TYPE} field aliases"
                ),
            ));
        }
    }

    for field in fields.iter() {
        for alias in &field.aliases {
            validate_field_key(type_path, field, "alias", alias)?;
            if !field_names.insert(alias.as_str()) {
                return Err(invalid_field_registration(
                    type_path,
                    &field.name,
                    &format!("duplicate reflected field name or alias `{alias}`"),
                ));
            }
        }
    }
    drop(field_names);
    for field in fields {
        field.aliases.sort_unstable();
    }
    Ok(())
}

fn validate_field_key(
    type_path: &str,
    field: &ReflectFieldInfo,
    label: &str,
    value: &str,
) -> Result<(), ReflectError> {
    if value.is_empty() || value.trim() != value {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must be non-empty and already trimmed"),
        ));
    }
    if value.len() > MAX_REFLECT_FIELD_NAME_BYTES {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must not exceed {MAX_REFLECT_FIELD_NAME_BYTES} UTF-8 bytes"),
        ));
    }
    if !value
        .bytes()
        .all(|byte| byte == b'_' || byte == b'-' || byte.is_ascii_alphanumeric())
    {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must use ASCII letters, digits, `_`, or `-`"),
        ));
    }
    Ok(())
}

pub(super) fn build_short_path_index(
    entries: &BTreeMap<String, ReflectSchemaCatalogEntry>,
) -> (BTreeMap<String, String>, BTreeSet<String>) {
    let mut short_paths = BTreeMap::new();
    let mut ambiguous = BTreeSet::new();
    for (type_path, entry) in entries {
        let short = entry.registration.type_path.short_type_path();
        if ambiguous.contains(short) {
            continue;
        }
        match short_paths.get(short) {
            None => {
                short_paths.insert(short.to_string(), type_path.clone());
            }
            Some(existing) if existing == type_path => {}
            Some(_) => {
                short_paths.remove(short);
                ambiguous.insert(short.to_string());
            }
        }
    }
    (short_paths, ambiguous)
}

pub(super) fn dependency_order(
    entries: &BTreeMap<String, ReflectSchemaCatalogEntry>,
) -> Result<Vec<String>, ReflectError> {
    let mut remaining_dependencies = BTreeMap::<String, usize>::new();
    let mut dependents = BTreeMap::<String, Vec<String>>::new();
    for (type_path, entry) in entries {
        remaining_dependencies.insert(type_path.clone(), entry.dependencies.len());
        for dependency in &entry.dependencies {
            if !entries.contains_key(dependency) {
                return Err(invalid_registration(
                    type_path,
                    &format!("missing schema dependency `{dependency}`"),
                ));
            }
            dependents
                .entry(dependency.clone())
                .or_default()
                .push(type_path.clone());
        }
    }
    for consumers in dependents.values_mut() {
        consumers.sort_unstable();
    }

    let mut ready = remaining_dependencies
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(type_path, _)| type_path.clone())
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(entries.len());
    while let Some(type_path) = ready.pop_first() {
        order.push(type_path.clone());
        let Some(consumers) = dependents.get(&type_path) else {
            continue;
        };
        for consumer in consumers {
            let remaining = remaining_dependencies
                .get_mut(consumer)
                .expect("catalog dependency consumer must exist");
            *remaining -= 1;
            if *remaining == 0 {
                ready.insert(consumer.clone());
            }
        }
    }
    if order.len() == entries.len() {
        return Ok(order);
    }
    let cycle_owner = remaining_dependencies
        .into_iter()
        .find(|(_, count)| *count != 0)
        .map(|(type_path, _)| type_path)
        .unwrap_or_else(|| "<schema-catalog>".to_string());
    Err(invalid_registration(
        &cycle_owner,
        "schema dependency graph contains a cycle",
    ))
}

fn invalid_registration(type_path: &str, reason: &str) -> ReflectError {
    ReflectError::InvalidRegistration {
        type_path: type_path.to_string(),
        reason: reason.to_string(),
    }
}

fn invalid_field_registration(type_path: &str, field_name: &str, reason: &str) -> ReflectError {
    ReflectError::InvalidFieldRegistration {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        reason: reason.to_string(),
    }
}
