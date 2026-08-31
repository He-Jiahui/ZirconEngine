use std::collections::HashSet;

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::{RuntimeExtensionRegistryError, UiComponentDescriptor};

use super::is_lowercase_plugin_package_id;

pub(in crate::plugin::extension_registry) fn validate_component_type_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_component_type_field("type_id", &descriptor.type_id)?;
    validate_component_type_field("plugin_id", &descriptor.plugin_id)?;
    validate_component_type_plugin_id(&descriptor.plugin_id)?;
    validate_component_type_field("display_name", &descriptor.display_name)?;
    let expected_prefix = format!("{}.", descriptor.plugin_id);
    if !descriptor.type_id.starts_with(&expected_prefix) {
        return Err(RuntimeExtensionRegistryError::InvalidComponentType(
            format!(
                "component type {} must be prefixed by plugin id {}",
                descriptor.type_id, descriptor.plugin_id
            ),
        ));
    }

    let mut property_names = HashSet::new();
    for property in &descriptor.properties {
        validate_component_type_field("property name", &property.name)?;
        validate_component_type_field("property value_type", &property.value_type)?;
        if !property_names.insert(property.name.as_str()) {
            return Err(RuntimeExtensionRegistryError::InvalidComponentType(
                format!(
                    "component type {} property `{}` must be unique",
                    descriptor.type_id, property.name
                ),
            ));
        }
    }
    Ok(())
}

pub(in crate::plugin::extension_registry) fn validate_ui_component_descriptor(
    descriptor: &UiComponentDescriptor,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_ui_component_field("component_id", &descriptor.component_id)?;
    validate_ui_component_field("plugin_id", &descriptor.plugin_id)?;
    validate_ui_component_plugin_id(&descriptor.plugin_id)?;
    validate_ui_component_field("ui_document", &descriptor.ui_document)?;
    let expected_prefix = format!("{}.", descriptor.plugin_id);
    if !descriptor.component_id.starts_with(&expected_prefix) {
        return Err(RuntimeExtensionRegistryError::InvalidUiComponent(format!(
            "ui component {} must be prefixed by plugin id {}",
            descriptor.component_id, descriptor.plugin_id
        )));
    }
    if !descriptor.ui_document.ends_with(".zui") {
        return Err(RuntimeExtensionRegistryError::InvalidUiComponent(format!(
            "ui component {} document `{}` must reference a .zui component asset",
            descriptor.component_id, descriptor.ui_document
        )));
    }
    Ok(())
}

fn validate_component_type_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(RuntimeExtensionRegistryError::InvalidComponentType(
            format!("{field_name} `{value}` must be non-empty and trimmed"),
        ));
    }
    Ok(())
}

fn validate_component_type_plugin_id(plugin_id: &str) -> Result<(), RuntimeExtensionRegistryError> {
    if !is_lowercase_plugin_package_id(plugin_id) {
        return Err(RuntimeExtensionRegistryError::InvalidComponentType(
            format!(
                "plugin_id `{plugin_id}` must start with a lowercase ASCII letter and contain only lowercase ASCII letters, digits, underscores, and dots in non-empty segments without trailing or repeated underscores"
            ),
        ));
    }
    Ok(())
}

fn validate_ui_component_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(RuntimeExtensionRegistryError::InvalidUiComponent(format!(
            "{field_name} `{value}` must be non-empty and trimmed"
        )));
    }
    Ok(())
}

fn validate_ui_component_plugin_id(plugin_id: &str) -> Result<(), RuntimeExtensionRegistryError> {
    if !is_lowercase_plugin_package_id(plugin_id) {
        return Err(RuntimeExtensionRegistryError::InvalidUiComponent(format!(
            "plugin_id `{plugin_id}` must start with a lowercase ASCII letter and contain only lowercase ASCII letters, digits, underscores, and dots in non-empty segments without trailing or repeated underscores"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::core::framework::scene::ComponentPropertyDescriptor;

    use super::*;

    const PROPERTY_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_PROPERTY_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        let rank = samples.len().saturating_mul(95).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    #[test]
    fn runtime42_hash_batch_component_p95_uses_nearest_rank() {
        let mut samples = (1..=17).map(Duration::from_nanos).collect::<Vec<_>>();

        assert_eq!(percentile_95(&mut samples), Duration::from_nanos(17));
    }

    fn property_names() -> Vec<String> {
        (0..PROPERTY_ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "generated_component_property_with_long_identity_{:05}",
                    (index * 4_099) % UNIQUE_PROPERTY_COUNT
                )
            })
            .collect()
    }

    fn ordered_unique_count(property_names: &[String]) -> usize {
        let mut unique = BTreeSet::new();
        property_names
            .iter()
            .filter(|name| unique.insert(name.as_str()))
            .count()
    }

    fn hash_unique_count(property_names: &[String]) -> usize {
        let mut unique = HashSet::new();
        property_names
            .iter()
            .filter(|name| unique.insert(name.as_str()))
            .count()
    }

    #[test]
    fn runtime42_hash_batch_component_preserves_first_duplicate_error() {
        let descriptor = ComponentTypeDescriptor {
            type_id: "sample.component".to_string(),
            plugin_id: "sample".to_string(),
            display_name: "Sample Component".to_string(),
            properties: vec![
                ComponentPropertyDescriptor {
                    name: "enabled".to_string(),
                    value_type: "bool".to_string(),
                    editable: true,
                },
                ComponentPropertyDescriptor {
                    name: "weight".to_string(),
                    value_type: "f32".to_string(),
                    editable: true,
                },
                ComponentPropertyDescriptor {
                    name: "enabled".to_string(),
                    value_type: "bool".to_string(),
                    editable: false,
                },
            ],
        };

        let error = validate_component_type_descriptor(&descriptor).unwrap_err();
        assert!(format!("{error:?}").contains("property `enabled` must be unique"));
    }

    #[test]
    fn runtime42_hash_batch_component_uses_borrowed_hash_set() {
        let source = include_str!("component.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("let mut property_names = HashSet::new();"));
        assert!(production.contains("property_names.insert(property.name.as_str())"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn runtime42_hash_batch_component_property_performance_evidence() {
        let property_names = property_names();
        assert_eq!(
            ordered_unique_count(&property_names),
            hash_unique_count(&property_names)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&property_names)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_unique_count(black_box(&property_names)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_unique_count(black_box(&property_names)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&property_names)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "RUNTIME42_COMPONENT_PROPERTY_HASH_VALIDATION_BENCH_V1 \
             admissions={PROPERTY_ADMISSION_COUNT} unique_properties={UNIQUE_PROPERTY_COUNT} \
             borrowed_identity=true ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-validation P95 {:?} exceeded 60% of ordered-validation P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
