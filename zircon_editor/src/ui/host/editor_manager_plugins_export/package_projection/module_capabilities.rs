use zircon_runtime::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

pub(in crate::ui::host::editor_manager_plugins_export) fn module_capabilities_for_package(
    package: &PluginPackageManifest,
    kind: PluginModuleKind,
) -> Vec<String> {
    let capacity = package
        .modules
        .iter()
        .filter(|module| module.kind == kind)
        .map(|module| module.capabilities.len())
        .sum();
    let mut capabilities = Vec::with_capacity(capacity);
    capabilities.extend(
        package
            .modules
            .iter()
            .filter(|module| module.kind == kind)
            .flat_map(|module| module.capabilities.iter().cloned()),
    );
    capabilities
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::plugin::{PluginModuleKind, PluginModuleManifest, PluginPackageManifest};

    use super::module_capabilities_for_package;

    const SAMPLE_PAIRS: usize = 17;
    const MODULES_PER_SAMPLE: usize = 128;
    const CAPABILITIES_PER_MODULE: usize = 8;

    #[test]
    fn module_capability_projection_reserves_matching_module_capacity() {
        let source = include_str!("module_capabilities.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("module capability projection implementation");

        assert!(implementation.contains("Vec::with_capacity(capacity)"));
        assert!(implementation.contains("capabilities.extend("));
    }

    #[test]
    fn module_capability_projection_preserves_module_order_and_kind_filter() {
        let package = package_with_modules(3, 2);
        let runtime = module_capabilities_for_package(&package, PluginModuleKind::Runtime);
        let editor = module_capabilities_for_package(&package, PluginModuleKind::Editor);

        assert_eq!(
            runtime,
            ["runtime-0-0", "runtime-0-1", "runtime-2-0", "runtime-2-1"]
        );
        assert_eq!(editor, ["editor-1-0", "editor-1-1"]);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cr_editor_module_capability_capacity_p95() {
        let package = package_with_modules(MODULES_PER_SAMPLE, CAPABILITIES_PER_MODULE);
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&package, false));
                optimized.push(measure(&package, true));
            } else {
                optimized.push(measure(&package, true));
                legacy.push(measure(&package, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR337_MODULE_CAPABILITY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} modules_per_sample={MODULES_PER_SAMPLE} capabilities_per_module={CAPABILITIES_PER_MODULE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn package_with_modules(
        module_count: usize,
        capabilities_per_module: usize,
    ) -> PluginPackageManifest {
        let modules = (0..module_count)
            .map(|module_index| {
                let capabilities = (0..capabilities_per_module).map(|capability| {
                    format!(
                        "{}-{module_index}-{capability}",
                        if module_index % 2 == 0 {
                            "runtime"
                        } else {
                            "editor"
                        }
                    )
                });
                if module_index % 2 == 0 {
                    PluginModuleManifest::runtime(
                        format!("module-{module_index}"),
                        format!("crate-{module_index}"),
                    )
                    .with_capabilities(capabilities)
                } else {
                    PluginModuleManifest::editor(
                        format!("module-{module_index}"),
                        format!("crate-{module_index}"),
                    )
                    .with_capabilities(capabilities)
                }
            })
            .collect();
        let mut package = PluginPackageManifest::new("test.package", "Test Package");
        package.modules = modules;
        package
    }

    fn measure(package: &PluginPackageManifest, use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let values = if use_capacity {
                module_capabilities_for_package(package, PluginModuleKind::Runtime)
            } else {
                package
                    .modules
                    .iter()
                    .filter(|module| module.kind == PluginModuleKind::Runtime)
                    .flat_map(|module| module.capabilities.iter().cloned())
                    .collect()
            };
            checksum ^= values.len();
            black_box(values);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

pub(in crate::ui::host::editor_manager_plugins_export) fn runtime_capabilities_for_package(
    package: &PluginPackageManifest,
) -> Vec<String> {
    module_capabilities_for_package(package, PluginModuleKind::Runtime)
}

pub(in crate::ui::host::editor_manager_plugins_export) fn editor_capabilities_for_package(
    package: &PluginPackageManifest,
) -> Vec<String> {
    module_capabilities_for_package(package, PluginModuleKind::Editor)
}
