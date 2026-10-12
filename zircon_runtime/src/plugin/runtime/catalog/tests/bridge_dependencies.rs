use std::collections::HashSet;

use crate::plugin::{
    PluginDependencyManifest, PluginPackageManifest, RuntimePluginRegistrationReport,
};

use super::{bridge_dependency_diagnostics_with_stats, BridgeDependencyGraph};

#[test]
fn memoized_bridge_chain_evaluates_each_node_and_edge_once() {
    for row_count in [1, 100, 1_000] {
        let registrations = bridge_chain(row_count, true);

        let (diagnostics, stats) = bridge_dependency_diagnostics_with_stats(&registrations);

        assert_eq!(diagnostics.len(), row_count);
        assert_eq!(stats.reachability_nodes_evaluated, row_count);
        assert_eq!(stats.reachability_edges_evaluated, row_count - 1);
        assert_eq!(stats.nodes_evaluated, row_count);
        assert_eq!(stats.edges_evaluated, row_count);
        assert_eq!(
            stats.diagnostic_chain_segments,
            row_count * (row_count + 3) / 2
        );
    }
}

#[test]
fn clean_bridge_cycle_short_circuits_after_linear_direct_issue_scan() {
    for row_count in [1, 100, 1_000] {
        let registrations = bridge_cycle(row_count);

        let (diagnostics, stats) = bridge_dependency_diagnostics_with_stats(&registrations);

        assert!(diagnostics.is_empty());
        assert_eq!(stats.direct_edges_scanned, row_count);
        assert_eq!(stats.direct_interfaces_scanned, row_count);
        assert_eq!(stats.reachability_nodes_evaluated, 0);
        assert_eq!(stats.reachability_edges_evaluated, 0);
        assert_eq!(stats.nodes_evaluated, 0);
        assert_eq!(stats.edges_evaluated, 0);
        assert_eq!(stats.diagnostic_chain_segments, 0);
    }
}

#[test]
fn bridge_cycle_with_missing_interface_is_linear_in_emitted_chain_segments() {
    for row_count in [1, 100, 1_000] {
        let mut registrations = bridge_cycle(row_count);
        registrations
            .last_mut()
            .expect("cycle fixture should contain a row")
            .package_manifest
            .dependencies
            .push(
                PluginDependencyManifest::new("bridge_missing", true)
                    .with_interface("bridge.missing.v1"),
            );

        let (diagnostics, stats) = bridge_dependency_diagnostics_with_stats(&registrations);

        assert_eq!(diagnostics.len(), row_count);
        assert_eq!(stats.reachability_nodes_evaluated, row_count);
        assert_eq!(stats.reachability_edges_evaluated, row_count);
        assert_eq!(stats.nodes_evaluated, row_count * row_count);
        assert_eq!(stats.edges_evaluated, row_count * (row_count + 1));
        assert_eq!(
            stats.diagnostic_chain_segments,
            row_count * (row_count + 3) / 2
        );
    }
}

#[test]
fn clean_dense_cycle_is_not_traversed_for_an_unrelated_issue() {
    for row_count in [1, 10, 100] {
        let mut registrations = dense_clean_bridge_component(row_count);
        registrations.push(
            RuntimePluginRegistrationReport::from_native_package_manifest(
                PluginPackageManifest::new("unrelated", "unrelated").with_dependency(
                    PluginDependencyManifest::new("bridge_missing", true)
                        .with_interface("bridge.missing.v1"),
                ),
            ),
        );

        let (diagnostics, stats) = bridge_dependency_diagnostics_with_stats(&registrations);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(stats.direct_edges_scanned, row_count * row_count + 1);
        assert_eq!(stats.reachability_nodes_evaluated, 1);
        assert_eq!(stats.reachability_edges_evaluated, 0);
        assert_eq!(stats.nodes_evaluated, 1);
        assert_eq!(stats.edges_evaluated, 1);
        assert_eq!(stats.diagnostic_chain_segments, 2);
    }
}

