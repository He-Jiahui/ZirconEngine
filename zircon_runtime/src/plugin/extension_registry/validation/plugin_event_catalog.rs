use std::collections::HashSet;

use crate::plugin::{PluginEventCatalogManifest, RuntimeExtensionRegistryError};

pub(in crate::plugin::extension_registry) fn validate_plugin_event_catalog_manifest(
    descriptor: &PluginEventCatalogManifest,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_event_field("namespace", &descriptor.namespace)?;
    validate_dot_namespaced_event_id("event catalog", "namespace", &descriptor.namespace)?;
    if descriptor.version == 0 {
        return invalid_plugin_event_catalog(format!(
            "{} version must be a positive u32",
            descriptor.namespace
        ));
    }
    if descriptor.events.is_empty() {
        return invalid_plugin_event_catalog(format!(
            "{} must declare at least one event",
            descriptor.namespace
        ));
    }

    let package_namespace = descriptor.namespace.split('.').next().unwrap_or_default();
    let event_prefix = format!("{}.", descriptor.namespace);
    let payload_prefix = format!("{package_namespace}.");
    let mut event_ids = HashSet::new();
    for event in &descriptor.events {
        validate_plugin_event_field("id", &event.id)?;
        validate_dot_namespaced_event_id("event", "id", &event.id)?;
        if !event.id.starts_with(&event_prefix) {
            return invalid_plugin_event_catalog(format!(
                "event id `{}` must stay under catalog namespace `{}`",
                event.id, descriptor.namespace
            ));
        }
        if !event_ids.insert(event.id.as_str()) {
            return invalid_plugin_event_catalog(format!(
                "event id `{}` must be unique inside catalog `{}`",
                event.id, descriptor.namespace
            ));
        }
        validate_plugin_event_field("display_name", &event.display_name)?;

        if !event.payload_schema.is_empty() {
            validate_plugin_event_field("payload_schema", &event.payload_schema)?;
            validate_dot_namespaced_event_id("event", "payload_schema", &event.payload_schema)?;
            if !event.payload_schema.starts_with(&payload_prefix) {
                return invalid_plugin_event_catalog(format!(
                    "payload_schema `{}` must stay under package namespace `{payload_prefix}`",
                    event.payload_schema
                ));
            }
            validate_versioned_payload_schema(&event.payload_schema)?;
        }
    }

    Ok(())
}

fn validate_dot_namespaced_event_id(
    context: &str,
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if !value.contains('.') {
        return invalid_plugin_event_catalog(format!(
            "{context} {field_name} `{value}` must use at least two dot-separated namespace segments"
        ));
    }
    for segment in value.split('.') {
        if segment.is_empty()
            || !segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return invalid_plugin_event_catalog(format!(
                "{context} {field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, and dots"
            ));
        }
    }
    Ok(())
}

fn validate_versioned_payload_schema(
    payload_schema: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    let version_segment = payload_schema.rsplit('.').next().unwrap_or(payload_schema);
    let Some(version_number) = version_segment.strip_prefix('v') else {
        return invalid_plugin_event_catalog(format!(
            "payload_schema `{payload_schema}` must end with a version segment like `v1`"
        ));
    };
    if version_number.is_empty()
        || !version_number.bytes().all(|byte| byte.is_ascii_digit())
        || version_number.starts_with('0')
    {
        return invalid_plugin_event_catalog(format!(
            "payload_schema `{payload_schema}` version segment must be a positive integer without leading zeroes"
        ));
    }
    Ok(())
}

fn validate_plugin_event_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if value.trim().is_empty() || value.trim() != value {
        return invalid_plugin_event_catalog(format!(
            "{field_name} `{value}` must be non-empty and trimmed"
        ));
    }
    Ok(())
}

fn invalid_plugin_event_catalog<T>(message: String) -> Result<T, RuntimeExtensionRegistryError> {
    Err(RuntimeExtensionRegistryError::InvalidPluginEventCatalog(
        message,
    ))
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::plugin::PluginEventManifest;

    use super::*;

    const EVENT_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_EVENT_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        let rank = samples.len().saturating_mul(95).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    #[test]
    fn runtime42_hash_batch_plugin_event_p95_uses_nearest_rank() {
        let mut samples = (1..=17).map(Duration::from_nanos).collect::<Vec<_>>();

        assert_eq!(percentile_95(&mut samples), Duration::from_nanos(17));
    }

    fn event_ids() -> Vec<String> {
        (0..EVENT_ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "sample.events.generated_event_with_long_identity_{:05}",
                    (index * 4_099) % UNIQUE_EVENT_COUNT
                )
            })
            .collect()
    }

    fn ordered_unique_count(event_ids: &[String]) -> usize {
        let mut unique = BTreeSet::new();
        event_ids
            .iter()
            .filter(|event_id| unique.insert(event_id.as_str()))
            .count()
    }

    fn hash_unique_count(event_ids: &[String]) -> usize {
        let mut unique = HashSet::new();
        event_ids
            .iter()
            .filter(|event_id| unique.insert(event_id.as_str()))
            .count()
    }

    fn event(id: &str, display_name: &str) -> PluginEventManifest {
        PluginEventManifest {
            id: id.to_string(),
            display_name: display_name.to_string(),
            payload_schema: String::new(),
        }
    }

    #[test]
    fn runtime42_hash_batch_plugin_event_preserves_first_duplicate_error() {
        let descriptor = PluginEventCatalogManifest {
            namespace: "sample.events".to_string(),
            version: 1,
            events: vec![
                event("sample.events.open", "Open"),
                event("sample.events.close", "Close"),
                event("sample.events.open", "Duplicate Open"),
            ],
        };

        let error = validate_plugin_event_catalog_manifest(&descriptor).unwrap_err();
        assert!(format!("{error:?}").contains(
            "event id `sample.events.open` must be unique inside catalog `sample.events`"
        ));
    }

    #[test]
    fn runtime42_hash_batch_plugin_event_uses_borrowed_hash_set() {
        let source = include_str!("plugin_event_catalog.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("let mut event_ids = HashSet::new();"));
        assert!(production.contains("event_ids.insert(event.id.as_str())"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn runtime42_hash_batch_plugin_event_performance_evidence() {
        let event_ids = event_ids();
        assert_eq!(
            ordered_unique_count(&event_ids),
            hash_unique_count(&event_ids)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&event_ids)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_unique_count(black_box(&event_ids)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_unique_count(black_box(&event_ids)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&event_ids)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "RUNTIME42_PLUGIN_EVENT_ID_HASH_VALIDATION_BENCH_V1 admissions={EVENT_ADMISSION_COUNT} \
             unique_events={UNIQUE_EVENT_COUNT} borrowed_identity=true \
             ordered_p95_ns={} hash_p95_ns={}",
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
