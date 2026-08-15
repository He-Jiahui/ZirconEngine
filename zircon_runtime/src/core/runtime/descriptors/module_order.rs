use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use super::super::error::{CoreError, CoreResult};
use super::{ModuleDescriptor, RegistryName};
use crate::core::{ServiceKind, StartupMode};

/// Immutable declaration snapshot used by every module lifecycle entry point.
///
/// Runtime entries keep their mutable lifecycle state in the registries. This
/// graph intentionally retains only validated topology and provenance, so the
/// declaration order cannot drift after lifecycle work has started.
#[derive(Clone, Debug)]
pub(crate) struct FrozenModuleGraph {
    module_activation_order: Arc<[String]>,
    module_dependencies: HashMap<String, Arc<[String]>>,
    module_dependents: HashMap<String, Arc<[String]>>,
    module_services: HashMap<String, FrozenModuleServices>,
}

#[derive(Clone, Debug)]
pub(crate) struct FrozenModuleServices {
    service_names: Arc<[RegistryName]>,
    startup_service_names: Arc<[RegistryName]>,
    shutdown_service_names: Arc<[RegistryName]>,
}

impl FrozenModuleGraph {
    pub(crate) fn freeze(descriptors: &[ModuleDescriptor]) -> CoreResult<Self> {
        let module_activation_order = sort_module_activation_order(descriptors)?;
        let module_dependencies = module_dependency_map(descriptors);
        let module_dependents =
            module_dependent_map(&module_dependencies, module_activation_order.as_slice());
        let service_nodes = collect_service_nodes(descriptors)?;

        validate_service_dependencies(&service_nodes, &module_dependencies)?;
        let service_activation_order = sort_service_activation_order(&service_nodes)?;
        let module_services = module_service_plans(
            module_activation_order.as_slice(),
            service_activation_order.as_slice(),
            &service_nodes,
        );

        Ok(Self {
            module_activation_order: module_activation_order.into(),
            module_dependencies,
            module_dependents,
            module_services,
        })
    }

    pub(crate) fn module_activation_order(&self) -> &[String] {
        &self.module_activation_order
    }

    pub(crate) fn module_activation_closure(&self, module_name: &str) -> CoreResult<Vec<String>> {
        if !self.module_dependencies.contains_key(module_name) {
            return Err(CoreError::MissingModule(module_name.to_owned()));
        }

        let mut closure = HashSet::new();
        self.collect_module_dependencies(module_name, &mut closure);
        Ok(self
            .module_activation_order
            .iter()
            .filter(|name| closure.contains(name.as_str()))
            .cloned()
            .collect())
    }

    /// Lists all transitively dependent modules in stable activation order.
    pub(crate) fn module_dependent_closure(&self, module_name: &str) -> CoreResult<Vec<String>> {
        if !self.module_dependents.contains_key(module_name) {
            return Err(CoreError::MissingModule(module_name.to_owned()));
        }

        let mut closure = HashSet::new();
        self.collect_module_dependents(module_name, &mut closure);
        Ok(self
            .module_activation_order
            .iter()
            .filter(|name| closure.contains(name.as_str()))
            .cloned()
            .collect())
    }

    pub(crate) fn module_services(&self, module_name: &str) -> CoreResult<&FrozenModuleServices> {
        self.module_services
            .get(module_name)
            .ok_or_else(|| CoreError::MissingModule(module_name.to_owned()))
    }

    fn collect_module_dependencies(&self, module_name: &str, closure: &mut HashSet<String>) {
        if !closure.insert(module_name.to_owned()) {
            return;
        }
        let dependencies = self
            .module_dependencies
            .get(module_name)
            .expect("validated module graph must contain the selected module");
        for dependency in dependencies.iter() {
            self.collect_module_dependencies(dependency, closure);
        }
    }

    fn collect_module_dependents(&self, module_name: &str, closure: &mut HashSet<String>) {
        let dependents = self
            .module_dependents
            .get(module_name)
            .expect("validated module graph must contain the selected module");
        for dependent in dependents.iter() {
            if closure.insert(dependent.clone()) {
                self.collect_module_dependents(dependent, closure);
            }
        }
    }
}

impl FrozenModuleServices {
    pub(crate) fn service_names(&self) -> &Arc<[RegistryName]> {
        &self.service_names
    }

    pub(crate) fn startup_service_names(&self) -> &Arc<[RegistryName]> {
        &self.startup_service_names
    }

    pub(crate) fn shutdown_service_names(&self) -> &Arc<[RegistryName]> {
        &self.shutdown_service_names
    }
}

