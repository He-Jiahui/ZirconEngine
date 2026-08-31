use std::collections::HashMap;

use crate::reflect::{ReflectFieldId, ReflectFieldInfo};

const HASHED_FIELD_INDEX_THRESHOLD: usize = 512;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ReflectSchemaFieldIndex {
    stable: StableFieldIndex,
    legacy_names: HashMap<String, ReflectFieldId>,
}

#[derive(Clone, Debug, PartialEq)]
enum StableFieldIndex {
    Sorted(Vec<(ReflectFieldId, u32)>),
    Hashed(HashMap<ReflectFieldId, u32>),
}

impl ReflectSchemaFieldIndex {
    pub(super) fn from_fields(fields: &[ReflectFieldInfo]) -> Self {
        let stable = if fields.len() <= HASHED_FIELD_INDEX_THRESHOLD {
            let mut entries = fields
                .iter()
                .enumerate()
                .map(|(slot, field)| (field.id, slot as u32))
                .collect::<Vec<_>>();
            entries.sort_unstable_by_key(|(field_id, _)| *field_id);
            StableFieldIndex::Sorted(entries)
        } else {
            StableFieldIndex::Hashed(
                fields
                    .iter()
                    .enumerate()
                    .map(|(slot, field)| (field.id, slot as u32))
                    .collect(),
            )
        };
        let mut legacy_names = HashMap::with_capacity(
            fields.len()
                + fields
                    .iter()
                    .map(|field| field.aliases.len())
                    .sum::<usize>(),
        );
        for field in fields {
            legacy_names.insert(field.name.clone(), field.id);
            for alias in &field.aliases {
                legacy_names.insert(alias.clone(), field.id);
            }
        }
        Self {
            stable,
            legacy_names,
        }
    }

    pub(super) fn field_slot(&self, field_id: ReflectFieldId) -> Option<u32> {
        match &self.stable {
            StableFieldIndex::Sorted(entries) => entries
                .binary_search_by_key(&field_id, |(candidate, _)| *candidate)
                .ok()
                .map(|index| entries[index].1),
            StableFieldIndex::Hashed(entries) => entries.get(&field_id).copied(),
        }
    }

    pub(super) fn legacy_field_id(&self, field_name: &str) -> Option<ReflectFieldId> {
        self.legacy_names.get(field_name).copied()
    }
}
