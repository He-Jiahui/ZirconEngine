use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::Arc;

use crate::plugin::RuntimePluginRegistrationReport;

const STRONG_DEPENDENCY_DIAGNOSTIC_CODE: &str = "bridge.strong_dependency_missing";
const DISABLE_BLOCKER_PROVIDER_PREFIX: &str =
    "bridge.strong_target_disable_blocked: provider plugin `";
const DISABLE_BLOCKER_DEPENDENT_PREFIX: &str = "` cannot be disabled while dependent plugin `";
const DISABLE_BLOCKER_INTERFACES_PREFIX: &str = "` requires interfaces [";
const DISABLE_BLOCKER_INTERFACE_SEPARATOR: &str = ", ";

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
        let mut diagnostic = String::with_capacity(self.diagnostic_len());
        self.write_diagnostic(&mut diagnostic);
        diagnostic
    }

    pub(super) fn write_diagnostic(&self, diagnostic: &mut String) {
        diagnostic.push_str(DISABLE_BLOCKER_PROVIDER_PREFIX);
        diagnostic.push_str(&self.provider_package_id);
        diagnostic.push_str(DISABLE_BLOCKER_DEPENDENT_PREFIX);
        diagnostic.push_str(&self.dependent_package_id);
        diagnostic.push_str(DISABLE_BLOCKER_INTERFACES_PREFIX);
        for (index, interface_id) in self.interface_ids.iter().enumerate() {
            if index != 0 {
                diagnostic.push_str(DISABLE_BLOCKER_INTERFACE_SEPARATOR);
            }
            diagnostic.push('`');
            diagnostic.push_str(interface_id);
            diagnostic.push('`');
        }
        diagnostic.push(']');
    }

    pub(super) fn diagnostic_len(&self) -> usize {
        DISABLE_BLOCKER_PROVIDER_PREFIX
            .len()
            .saturating_add(self.provider_package_id.len())
            .saturating_add(DISABLE_BLOCKER_DEPENDENT_PREFIX.len())
            .saturating_add(self.dependent_package_id.len())
            .saturating_add(DISABLE_BLOCKER_INTERFACES_PREFIX.len())
            .saturating_add(
                self.interface_ids
                    .iter()
                    .map(|interface_id| interface_id.len().saturating_add(2))
                    .sum::<usize>(),
            )
            .saturating_add(
                self.interface_ids
                    .len()
                    .saturating_sub(1)
                    .saturating_mul(DISABLE_BLOCKER_INTERFACE_SEPARATOR.len()),
            )
            .saturating_add(1)
    }
}

#[cfg(test)]
#[path = "bridge_dependencies/tests/diagnostic_buffer_tests.rs"]
mod diagnostic_buffer_tests;

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
    let package_capacity = graph.package_order.len();
    graph.closure_cache.reserve(package_capacity);
    let mut visiting = HashSet::with_capacity(package_capacity);

    for registration in registrations {
        let package_id = registration.package_manifest.id.as_str();
        visiting.clear();
        let (issues, _) = graph.dependency_issues(package_id, &mut visiting);
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
        let registration_capacity = registrations.len();
        let mut registered_plugins = HashSet::with_capacity(registration_capacity);
        let mut package_order = Vec::with_capacity(registration_capacity);
        let mut provided_interfaces_by_plugin = HashMap::with_capacity(registration_capacity);
        let mut dependencies_by_plugin = HashMap::with_capacity(registration_capacity);

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

        if issue_sources.is_empty() {
            return false;
        }
        self.issue_reachable_plugins
            .reserve(self.package_order.len());

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
#[path = "tests/bridge_dependencies.rs"]
mod tests;