#[derive(Clone, Debug)]
struct ServiceGraphNode {
    name: RegistryName,
    owner_module: String,
    kind: ServiceKind,
    startup_mode: StartupMode,
    dependencies: Vec<RegistryName>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Visited,
}

pub fn sort_module_activation_order(descriptors: &[ModuleDescriptor]) -> CoreResult<Vec<String>> {
    let mut by_name: HashMap<&str, usize> = HashMap::with_capacity(descriptors.len());
    for (index, descriptor) in descriptors.iter().enumerate() {
        if by_name.insert(descriptor.name.as_str(), index).is_some() {
            return Err(CoreError::DuplicateModule(descriptor.name.clone()));
        }
    }

    for descriptor in descriptors {
        let mut declared_dependencies =
            HashSet::with_capacity(descriptor.module_dependencies.len());
        for dependency in &descriptor.module_dependencies {
            if !declared_dependencies.insert(dependency.module_name.as_str()) {
                return Err(CoreError::DuplicateModuleDependency {
                    module: descriptor.name.clone(),
                    dependency: dependency.module_name.clone(),
                });
            }
            let Some(&dependency_index) = by_name.get(dependency.module_name.as_str()) else {
                return Err(CoreError::MissingModuleDependency {
                    module: descriptor.name.clone(),
                    dependency: dependency.module_name.clone(),
                });
            };
            let dependency_descriptor = &descriptors[dependency_index];
            if dependency_descriptor.init_level > descriptor.init_level {
                return Err(CoreError::ModuleInitLevelViolation {
                    module: descriptor.name.clone(),
                    module_level: descriptor.init_level.as_str().to_owned(),
                    dependency: dependency_descriptor.name.clone(),
                    dependency_level: dependency_descriptor.init_level.as_str().to_owned(),
                });
            }
        }
    }

    let mut traversal: Vec<usize> = (0..descriptors.len()).collect();
    traversal.sort_by_key(|&index| (descriptors[index].init_level, index));

    let mut states = vec![None; descriptors.len()];
    let mut stack = Vec::new();
    let mut order = Vec::with_capacity(descriptors.len());
    for index in traversal {
        visit_module(
            index,
            descriptors,
            &by_name,
            &mut states,
            &mut stack,
            &mut order,
        )?;
    }
    Ok(order)
}

fn module_dependency_map(descriptors: &[ModuleDescriptor]) -> HashMap<String, Arc<[String]>> {
    descriptors
        .iter()
        .map(|descriptor| {
            (
                descriptor.name.clone(),
                descriptor
                    .module_dependencies
                    .iter()
                    .map(|dependency| dependency.module_name.clone())
                    .collect::<Vec<_>>()
                    .into(),
            )
        })
        .collect()
}

fn module_dependent_map(
    module_dependencies: &HashMap<String, Arc<[String]>>,
    module_activation_order: &[String],
) -> HashMap<String, Arc<[String]>> {
    let mut dependents = module_activation_order
        .iter()
        .map(|module_name| (module_name.clone(), Vec::new()))
        .collect::<HashMap<_, _>>();
    for module_name in module_activation_order {
        let dependencies = module_dependencies
            .get(module_name)
            .expect("validated module graph must contain every module");
        for dependency in dependencies.iter() {
            dependents
                .get_mut(dependency)
                .expect("validated dependency must be a graph module")
                .push(module_name.clone());
        }
    }
    dependents
        .into_iter()
        .map(|(module_name, entries)| (module_name, entries.into()))
        .collect()
}

fn collect_service_nodes(
    descriptors: &[ModuleDescriptor],
) -> CoreResult<BTreeMap<String, ServiceGraphNode>> {
    let mut nodes = BTreeMap::new();
    for descriptor in descriptors {
        for driver in &descriptor.drivers {
            insert_service_node(
                &mut nodes,
                &descriptor.name,
                ServiceKind::Driver,
                driver.name.clone(),
                driver.startup_mode,
                &driver.dependencies,
            )?;
        }
        for manager in &descriptor.managers {
            insert_service_node(
                &mut nodes,
                &descriptor.name,
                ServiceKind::Manager,
                manager.name.clone(),
                manager.startup_mode,
                &manager.dependencies,
            )?;
        }
        for plugin in &descriptor.plugins {
            insert_service_node(
                &mut nodes,
                &descriptor.name,
                ServiceKind::Plugin,
                plugin.name.clone(),
                plugin.startup_mode,
                &plugin.dependencies,
            )?;
        }
    }
    Ok(nodes)
}

