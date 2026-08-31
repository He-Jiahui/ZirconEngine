use std::collections::HashSet;

use crate::plugin::{PluginOptionManifest, RuntimeExtensionRegistryError};

pub(in crate::plugin::extension_registry) fn validate_plugin_option_manifest(
    descriptor: &PluginOptionManifest,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_option_field("key", &descriptor.key)?;
    validate_plugin_option_key(&descriptor.key)?;
    validate_plugin_option_field("display_name", &descriptor.display_name)?;
    validate_plugin_option_field("value_type", &descriptor.value_type)?;
    validate_plugin_option_field("default_value", &descriptor.default_value)?;
    if let Some(required_capability) = &descriptor.required_capability {
        validate_plugin_option_field("required_capability", required_capability)?;
        validate_plugin_option_capability(required_capability)?;
    }

    match descriptor.value_type.as_str() {
        "bool" => {
            if !matches!(descriptor.default_value.as_str(), "true" | "false") {
                return invalid_plugin_option(format!(
                    "{} bool default_value `{}` must be true or false",
                    descriptor.key, descriptor.default_value
                ));
            }
        }
        "integer" => {
            if descriptor.default_value.parse::<i64>().is_err() {
                return invalid_plugin_option(format!(
                    "{} integer default_value `{}` must parse as i64",
                    descriptor.key, descriptor.default_value
                ));
            }
        }
        "number" => match descriptor.default_value.parse::<f64>() {
            Ok(number) if number.is_finite() => {}
            _ => {
                return invalid_plugin_option(format!(
                    "{} number default_value `{}` must parse as a finite f64",
                    descriptor.key, descriptor.default_value
                ));
            }
        },
        "string" => {
            if !descriptor.enum_values.is_empty() {
                return invalid_plugin_option(format!(
                    "{} non-enum option must not declare enum_values",
                    descriptor.key
                ));
            }
        }
        "enum" => validate_plugin_option_enum_values(descriptor)?,
        _ => {
            return invalid_plugin_option(format!(
                "{} value_type `{}` must be bool, integer, number, string, or enum",
                descriptor.key, descriptor.value_type
            ));
        }
    }

    if descriptor.value_type != "enum" && !descriptor.enum_values.is_empty() {
        return invalid_plugin_option(format!(
            "{} non-enum option must not declare enum_values",
            descriptor.key
        ));
    }
    Ok(())
}

fn validate_plugin_option_key(option_key: &str) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_option_namespace("key", option_key)
}

fn validate_plugin_option_capability(
    required_capability: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_option_namespace("required_capability", required_capability)
}

fn validate_plugin_option_namespace(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if !value.contains('.') {
        return invalid_plugin_option(format!(
            "{field_name} `{value}` must use at least two dot-separated namespace segments"
        ));
    }
    for segment in value.split('.') {
        if segment.is_empty()
            || !segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return invalid_plugin_option(format!(
                "{field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, and dots"
            ));
        }
    }
    Ok(())
}

fn validate_plugin_option_enum_values(
    descriptor: &PluginOptionManifest,
) -> Result<(), RuntimeExtensionRegistryError> {
    if descriptor.enum_values.is_empty() {
        return invalid_plugin_option(format!(
            "{} enum option must declare enum_values",
            descriptor.key
        ));
    }

    validate_plugin_option_enum_token(&descriptor.key, "default_value", &descriptor.default_value)?;
    let mut seen_values = HashSet::new();
    for enum_value in &descriptor.enum_values {
        validate_plugin_option_enum_token(&descriptor.key, "enum_values", enum_value)?;
        if !seen_values.insert(enum_value) {
            return invalid_plugin_option(format!(
                "{} enum_values entry `{}` must be unique",
                descriptor.key, enum_value
            ));
        }
    }
    if !descriptor
        .enum_values
        .iter()
        .any(|enum_value| enum_value == &descriptor.default_value)
    {
        return invalid_plugin_option(format!(
            "{} enum default_value `{}` must be declared in enum_values",
            descriptor.key, descriptor.default_value
        ));
    }
    Ok(())
}

fn validate_plugin_option_enum_token(
    option_key: &str,
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_option_field(field_name, value)?;
    if !value.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
    }) {
        return invalid_plugin_option(format!(
            "{option_key} enum {field_name} value `{value}` must contain only lowercase ASCII letters, digits, underscores, or hyphens"
        ));
    }
    Ok(())
}

fn validate_plugin_option_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if value.trim().is_empty() || value.trim() != value {
        return invalid_plugin_option(format!(
            "{field_name} `{value}` must be non-empty and trimmed"
        ));
    }
    Ok(())
}

fn invalid_plugin_option<T>(message: String) -> Result<T, RuntimeExtensionRegistryError> {
    Err(RuntimeExtensionRegistryError::InvalidPluginOption(message))
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const ENUM_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_ENUM_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        let rank = samples.len().saturating_mul(95).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    #[test]
    fn runtime42_hash_batch_plugin_option_p95_uses_nearest_rank() {
        let mut samples = (1..=17).map(Duration::from_nanos).collect::<Vec<_>>();

        assert_eq!(percentile_95(&mut samples), Duration::from_nanos(17));
    }

    fn enum_values() -> Vec<String> {
        (0..ENUM_ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "generated_enum_value_with_long_shared_identity_{:05}",
                    (index * 4_099) % UNIQUE_ENUM_COUNT
                )
            })
            .collect()
    }

    fn ordered_unique_count(enum_values: &[String]) -> usize {
        let mut unique = BTreeSet::new();
        enum_values
            .iter()
            .filter(|value| unique.insert(value.as_str()))
            .count()
    }

    fn hash_unique_count(enum_values: &[String]) -> usize {
        let mut unique = HashSet::new();
        enum_values
            .iter()
            .filter(|value| unique.insert(value.as_str()))
            .count()
    }

    #[test]
    fn runtime42_hash_batch_plugin_option_preserves_first_duplicate_error() {
        let descriptor =
            PluginOptionManifest::new("sample.render.mode", "Render Mode", "enum", "quality")
                .with_enum_values(["quality", "performance", "quality"]);

        let error = validate_plugin_option_manifest(&descriptor).unwrap_err();
        assert!(format!("{error:?}")
            .contains("sample.render.mode enum_values entry `quality` must be unique"));
    }

    #[test]
    fn runtime42_hash_batch_plugin_option_uses_borrowed_hash_set() {
        let source = include_str!("plugin_option.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("let mut seen_values = HashSet::new();"));
        assert!(production.contains("seen_values.insert(enum_value)"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn runtime42_hash_batch_plugin_option_performance_evidence() {
        let enum_values = enum_values();
        assert_eq!(
            ordered_unique_count(&enum_values),
            hash_unique_count(&enum_values)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&enum_values)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_unique_count(black_box(&enum_values)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_unique_count(black_box(&enum_values)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&enum_values)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "RUNTIME42_PLUGIN_OPTION_ENUM_HASH_VALIDATION_BENCH_V1 \
             admissions={ENUM_ADMISSION_COUNT} unique_values={UNIQUE_ENUM_COUNT} \
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
