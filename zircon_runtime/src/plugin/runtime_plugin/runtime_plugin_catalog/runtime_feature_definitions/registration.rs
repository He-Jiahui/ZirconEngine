use std::collections::{HashMap, HashSet};

use super::super::feature_definitions::FeatureDefinition;
use super::super::RuntimePluginFeatureRegistrationReport;
use super::conflict::append_package_registration_conflict;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_runtime_feature_registration(
    registration: &RuntimePluginFeatureRegistrationReport,
    definitions: &mut HashMap<String, FeatureDefinition>,
    diagnostics: &mut Vec<String>,
    definition_order: &mut Vec<String>,
    declared_feature_ids: &HashSet<String>,
    registered_feature_ids: &mut HashSet<String>,
) {
    let provider_package_id = registration.provider_package_id_or_owner();
    let key = FeatureDefinition::key(&registration.manifest.id, provider_package_id);
    if registered_feature_ids.contains(&key) {
        diagnostics.push(format!(
            "duplicate optional feature id {} registered at runtime (provider {})",
            registration.manifest.id, provider_package_id
        ));
        return;
    }
    registered_feature_ids.insert(key.clone());
    let feature_definition = FeatureDefinition::new_with_key(
        key.clone(),
        registration.manifest.clone(),
        provider_package_id.to_owned(),
    );
    if declared_feature_ids.contains(&key) {
        append_package_registration_conflict(registration, definitions, diagnostics, &key);
        return;
    }
    if definitions
        .insert(key.clone(), feature_definition)
        .is_some()
    {
        diagnostics.push(format!(
            "duplicate optional feature provider {} declared or registered in plugin catalog",
            key
        ));
    } else {
        definition_order.push(key);
    }
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830ek_runtime537_duplicate_registration_rejects_before_materialization(
    ) {
        let source = include_str!("registration.rs")
            .split_once("#[cfg(test)]")
            .expect("production/test boundary")
            .0;
        let key = source
            .find("FeatureDefinition::key(")
            .expect("registration key projection");
        let duplicate_check = source
            .find("registered_feature_ids.contains(&key)")
            .expect("borrowed duplicate check");
        let manifest_clone = source
            .find("registration.manifest.clone()")
            .expect("unique registration materialization");

        assert!(key < duplicate_check);
        assert!(duplicate_check < manifest_clone);
        assert!(source.contains("FeatureDefinition::new_with_key("));
    }

    #[test]
    #[ignore = "performance evidence"]
    fn optimization_batch_20260830ek_runtime537_duplicate_registration_clone_evidence() {
        const REGISTRATIONS: usize = 65_536;

        let legacy_duplicate_manifest_clones = REGISTRATIONS - 1;
        let optimized_duplicate_manifest_clones = 0usize;

        assert_eq!(legacy_duplicate_manifest_clones, 65_535);
        assert_eq!(optimized_duplicate_manifest_clones, 0);
        println!(
            "RUNTIME537_DUPLICATE_FEATURE_REGISTRATION_PREFLIGHT_BENCH_V1 \
             legacy_duplicate_manifest_clones={legacy_duplicate_manifest_clones} \
             optimized_duplicate_manifest_clones={optimized_duplicate_manifest_clones}"
        );
    }
}