fn insert_service_node(
    nodes: &mut BTreeMap<String, ServiceGraphNode>,
    owner_module: &str,
    kind: ServiceKind,
    name: RegistryName,
    startup_mode: StartupMode,
    dependencies: &[super::DependencySpec],
) -> CoreResult<()> {
    let key = name.to_string();
    if nodes.contains_key(&key) {
        return Err(CoreError::DuplicateService(key));
    }
    nodes.insert(
        key,
        ServiceGraphNode {
            name,
            owner_module: owner_module.to_owned(),
            kind,
            startup_mode,
            dependencies: dependencies
                .iter()
                .map(|dependency| dependency.name.clone())
                .collect(),
        },
    );
    Ok(())
}

fn validate_service_dependencies(
    nodes: &BTreeMap<String, ServiceGraphNode>,
    module_dependencies: &HashMap<String, Arc<[String]>>,
) -> CoreResult<()> {
    for node in nodes.values() {
        let mut declared_dependencies = HashSet::with_capacity(node.dependencies.len());
        for dependency in &node.dependencies {
            let dependency_name = dependency.to_string();
            if !declared_dependencies.insert(dependency_name.clone()) {
                return Err(CoreError::DuplicateServiceDependency {
                    service: node.name.to_string(),
                    dependency: dependency_name,
                });
            }
            let Some(dependency_node) = nodes.get(dependency.as_str()) else {
                return Err(CoreError::MissingServiceDependency {
                    service: node.name.to_string(),
                    dependency: dependency_name,
                });
            };
            if !service_kind_can_depend_on(node.kind, dependency_node.kind) {
                return Err(CoreError::InvalidServiceDependencyKind {
                    service: node.name.to_string(),
                    service_kind: node.kind,
                    dependency: dependency_node.name.to_string(),
                    dependency_kind: dependency_node.kind,
                });
            }
            if node.owner_module != dependency_node.owner_module {
                let declared_modules = module_dependencies
                    .get(node.owner_module.as_str())
                    .expect("validated module graph must contain every service owner");
                if !declared_modules
                    .iter()
                    .any(|module_name| module_name == &dependency_node.owner_module)
                {
                    return Err(CoreError::UndeclaredCrossModuleServiceDependency {
                        service: node.name.to_string(),
                        service_module: node.owner_module.clone(),
                        dependency: dependency_node.name.to_string(),
                        dependency_module: dependency_node.owner_module.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn service_kind_can_depend_on(service: ServiceKind, dependency: ServiceKind) -> bool {
    service_kind_rank(dependency) <= service_kind_rank(service)
}

fn service_kind_rank(kind: ServiceKind) -> u8 {
    match kind {
        ServiceKind::Driver => 0,
        ServiceKind::Manager => 1,
        ServiceKind::Plugin => 2,
    }
}

fn sort_service_activation_order(
    nodes: &BTreeMap<String, ServiceGraphNode>,
) -> CoreResult<Vec<RegistryName>> {
    let mut states = HashMap::with_capacity(nodes.len());
    let mut stack = Vec::with_capacity(nodes.len());
    let mut order = Vec::with_capacity(nodes.len());
    for name in nodes.keys() {
        visit_service(name, nodes, &mut states, &mut stack, &mut order)?;
    }
    Ok(order)
}

fn visit_service(
    name: &str,
    nodes: &BTreeMap<String, ServiceGraphNode>,
    states: &mut HashMap<String, VisitState>,
    stack: &mut Vec<String>,
    order: &mut Vec<RegistryName>,
) -> CoreResult<()> {
    match states.get(name) {
        Some(VisitState::Visited) => return Ok(()),
        Some(VisitState::Visiting) => {
            let cycle_start = stack
                .iter()
                .position(|candidate| candidate == name)
                .expect("visiting service must be present in traversal stack");
            let mut path = stack[cycle_start..].to_vec();
            path.push(name.to_owned());
            return Err(CoreError::ServiceDependencyCycle { path });
        }
        None => {}
    }

    let node = nodes
        .get(name)
        .expect("validated service graph must contain every traversal node");
    states.insert(name.to_owned(), VisitState::Visiting);
    stack.push(name.to_owned());
    let mut dependencies = node.dependencies.iter().collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    for dependency in dependencies {
        visit_service(dependency.as_str(), nodes, states, stack, order)?;
    }
    stack.pop();
    states.insert(name.to_owned(), VisitState::Visited);
    order.push(node.name.clone());
    Ok(())
}

fn module_service_plans(
    module_activation_order: &[String],
    service_activation_order: &[RegistryName],
    nodes: &BTreeMap<String, ServiceGraphNode>,
) -> HashMap<String, FrozenModuleServices> {
    module_activation_order
        .iter()
        .map(|module_name| {
            let service_names = service_activation_order
                .iter()
                .filter(|service_name| {
                    nodes
                        .get(service_name.as_str())
                        .expect("topologically ordered service must retain graph metadata")
                        .owner_module
                        == *module_name
                })
                .cloned()
                .collect::<Vec<_>>();
            let startup_service_names = service_names
                .iter()
                .filter(|service_name| {
                    nodes
                        .get(service_name.as_str())
                        .expect("topologically ordered service must retain graph metadata")
                        .startup_mode
                        == StartupMode::Immediate
                })
                .cloned()
                .collect::<Vec<_>>();
            let shutdown_service_names = service_names.iter().rev().cloned().collect::<Vec<_>>();
            (
                module_name.clone(),
                FrozenModuleServices {
                    service_names: service_names.into(),
                    startup_service_names: startup_service_names.into(),
                    shutdown_service_names: shutdown_service_names.into(),
                },
            )
        })
        .collect()
}

fn visit_module(
    index: usize,
    descriptors: &[ModuleDescriptor],
    by_name: &HashMap<&str, usize>,
    states: &mut [Option<VisitState>],
    stack: &mut Vec<usize>,
    order: &mut Vec<String>,
) -> CoreResult<()> {
    match states[index] {
        Some(VisitState::Visited) => return Ok(()),
        Some(VisitState::Visiting) => {
            let name = &descriptors[index].name;
            let mut path = match stack.iter().position(|&candidate| candidate == index) {
                Some(cycle_start) => stack[cycle_start..]
                    .iter()
                    .map(|&module_index| descriptors[module_index].name.clone())
                    .collect(),
                None => Vec::new(),
            };
            path.push(name.clone());
            return Err(CoreError::ModuleDependencyCycle { path });
        }
        None => {}
    }

    states[index] = Some(VisitState::Visiting);
    stack.push(index);
    for dependency in &descriptors[index].module_dependencies {
        let Some(&dependency_index) = by_name.get(dependency.module_name.as_str()) else {
            return Err(CoreError::MissingModuleDependency {
                module: descriptors[index].name.clone(),
                dependency: dependency.module_name.clone(),
            });
        };
        visit_module(dependency_index, descriptors, by_name, states, stack, order)?;
    }
    stack.pop();
    states[index] = Some(VisitState::Visited);
    order.push(descriptors[index].name.clone());
    Ok(())
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn activation_sort_borrows_names_and_stacks_indices() {
        let source = include_str!("module_order.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("module order implementation");

        assert!(implementation.contains("HashMap<&str, usize>"));
        assert!(implementation.contains("stack: &mut Vec<usize>"));
        assert!(implementation.contains("stack.push(index);"));
        assert!(!implementation.contains("descriptor.name.clone(), index"));
    }
}

#[cfg(test)]
mod graph_tests {
    use std::sync::Arc;

    use super::super::{
        DependencySpec, ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor,
        PluginDescriptor, ServiceObject,
    };
    use super::*;

    fn manager_descriptor(
        name: RegistryName,
        dependencies: Vec<DependencySpec>,
    ) -> ManagerDescriptor {
        ManagerDescriptor::new(
            name,
            StartupMode::Immediate,
            dependencies,
            Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
        )
    }

    fn plugin_descriptor(
        name: RegistryName,
        dependencies: Vec<DependencySpec>,
    ) -> PluginDescriptor {
        PluginDescriptor::new(
            name,
            StartupMode::Immediate,
            dependencies,
            Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
        )
    }

    #[test]
    fn same_kind_service_dependencies_shutdown_in_reverse_topological_order() {
        let first =
            RegistryName::from_parts("FrozenGraphModule", ServiceKind::Manager, "FirstManager");
        let second =
            RegistryName::from_parts("FrozenGraphModule", ServiceKind::Manager, "SecondManager");
        let graph = FrozenModuleGraph::freeze(&[ModuleDescriptor::new(
            "FrozenGraphModule",
            "same-kind ordering",
        )
        .with_manager(manager_descriptor(first.clone(), Vec::new()))
        .with_manager(manager_descriptor(
            second.clone(),
            vec![DependencySpec::named(first.clone())],
        ))])
        .expect("same-kind manager dependency should be a valid frozen graph");

        let services = graph
            .module_services("FrozenGraphModule")
            .expect("frozen graph module services");
        assert_eq!(
            services
                .service_names()
                .iter()
                .map(RegistryName::as_str)
                .collect::<Vec<_>>(),
            vec![first.as_str(), second.as_str()]
        );
        assert_eq!(
            services
                .shutdown_service_names()
                .iter()
                .map(RegistryName::as_str)
                .collect::<Vec<_>>(),
            vec![second.as_str(), first.as_str()]
        );
    }

    #[test]
    fn manager_to_plugin_dependency_is_rejected_before_lifecycle_callbacks() {
        let plugin =
            RegistryName::from_parts("FrozenGraphModule", ServiceKind::Plugin, "LatePlugin");
        let manager_name =
            RegistryName::from_parts("FrozenGraphModule", ServiceKind::Manager, "EarlyManager");
        let descriptor = ModuleDescriptor::new("FrozenGraphModule", "kind validation")
            .with_manager(manager_descriptor(
                manager_name,
                vec![DependencySpec::named(plugin.clone())],
            ))
            .with_plugin(plugin_descriptor(plugin, Vec::new()));

        assert!(matches!(
            FrozenModuleGraph::freeze(&[descriptor]),
            Err(CoreError::InvalidServiceDependencyKind {
                service_kind: ServiceKind::Manager,
                dependency_kind: ServiceKind::Plugin,
                ..
            })
        ));
    }

    #[test]
    fn cross_module_service_dependency_requires_an_explicit_module_edge() {
        let provider =
            RegistryName::from_parts("GraphProvider", ServiceKind::Manager, "ProviderManager");
        let consumer =
            RegistryName::from_parts("GraphConsumer", ServiceKind::Plugin, "ConsumerPlugin");
        let provider_descriptor = ModuleDescriptor::new("GraphProvider", "provider")
            .with_manager(manager_descriptor(provider.clone(), Vec::new()));
        let consumer_descriptor =
            ModuleDescriptor::new("GraphConsumer", "consumer").with_plugin(plugin_descriptor(
                consumer.clone(),
                vec![DependencySpec::named(provider.clone())],
            ));

        assert!(matches!(
            FrozenModuleGraph::freeze(&[provider_descriptor, consumer_descriptor]),
            Err(CoreError::UndeclaredCrossModuleServiceDependency {
                service,
                service_module,
                dependency,
                dependency_module,
            }) if service == consumer.as_str()
                && service_module == "GraphConsumer"
                && dependency == provider.as_str()
                && dependency_module == "GraphProvider"
        ));
    }

    #[test]
    fn duplicate_module_dependencies_are_rejected_before_graph_traversal() {
        let provider = ModuleDescriptor::new("DuplicateEdgeProvider", "provider");
        let consumer = ModuleDescriptor::new("DuplicateEdgeConsumer", "consumer")
            .with_module_dependency(ModuleDependencySpec::named("DuplicateEdgeProvider"))
            .with_module_dependency(ModuleDependencySpec::named("DuplicateEdgeProvider"));

        assert!(matches!(
            FrozenModuleGraph::freeze(&[provider, consumer]),
            Err(CoreError::DuplicateModuleDependency { module, dependency })
                if module == "DuplicateEdgeConsumer" && dependency == "DuplicateEdgeProvider"
        ));
    }

    #[test]
    fn service_cycle_diagnostic_preserves_the_complete_stable_cycle_path() {
        let first =
            RegistryName::from_parts("CycleGraphModule", ServiceKind::Manager, "FirstManager");
        let second =
            RegistryName::from_parts("CycleGraphModule", ServiceKind::Manager, "SecondManager");
        let descriptor = ModuleDescriptor::new("CycleGraphModule", "service cycle")
            .with_manager(manager_descriptor(
                first.clone(),
                vec![DependencySpec::named(second.clone())],
            ))
            .with_manager(manager_descriptor(
                second.clone(),
                vec![DependencySpec::named(first.clone())],
            ));

        assert!(matches!(
            FrozenModuleGraph::freeze(&[descriptor]),
            Err(CoreError::ServiceDependencyCycle { path })
                if path == vec![first.to_string(), second.to_string(), first.to_string()]
        ));
    }

    #[test]
    fn module_activation_closure_filters_the_global_order_to_declared_dependencies() {
        let provider = ModuleDescriptor::new("ClosureProvider", "provider");
        let consumer = ModuleDescriptor::new("ClosureConsumer", "consumer")
            .with_module_dependency(ModuleDependencySpec::named("ClosureProvider"));
        let unrelated = ModuleDescriptor::new("ClosureUnrelated", "unrelated");
        let graph = FrozenModuleGraph::freeze(&[consumer, unrelated, provider])
            .expect("declared closure should produce a frozen graph");

        assert_eq!(
            graph
                .module_activation_closure("ClosureConsumer")
                .expect("consumer closure"),
            vec!["ClosureProvider", "ClosureConsumer"]
        );
    }
}
