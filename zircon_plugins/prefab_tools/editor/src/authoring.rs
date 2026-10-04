use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use zircon_runtime::asset::{PrefabInstanceAsset, PrefabPropertyOverrideAsset};

enum EffectiveOverrideBucket<'a> {
    One(&'a PrefabPropertyOverrideAsset),
    Many(Vec<&'a PrefabPropertyOverrideAsset>),
}

pub fn effective_prefab_overrides(
    instance: &PrefabInstanceAsset,
) -> Vec<PrefabPropertyOverrideAsset> {
    // Keep every value for a path instead of letting `BTreeMap::insert` silently
    // discard an earlier conflict. Validation rejects duplicate paths before a
    // mutation is admitted, while this read/query surface remains lossless for
    // diagnostics and deterministic replay.
    let mut overrides: BTreeMap<(&str, &str), EffectiveOverrideBucket<'_>> = BTreeMap::new();
    for override_value in &instance.overrides {
        let key = (
            override_value.entity_path.as_str(),
            override_value.property_path.as_str(),
        );
        match overrides.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(EffectiveOverrideBucket::One(override_value));
            }
            Entry::Occupied(mut entry) => {
                let bucket = entry.get_mut();
                let previous =
                    std::mem::replace(bucket, EffectiveOverrideBucket::One(override_value));
                *bucket = match previous {
                    EffectiveOverrideBucket::One(first) => {
                        EffectiveOverrideBucket::Many(vec![first, override_value])
                    }
                    EffectiveOverrideBucket::Many(mut values) => {
                        values.push(override_value);
                        EffectiveOverrideBucket::Many(values)
                    }
                };
            }
        }
    }
    let mut effective = Vec::with_capacity(instance.overrides.len());
    for bucket in overrides.into_values() {
        match bucket {
            EffectiveOverrideBucket::One(value) => effective.push(value.clone()),
            EffectiveOverrideBucket::Many(values) => {
                effective.extend(values.into_iter().map(Clone::clone));
            }
        }
    }
    effective
}

pub fn validate_prefab_instance(
    instance: &PrefabInstanceAsset,
    source_prefab_available: bool,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut override_paths = BTreeSet::new();
    if !source_prefab_available {
        diagnostics.push(format!(
            "prefab instance source `{}` is not available",
            instance.prefab
        ));
    }
    for override_value in &instance.overrides {
        if override_value.entity_path.trim().is_empty() {
            diagnostics.push("prefab override entity path must not be empty".to_string());
        }
        if override_value.property_path.trim().is_empty() {
            diagnostics.push("prefab override property path must not be empty".to_string());
        }
        if !override_paths.insert((
            override_value.entity_path.as_str(),
            override_value.property_path.as_str(),
        )) {
            diagnostics.push(format!(
                "duplicate prefab override `{}` / `{}` is ambiguous",
                override_value.entity_path, override_value.property_path
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}
