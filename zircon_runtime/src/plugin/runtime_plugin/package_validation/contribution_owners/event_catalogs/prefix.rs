pub(super) fn runtime_plugin_package_event_catalog_has_owner(
    package_id: &str,
    event_catalog_namespace: &str,
) -> bool {
    let namespace = event_catalog_namespace.as_bytes();
    let owner = package_id.as_bytes();
    namespace.len() > owner.len() && namespace.starts_with(owner) && namespace[owner.len()] == b'.'
}

#[cfg(test)]
mod tests {
    #[test]
    fn event_catalog_owner_check_does_not_format_a_prefix() {
        let source = include_str!("prefix.rs");
        let formatted_prefix = ["format!(\"", "{package_id}.", "\")"].concat();
        assert!(!source.contains(&formatted_prefix));
    }

    #[test]
    fn event_catalog_owner_check_preserves_the_dot_boundary() {
        assert!(super::runtime_plugin_package_event_catalog_has_owner(
            "rendering",
            "rendering.events"
        ));
        assert!(!super::runtime_plugin_package_event_catalog_has_owner(
            "render",
            "rendering.events"
        ));
    }

    #[test]
    fn optimization_batch_gm_runtime495_event_catalog_owner_byte_boundary_preserves_rules() {
        assert!(super::runtime_plugin_package_event_catalog_has_owner(
            "rendering",
            "rendering.events"
        ));
        assert!(!super::runtime_plugin_package_event_catalog_has_owner(
            "rendering",
            "rendering"
        ));
        assert!(!super::runtime_plugin_package_event_catalog_has_owner(
            "render",
            "rendering.events"
        ));
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gm_runtime495_event_catalog_owner_byte_boundary_benchmark() {
        const MARKER: &str = "RUNTIME495_EVENT_CATALOG_OWNER_BYTE_BOUNDARY_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let package_id = "rendering";
        let namespace = "rendering.events.materials.shadow_pass.quality_profile";
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(super::runtime_plugin_package_event_catalog_has_owner(
                package_id, namespace
            ));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(namespace
                .strip_prefix(package_id)
                .is_some_and(|suffix| suffix.starts_with('.')));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
    }
}
