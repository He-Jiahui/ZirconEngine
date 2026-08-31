use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::asset::{PrefabInstanceAsset, PrefabPropertyOverrideAsset};

pub fn effective_prefab_overrides(
    instance: &PrefabInstanceAsset,
) -> Vec<PrefabPropertyOverrideAsset> {
    let mut overrides = BTreeMap::new();
    for override_value in &instance.overrides {
        overrides.insert(
            (
                override_value.entity_path.as_str(),
                override_value.property_path.as_str(),
            ),
            override_value,
        );
    }
    overrides.into_values().cloned().collect()
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
