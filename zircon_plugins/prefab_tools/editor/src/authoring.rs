use std::collections::BTreeMap;

use zircon_runtime::asset::{PrefabInstanceAsset, PrefabPropertyOverrideAsset, TransformAsset};

#[derive(Clone, Debug, PartialEq)]
pub struct BrokenPrefabInstanceAuthoringState {
    pub local_transform: TransformAsset,
    pub baked_overrides: Vec<PrefabPropertyOverrideAsset>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrefabOverrideApplication {
    pub applied_overrides: Vec<PrefabPropertyOverrideAsset>,
    pub cleared_instance_override_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefabOverrideRevertReport {
    pub reverted_override_count: usize,
}

pub fn effective_prefab_overrides(
    instance: &PrefabInstanceAsset,
) -> Vec<PrefabPropertyOverrideAsset> {
    let mut overrides = BTreeMap::new();
    for override_value in &instance.overrides {
        overrides.insert(
            (
                override_value.entity_path.clone(),
                override_value.property_path.clone(),
            ),
            override_value.clone(),
        );
    }
    overrides.into_values().collect()
}

pub fn validate_prefab_instance(
    instance: &PrefabInstanceAsset,
    source_prefab_available: bool,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
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
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn apply_prefab_overrides(
    instance: &mut PrefabInstanceAsset,
    source_prefab_available: bool,
) -> Result<PrefabOverrideApplication, Vec<String>> {
    let diagnostics = validate_prefab_instance(instance, source_prefab_available);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let cleared_instance_override_count = instance.overrides.len();
    let applied_overrides = effective_prefab_overrides(instance);
    instance.overrides.clear();
    Ok(PrefabOverrideApplication {
        applied_overrides,
        cleared_instance_override_count,
    })
}

pub fn revert_prefab_overrides(instance: &mut PrefabInstanceAsset) -> PrefabOverrideRevertReport {
    let reverted_override_count = instance.overrides.len();
    instance.overrides.clear();
    PrefabOverrideRevertReport {
        reverted_override_count,
    }
}

pub fn break_prefab_instance(instance: &PrefabInstanceAsset) -> BrokenPrefabInstanceAuthoringState {
    BrokenPrefabInstanceAuthoringState {
        local_transform: instance.local_transform.clone(),
        baked_overrides: effective_prefab_overrides(instance),
    }
}