#[test]
fn cyclic_diagnostics_keep_each_root_manifest_order() {
    let registrations = vec![
        RuntimePluginRegistrationReport::from_native_package_manifest(
            PluginPackageManifest::new("bridge_a", "bridge_a")
                .with_provided_interface_id("bridge.link.v1")
                .with_dependency(
                    PluginDependencyManifest::new("missing_a", true)
                        .with_interface("bridge.missing.a.v1"),
                )
                .with_dependency(
                    PluginDependencyManifest::new("bridge_b", true)
                        .with_interface("bridge.link.v1"),
                ),
        ),
        RuntimePluginRegistrationReport::from_native_package_manifest(
            PluginPackageManifest::new("bridge_b", "bridge_b")
                .with_provided_interface_id("bridge.link.v1")
                .with_dependency(
                    PluginDependencyManifest::new("missing_b", true)
                        .with_interface("bridge.missing.b.v1"),
                )
                .with_dependency(
                    PluginDependencyManifest::new("bridge_a", true)
                        .with_interface("bridge.link.v1"),
                ),
        ),
    ];

    let (diagnostics, _) = bridge_dependency_diagnostics_with_stats(&registrations);

    assert_eq!(
        diagnostics,
        [
            "bridge.strong_dependency_missing: dependency closure for package `bridge_a` is incomplete; provider plugin `missing_a` is not registered for interface `bridge.missing.a.v1`; chain: bridge_a -> missing_a",
            "bridge.strong_dependency_missing: dependency closure for package `bridge_a` is incomplete; provider plugin `missing_b` is not registered for interface `bridge.missing.b.v1`; chain: bridge_a -> bridge_b -> missing_b",
            "bridge.strong_dependency_missing: dependency closure for package `bridge_b` is incomplete; provider plugin `missing_b` is not registered for interface `bridge.missing.b.v1`; chain: bridge_b -> missing_b",
            "bridge.strong_dependency_missing: dependency closure for package `bridge_b` is incomplete; provider plugin `missing_a` is not registered for interface `bridge.missing.a.v1`; chain: bridge_b -> bridge_a -> missing_a",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn cycle_edge_issue_keeps_the_full_trigger_chain() {
    let registrations = vec![
        RuntimePluginRegistrationReport::from_native_package_manifest(
            PluginPackageManifest::new("bridge_a", "bridge_a")
                .with_provided_interface_id("bridge.link.v1")
                .with_dependency(
                    PluginDependencyManifest::new("bridge_b", true)
                        .with_interface("bridge.link.v1"),
                ),
        ),
        RuntimePluginRegistrationReport::from_native_package_manifest(
            PluginPackageManifest::new("bridge_b", "bridge_b")
                .with_provided_interface_id("bridge.link.v1")
                .with_dependency(
                    PluginDependencyManifest::new("bridge_a", true)
                        .with_interface("bridge.loop.missing.v1"),
                ),
        ),
    ];

    let (diagnostics, _) = bridge_dependency_diagnostics_with_stats(&registrations);

    assert_eq!(
        diagnostics,
        [
            "bridge.strong_dependency_missing: dependency closure for package `bridge_a` is incomplete; provider plugin `bridge_a` does not declare the interface for interface `bridge.loop.missing.v1`; chain: bridge_a -> bridge_b -> bridge_a",
            "bridge.strong_dependency_missing: dependency closure for package `bridge_b` is incomplete; provider plugin `bridge_a` does not declare the interface for interface `bridge.loop.missing.v1`; chain: bridge_b -> bridge_a",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn reused_root_visiting_scratch_clears_after_cycle_diagnostics() {
    let mut registrations = bridge_cycle(3);
    registrations
        .last_mut()
        .expect("cycle fixture should contain a row")
        .package_manifest
        .dependencies
        .push(
            PluginDependencyManifest::new("bridge_missing", true)
                .with_interface("bridge.missing.v1"),
        );
    let mut graph = BridgeDependencyGraph::new(&registrations);
    let registration_capacity = registrations.len();

    assert!(graph.package_order.capacity() >= registration_capacity);
    assert!(graph.registered_plugins.capacity() >= registration_capacity);
    assert!(graph.provided_interfaces_by_plugin.capacity() >= registration_capacity);
    assert!(graph.dependencies_by_plugin.capacity() >= registration_capacity);
    assert_eq!(graph.issue_reachable_plugins.capacity(), 0);
    assert_eq!(graph.closure_cache.capacity(), 0);
    assert!(graph.index_issue_reachability());
    assert!(graph.issue_reachable_plugins.capacity() >= registration_capacity);

    let mut visiting = HashSet::with_capacity(registration_capacity);
    let (_, first_cycle_free) = graph.dependency_issues("bridge_0000", &mut visiting);
    assert!(!first_cycle_free);
    assert!(visiting.is_empty());

    let (_, second_cycle_free) = graph.dependency_issues("bridge_0001", &mut visiting);
    assert!(!second_cycle_free);
    assert!(visiting.is_empty());
}

fn bridge_chain(row_count: usize, missing_terminal: bool) -> Vec<RuntimePluginRegistrationReport> {
    (0..row_count)
        .map(|index| {
            let package_id = format!("bridge_{index:04}");
            let mut package = PluginPackageManifest::new(&package_id, &package_id)
                .with_provided_interface_id("bridge.link.v1");
            if index + 1 < row_count {
                package = package.with_dependency(
                    PluginDependencyManifest::new(format!("bridge_{:04}", index + 1), true)
                        .with_interface("bridge.link.v1"),
                );
            } else if missing_terminal {
                package = package.with_dependency(
                    PluginDependencyManifest::new("bridge_missing", true)
                        .with_interface("bridge.missing.v1"),
                );
            }
            RuntimePluginRegistrationReport::from_native_package_manifest(package)
        })
        .collect()
}

fn bridge_cycle(row_count: usize) -> Vec<RuntimePluginRegistrationReport> {
    let mut registrations = bridge_chain(row_count, false);
    let first_package_id = "bridge_0000";
    let last = registrations
        .last_mut()
        .expect("cycle fixture should contain at least one row");
    last.package_manifest.dependencies.push(
        PluginDependencyManifest::new(first_package_id, true).with_interface("bridge.link.v1"),
    );
    registrations
}

fn dense_clean_bridge_component(row_count: usize) -> Vec<RuntimePluginRegistrationReport> {
    (0..row_count)
        .map(|source_index| {
            let source_id = format!("dense_{source_index:04}");
            let mut package = PluginPackageManifest::new(&source_id, &source_id)
                .with_provided_interface_id("bridge.link.v1");
            for target_index in 0..row_count {
                package = package.with_dependency(
                    PluginDependencyManifest::new(format!("dense_{target_index:04}"), true)
                        .with_interface("bridge.link.v1"),
                );
            }
            RuntimePluginRegistrationReport::from_native_package_manifest(package)
        })
        .collect()
}
