use crate::core::CoreError;
use crate::core::ServiceKind;

use super::super::super::descriptors::{DependencySpec, RegistryName};

pub(super) fn is_canonical_module_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|first| !first.is_whitespace())
        && name
            .chars()
            .next_back()
            .is_some_and(|last| !last.is_whitespace())
}

pub(super) fn validate_service_descriptor(
    owner_module: &str,
    kind: ServiceKind,
    name: &RegistryName,
    dependencies: &[DependencySpec],
) -> Result<(), CoreError> {
    let actual_owner = name.module_name();
    if actual_owner != owner_module {
        return Err(CoreError::ServiceOwnerMismatch {
            name: name.to_string(),
            expected: owner_module.to_owned(),
            actual: actual_owner.to_string(),
        });
    }
    let actual_kind = name.service_kind();
    if actual_kind != kind {
        return Err(CoreError::ServiceKindMismatch {
            name: name.to_string(),
            expected: kind,
            actual: actual_kind,
        });
    }
    validate_driver_dependencies(kind, name, dependencies)?;
    Ok(())
}

fn validate_driver_dependencies(
    kind: ServiceKind,
    name: &RegistryName,
    dependencies: &[DependencySpec],
) -> Result<(), CoreError> {
    if kind != ServiceKind::Driver {
        return Ok(());
    }
    if dependencies.is_empty() {
        return Ok(());
    }
    if let [dependency] = dependencies {
        return validate_driver_dependency_kind(kind, name, dependency);
    }
    if let [first_dependency, second_dependency] = dependencies {
        validate_driver_dependency_kind(kind, name, first_dependency)?;
        return validate_driver_dependency_kind(kind, name, second_dependency);
    }
    if let [first_dependency, second_dependency, third_dependency] = dependencies {
        validate_driver_dependency_kind(kind, name, first_dependency)?;
        validate_driver_dependency_kind(kind, name, second_dependency)?;
        return validate_driver_dependency_kind(kind, name, third_dependency);
    }
    if let [first_dependency, second_dependency, third_dependency, fourth_dependency] = dependencies
    {
        validate_driver_dependency_kind(kind, name, first_dependency)?;
        validate_driver_dependency_kind(kind, name, second_dependency)?;
        validate_driver_dependency_kind(kind, name, third_dependency)?;
        return validate_driver_dependency_kind(kind, name, fourth_dependency);
    }
    if let [first_dependency, second_dependency, third_dependency, fourth_dependency, fifth_dependency] =
        dependencies
    {
        validate_driver_dependency_kind(kind, name, first_dependency)?;
        validate_driver_dependency_kind(kind, name, second_dependency)?;
        validate_driver_dependency_kind(kind, name, third_dependency)?;
        validate_driver_dependency_kind(kind, name, fourth_dependency)?;
        return validate_driver_dependency_kind(kind, name, fifth_dependency);
    }
    for dependency in dependencies {
        validate_driver_dependency_kind(kind, name, dependency)?;
    }
    Ok(())
}

fn validate_driver_dependency_kind(
    kind: ServiceKind,
    name: &RegistryName,
    dependency: &DependencySpec,
) -> Result<(), CoreError> {
    let dependency_kind = dependency.name.service_kind();
    if dependency_kind == ServiceKind::Driver {
        return Ok(());
    }
    Err(CoreError::InvalidServiceDependencyKind {
        service: name.to_string(),
        service_kind: kind,
        dependency: dependency.name.to_string(),
        dependency_kind,
    })
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::is_canonical_module_name;

    const CHECKS_PER_SAMPLE: usize = 32_768;
    const MODULE_NAME_BYTES: usize = 2_048;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fs_runtime475_preserves_canonical_module_name_semantics() {
        for canonical in [
            "runtime",
            "runtime core",
            "runtime\tcore",
            "runtime\u{2003}core",
        ] {
            assert!(is_canonical_module_name(canonical), "{canonical:?}");
        }
        for non_canonical in [
            "",
            " runtime",
            "runtime ",
            "\u{2003}runtime",
            "runtime\u{2003}",
        ] {
            assert!(
                !is_canonical_module_name(non_canonical),
                "{non_canonical:?}"
            );
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fs_runtime475_endpoint_module_name_validation_benchmark() {
        let module_name = format!("{}r", " ".repeat(MODULE_NAME_BYTES - 1));
        for _ in 0..4 {
            black_box(measure_checks(&module_name, false));
            black_box(measure_checks(&module_name, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_checks(&module_name, false));
                optimized_samples.push(measure_checks(&module_name, true));
            } else {
                optimized_samples.push(measure_checks(&module_name, true));
                legacy_samples.push(measure_checks(&module_name, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME475_ENDPOINT_MODULE_NAME_REJECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} module_name_bytes={MODULE_NAME_BYTES} leading_whitespace_bytes={} legacy_prefix_bytes_examined_per_check={} optimized_endpoint_chars_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=75",
            MODULE_NAME_BYTES - 1,
            MODULE_NAME_BYTES - 1,
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 25 / 100);
    }

    fn measure_checks(module_name: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let name = black_box(module_name);
            let canonical = if optimized {
                is_canonical_module_name(name)
            } else {
                !name.is_empty() && name.trim() == name
            };
            black_box(canonical);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
