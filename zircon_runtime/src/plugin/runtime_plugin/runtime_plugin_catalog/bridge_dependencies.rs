use std::collections::{HashMap, HashSet};

use crate::plugin::{PluginDependencyManifest, RuntimePluginRegistrationReport};

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

pub(super) fn bridge_disable_blockers_for_provider(
    registrations: &[RuntimePluginRegistrationReport],
    provider_package_id: &str,
) -> Vec<RuntimePluginBridgeDisableBlocker> {
    bridge_dependents_for_provider(registrations, provider_package_id)
        .into_iter()
        .map(|dependent| RuntimePluginBridgeDisableBlocker {
            provider_package_id: provider_package_id.to_string(),
            dependent_package_id: dependent.package_id,
            interface_ids: dependent.interface_ids,
        })
        .collect()
}

pub(super) fn bridge_dependents_for_provider(
    registrations: &[RuntimePluginRegistrationReport],
    provider_package_id: &str,
) -> Vec<RuntimePluginBridgeDependent> {
    let mut dependents = Vec::new();

    for registration in registrations {
        let mut interface_ids = Vec::new();
        for dependency in &registration.package_manifest.dependencies {
            if dependency.required
                && dependency.id == provider_package_id
                && !dependency.interfaces.is_empty()
            {
                interface_ids.extend(dependency.interfaces.iter().cloned());
            }
        }

        if !interface_ids.is_empty() {
            interface_ids.sort();
            interface_ids.dedup();
            dependents.push(RuntimePluginBridgeDependent {
                package_id: registration.package_manifest.id.clone(),
                interface_ids,
            });
        }
    }

    dependents.sort_by(|left, right| left.package_id.cmp(&right.package_id));
    dependents
}

pub(super) fn validate_bridge_dependency_closure(
    registrations: &[RuntimePluginRegistrationReport],
    diagnostics: &mut Vec<String>,
) {
    let graph = BridgeDependencyGraph::new(registrations);
    let mut emitted = HashSet::new();

    for registration in registrations {
        let package_id = registration.package_manifest.id.as_str();
        let mut chain = vec![package_id];
        graph.validate_package_dependencies(
            package_id,
            package_id,
            &mut chain,
            diagnostics,
            &mut emitted,
        );
    }
}

struct BridgeDependencyGraph<'a> {
    registrations_by_id: HashMap<&'a str, &'a RuntimePluginRegistrationReport>,
    provided_interfaces_by_plugin: HashMap<&'a str, HashSet<&'a str>>,
}

impl<'a> BridgeDependencyGraph<'a> {
    fn new(registrations: &'a [RuntimePluginRegistrationReport]) -> Self {
        let mut registrations_by_id = HashMap::new();
        let mut provided_interfaces_by_plugin = HashMap::new();

        for registration in registrations {
            let package_id = registration.package_manifest.id.as_str();
            registrations_by_id.insert(package_id, registration);
            provided_interfaces_by_plugin.insert(
                package_id,
                registration
                    .package_manifest
                    .provides_interfaces
                    .iter()
                    .map(|interface| interface.id.as_str())
                    .collect(),
            );
        }

        Self {
            registrations_by_id,
            provided_interfaces_by_plugin,
        }
    }

    fn validate_package_dependencies(
        &self,
        root_id: &'a str,
        current_id: &'a str,
        chain: &mut Vec<&'a str>,
        diagnostics: &mut Vec<String>,
        emitted: &mut HashSet<String>,
    ) {
        let Some(registration) = self.registrations_by_id.get(current_id) else {
            return;
        };

        for dependency in registration
            .package_manifest
            .dependencies
            .iter()
            .filter(|dependency| dependency.required && !dependency.interfaces.is_empty())
        {
            self.validate_dependency(root_id, dependency, chain, diagnostics, emitted);
        }
    }

    fn validate_dependency(
        &self,
        root_id: &'a str,
        dependency: &'a PluginDependencyManifest,
        chain: &mut Vec<&'a str>,
        diagnostics: &mut Vec<String>,
        emitted: &mut HashSet<String>,
    ) {
        let target_id = dependency.id.as_str();
        let already_in_chain = chain.contains(&target_id);
        chain.push(target_id);

        for interface_id in &dependency.interfaces {
            match self.provided_interfaces_by_plugin.get(target_id) {
                None => self.emit_missing_dependency(
                    root_id,
                    target_id,
                    interface_id,
                    "is not registered",
                    chain,
                    diagnostics,
                    emitted,
                ),
                Some(provided_interfaces)
                    if !provided_interfaces.contains(interface_id.as_str()) =>
                {
                    self.emit_missing_dependency(
                        root_id,
                        target_id,
                        interface_id,
                        "does not declare the interface",
                        chain,
                        diagnostics,
                        emitted,
                    );
                }
                Some(_) => {}
            }
        }

        if !already_in_chain && self.registrations_by_id.contains_key(target_id) {
            self.validate_package_dependencies(root_id, target_id, chain, diagnostics, emitted);
        }

        chain.pop();
    }

    fn emit_missing_dependency(
        &self,
        root_id: &str,
        target_id: &str,
        interface_id: &str,
        reason: &str,
        chain: &[&str],
        diagnostics: &mut Vec<String>,
        emitted: &mut HashSet<String>,
    ) {
        let diagnostic = format!(
            "{STRONG_DEPENDENCY_DIAGNOSTIC_CODE}: dependency closure for package `{root_id}` is incomplete; provider plugin `{target_id}` {reason} for interface `{interface_id}`; chain: {}",
            chain.join(" -> ")
        );

        if emitted.insert(diagnostic.clone()) {
            diagnostics.push(diagnostic);
        }
    }
}
