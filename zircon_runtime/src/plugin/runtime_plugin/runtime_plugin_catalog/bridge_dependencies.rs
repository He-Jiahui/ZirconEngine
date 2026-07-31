use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::Arc;

use crate::plugin::RuntimePluginRegistrationReport;

const STRONG_DEPENDENCY_DIAGNOSTIC_CODE: &str = "bridge.strong_dependency_missing";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginBridgeDependent {
    pub package_id: String,
    pub interface_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginBridgeDisableBlocker {
    pub provider_package_id: String,
    pub dependent_package_id: String,
    pub interface_ids: Vec<String>,
}

impl RuntimePluginBridgeDisableBlocker {
    pub fn diagnostic(&self) -> String {
        format!(
            "bridge.strong_target_disable_blocked: provider plugin `{}` cannot be disabled while dependent plugin `{}` requires interfaces [{}]",
            self.provider_package_id,
            self.dependent_package_id,
            self.interface_ids
                .iter()
                .map(|interface_id| format!("`{interface_id}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

pub(super) fn bridge_dependency_diagnostics(
    registrations: &[RuntimePluginRegistrationReport],
) -> Vec<String> {
    bridge_dependency_diagnostics_with_stats(registrations).0
}

fn bridge_dependency_diagnostics_with_stats(
    registrations: &[RuntimePluginRegistrationReport],
) -> (Vec<String>, BridgeDependencyTraversalStats) {
    let mut graph = BridgeDependencyGraph::new(registrations);
    let mut emitted = HashSet::new();
    let mut diagnostics = Vec::new();
    if !graph.index_issue_reachability() {
        return (diagnostics, graph.stats);
    }

    for registration in registrations {
        let package_id = registration.package_manifest.id.as_str();
        let (issues, _) = graph.dependency_issues(package_id, &mut HashSet::new());
        for issue in issues {
            graph.stats.diagnostic_chain_segments += issue.chain.len();
            let diagnostic = format!(
                "{STRONG_DEPENDENCY_DIAGNOSTIC_CODE}: dependency closure for package `{package_id}` is incomplete; provider plugin `{}` {} for interface `{}`; chain: {}",
                issue.target_id,
                issue.reason,
                issue.interface_id,
                issue.chain.as_ref()
            );
            if emitted.insert(diagnostic.clone()) {
                diagnostics.push(diagnostic);
            }
        }
    }
    (diagnostics, graph.stats)
}

struct BridgeDependencyGraph<'a> {
    package_order: Vec<&'a str>,
    registered_plugins: HashSet<&'a str>,
    provided_interfaces_by_plugin: HashMap<&'a str, HashSet<&'a str>>,
    dependencies_by_plugin: HashMap<&'a str, Vec<BridgeDependencyEdge<'a>>>,
    issue_reachable_plugins: HashSet<&'a str>,
    closure_cache: HashMap<&'a str, Vec<BridgeDependencyIssue<'a>>>,
    stats: BridgeDependencyTraversalStats,
}

#[derive(Clone, Copy)]
struct BridgeDependencyEdge<'a> {
    target_id: &'a str,
    interface_ids: &'a [String],
}

#[derive(Clone)]
struct BridgeDependencyIssue<'a> {
    target_id: &'a str,
    interface_id: &'a str,
    reason: &'static str,
    chain: Arc<BridgeDependencyChain<'a>>,
}

struct BridgeDependencyChain<'a> {
    package_id: &'a str,
    next: Option<Arc<Self>>,
}

impl<'a> BridgeDependencyChain<'a> {
    fn direct(source_id: &'a str, target_id: &'a str) -> Arc<Self> {
        Arc::new(Self {
            package_id: source_id,
            next: Some(Arc::new(Self {
                package_id: target_id,
                next: None,
            })),
        })
    }

    fn prepend(package_id: &'a str, suffix: Arc<Self>) -> Arc<Self> {
        Arc::new(Self {
            package_id,
            next: Some(suffix),
        })
    }

    fn len(&self) -> usize {
        let mut len = 0;
        let mut current = Some(self);
        while let Some(node) = current {
            len += 1;
            current = node.next.as_deref();
        }
        len
    }
}

impl fmt::Display for BridgeDependencyChain<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = Some(self);
        let mut first = true;
        while let Some(node) = current {
            if !first {
                formatter.write_str(" -> ")?;
            }
            formatter.write_str(node.package_id)?;
            first = false;
            current = node.next.as_deref();
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BridgeDependencyTraversalStats {
    direct_edges_scanned: usize,
    direct_interfaces_scanned: usize,
    reachability_nodes_evaluated: usize,
    reachability_edges_evaluated: usize,
    nodes_evaluated: usize,
    edges_evaluated: usize,
    diagnostic_chain_segments: usize,
}

impl<'a> BridgeDependencyGraph<'a> {
    fn new(registrations: &'a [RuntimePluginRegistrationReport]) -> Self {
        let mut registered_plugins = HashSet::new();
        let mut package_order = Vec::new();
        let mut provided_interfaces_by_plugin = HashMap::new();
        let mut dependencies_by_plugin = HashMap::new();

        for registration in registrations {
            let package_id = registration.package_manifest.id.as_str();
            if registered_plugins.insert(package_id) {
                package_order.push(package_id);
            }
            provided_interfaces_by_plugin.insert(
                package_id,
                registration
                    .package_manifest
                    .provides_interfaces
                    .iter()
                    .map(|interface| interface.id.as_str())
                    .collect(),
            );
            dependencies_by_plugin.insert(
                package_id,
                registration
                    .package_manifest
                    .dependencies
                    .iter()
                    .filter(|dependency| dependency.required && !dependency.interfaces.is_empty())
                    .map(|dependency| BridgeDependencyEdge {
                        target_id: dependency.id.as_str(),
                        interface_ids: &dependency.interfaces,
                    })
                    .collect(),
            );
        }

        Self {
            package_order,
            registered_plugins,
            provided_interfaces_by_plugin,
            dependencies_by_plugin,
            issue_reachable_plugins: HashSet::new(),
            closure_cache: HashMap::new(),
            stats: BridgeDependencyTraversalStats::default(),
        }
    }

    fn dependency_issues(
        &mut self,
        current_id: &'a str,
        visiting: &mut HashSet<&'a str>,
    ) -> (Vec<BridgeDependencyIssue<'a>>, bool) {
        if !self.issue_reachable_plugins.contains(current_id) {
            return (Vec::new(), true);
        }
        if let Some(cached) = self.closure_cache.get(current_id) {
            return (cached.clone(), true);
        }
        if !visiting.insert(current_id) {
            return (Vec::new(), false);
        }
        self.stats.nodes_evaluated += 1;
        let dependencies = self
            .dependencies_by_plugin
            .get(current_id)
            .cloned()
            .unwrap_or_default();
        let mut issues = Vec::new();
        let mut cycle_free = true;

        for dependency in dependencies {
            self.stats.edges_evaluated += 1;
            for interface_id in dependency.interface_ids {
                let reason = match self.provided_interfaces_by_plugin.get(dependency.target_id) {
                    None => Some("is not registered"),
                    Some(provided_interfaces)
                        if !provided_interfaces.contains(interface_id.as_str()) =>
                    {
                        Some("does not declare the interface")
                    }
                    Some(_) => None,
                };
                if let Some(reason) = reason {
                    issues.push(BridgeDependencyIssue {
                        target_id: dependency.target_id,
                        interface_id,
                        reason,
                        chain: BridgeDependencyChain::direct(current_id, dependency.target_id),
                    });
                }
            }

            if self.registered_plugins.contains(dependency.target_id) {
                let (nested_issues, nested_cycle_free) =
                    self.dependency_issues(dependency.target_id, visiting);
                cycle_free &= nested_cycle_free;
                for nested_issue in nested_issues {
                    issues.push(BridgeDependencyIssue {
                        chain: BridgeDependencyChain::prepend(
                            current_id,
                            nested_issue.chain.clone(),
                        ),
                        ..nested_issue
                    });
                }
            }
        }
        visiting.remove(current_id);
        if cycle_free {
            self.closure_cache.insert(current_id, issues.clone());
        }
        (issues, cycle_free)
    }

    fn index_issue_reachability(&mut self) -> bool {
        let mut predecessors = HashMap::<&'a str, Vec<&'a str>>::new();
        let mut issue_sources = VecDeque::new();

        for source_id in &self.package_order {
            let mut source_has_issue = false;
            for dependency in self
                .dependencies_by_plugin
                .get(source_id)
                .into_iter()
                .flatten()
            {
                self.stats.direct_edges_scanned += 1;
                if self.registered_plugins.contains(dependency.target_id) {
                    predecessors
                        .entry(dependency.target_id)
                        .or_default()
                        .push(*source_id);
                }
                for interface_id in dependency.interface_ids {
                    self.stats.direct_interfaces_scanned += 1;
                    source_has_issue |= self
                        .provided_interfaces_by_plugin
                        .get(dependency.target_id)
                        .is_none_or(|provided| !provided.contains(interface_id.as_str()));
                }
            }
            if source_has_issue {
                issue_sources.push_back(*source_id);
            }
        }

        while let Some(package_id) = issue_sources.pop_front() {
            if !self.issue_reachable_plugins.insert(package_id) {
                continue;
            }
            self.stats.reachability_nodes_evaluated += 1;
            for predecessor in predecessors.get(package_id).into_iter().flatten() {
                self.stats.reachability_edges_evaluated += 1;
                if !self.issue_reachable_plugins.contains(predecessor) {
                    issue_sources.push_back(*predecessor);
                }
            }
        }

        !self.issue_reachable_plugins.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::plugin::{
        PluginDependencyManifest, PluginPackageManifest, RuntimePluginRegistrationReport,
    };

    use super::bridge_dependency_diagnostics_with_stats;

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

    fn bridge_chain(
        row_count: usize,
        missing_terminal: bool,
    ) -> Vec<RuntimePluginRegistrationReport> {
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
}
