use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::{
    DependencySpec, ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor, PluginDescriptor,
    ServiceObject,
};
use super::*;

fn manager_descriptor(name: RegistryName, dependencies: Vec<DependencySpec>) -> ManagerDescriptor {
    ManagerDescriptor::new(
        name,
        StartupMode::Immediate,
        dependencies,
        Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
    )
}

fn plugin_descriptor(name: RegistryName, dependencies: Vec<DependencySpec>) -> PluginDescriptor {
    PluginDescriptor::new(
        name,
        StartupMode::Immediate,
        dependencies,
        Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
    )
}

#[test]
fn activation_sort_borrows_names_and_stacks_indices() {
    let source = include_str!("module_order.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("module order implementation");

    assert!(implementation.contains("HashMap<&str, usize>"));
    assert!(implementation.contains("HashSet<&str>"));
    assert!(implementation.contains("struct TraversalFrame"));
    assert!(implementation.contains("node_index: usize"));
    assert!(implementation.contains("next_dependency_index: usize"));
    assert!(implementation.contains("frames.push(TraversalFrame"));
    assert!(!implementation.contains("descriptor.name.clone(), index"));
}

#[test]
fn graph_walks_use_explicit_frames_without_recursive_helpers() {
    let source = include_str!("module_order.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("module order implementation");

    assert!(implementation.contains("struct TraversalFrame"));
    assert!(implementation.contains("fn visit_module_iterative("));
    assert!(implementation.contains("fn visit_service_iterative("));
    assert!(!implementation.contains("fn visit_module("));
    assert!(!implementation.contains("fn visit_service("));
    assert!(!implementation.contains("collect_module_dependencies"));
    assert!(!implementation.contains("collect_module_dependents"));
}

#[test]
fn same_kind_service_dependencies_shutdown_in_reverse_topological_order() {
    let first = RegistryName::from_parts("FrozenGraphModule", ServiceKind::Manager, "FirstManager");
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
fn service_dependencies_keep_their_stable_lexical_order_after_freezing() {
    let consumer = RegistryName::from_parts(
        "FrozenGraphModule",
        ServiceKind::Manager,
        "AConsumerManager",
    );
    let first_provider = RegistryName::from_parts(
        "FrozenGraphModule",
        ServiceKind::Manager,
        "YFirstProviderManager",
    );
    let second_provider = RegistryName::from_parts(
        "FrozenGraphModule",
        ServiceKind::Manager,
        "ZSecondProviderManager",
    );
    let graph = FrozenModuleGraph::freeze(&[ModuleDescriptor::new(
        "FrozenGraphModule",
        "stable service dependency order",
    )
    .with_manager(manager_descriptor(
        consumer.clone(),
        vec![
            DependencySpec::named(second_provider.clone()),
            DependencySpec::named(first_provider.clone()),
        ],
    ))
    .with_manager(manager_descriptor(first_provider.clone(), Vec::new()))
    .with_manager(manager_descriptor(second_provider.clone(), Vec::new()))])
    .expect("same-kind dependencies should produce a frozen graph");

    let services = graph
        .module_services("FrozenGraphModule")
        .expect("frozen graph module services");
    assert_eq!(
        services
            .service_names()
            .iter()
            .map(RegistryName::as_str)
            .collect::<Vec<_>>(),
        vec![
            first_provider.as_str(),
            second_provider.as_str(),
            consumer.as_str(),
        ]
    );
}

#[test]
fn service_validation_reports_the_first_declared_invalid_dependency() {
    let consumer =
        RegistryName::from_parts("FrozenGraphModule", ServiceKind::Manager, "ConsumerManager");
    let first_missing = RegistryName::from_parts(
        "FrozenGraphModule",
        ServiceKind::Manager,
        "ZFirstMissingManager",
    );
    let second_missing = RegistryName::from_parts(
        "FrozenGraphModule",
        ServiceKind::Manager,
        "ASecondMissingManager",
    );
    let descriptor = ModuleDescriptor::new("FrozenGraphModule", "diagnostic order").with_manager(
        manager_descriptor(
            consumer.clone(),
            vec![
                DependencySpec::named(first_missing.clone()),
                DependencySpec::named(second_missing),
            ],
        ),
    );

    assert!(matches!(
        FrozenModuleGraph::freeze(&[descriptor]),
        Err(CoreError::MissingServiceDependency {
            service,
            dependency,
        }) if service == consumer.as_str() && dependency == first_missing.as_str()
    ));
}

#[test]
fn manager_to_plugin_dependency_is_rejected_before_lifecycle_callbacks() {
    let plugin = RegistryName::from_parts("FrozenGraphModule", ServiceKind::Plugin, "LatePlugin");
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
    let consumer = RegistryName::from_parts("GraphConsumer", ServiceKind::Plugin, "ConsumerPlugin");
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
    let first = RegistryName::from_parts("CycleGraphModule", ServiceKind::Manager, "FirstManager");
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
fn service_activation_order_handles_a_one_hundred_thousand_service_deep_chain() {
    const SERVICE_COUNT: usize = 100_000;
    const MODULE_NAME: &str = "DeepServiceModule";

    let mut nodes = BTreeMap::new();
    for index in 0..SERVICE_COUNT {
        let service_segment = format!("Service{index:06}");
        let name = RegistryName::from_parts(MODULE_NAME, ServiceKind::Manager, &service_segment);
        let dependencies = (index + 1 < SERVICE_COUNT)
            .then(|| {
                RegistryName::from_parts(
                    MODULE_NAME,
                    ServiceKind::Manager,
                    &format!("Service{:06}", index + 1),
                )
            })
            .into_iter()
            .collect();
        nodes.insert(
            name.to_string(),
            ServiceGraphNode {
                name,
                owner_module: MODULE_NAME.to_owned(),
                kind: ServiceKind::Manager,
                startup_mode: StartupMode::Immediate,
                dependencies,
            },
        );
    }

    let order = sort_service_activation_order(&nodes)
        .expect("a valid 100k service chain must not consume the native call stack");

    assert_eq!(order.len(), SERVICE_COUNT);
    assert_eq!(
        order.first().map(RegistryName::as_str),
        Some("DeepServiceModule.Manager.Service099999")
    );
    assert_eq!(
        order.last().map(RegistryName::as_str),
        Some("DeepServiceModule.Manager.Service000000")
    );
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

#[test]
fn module_dependent_closure_excludes_the_target_and_keeps_global_order() {
    let provider = ModuleDescriptor::new("DependentProvider", "provider");
    let first_consumer = ModuleDescriptor::new("DependentFirstConsumer", "first consumer")
        .with_module_dependency(ModuleDependencySpec::named("DependentProvider"));
    let second_consumer = ModuleDescriptor::new("DependentSecondConsumer", "second consumer")
        .with_module_dependency(ModuleDependencySpec::named("DependentFirstConsumer"));
    let unrelated = ModuleDescriptor::new("DependentUnrelated", "unrelated");
    let graph = FrozenModuleGraph::freeze(&[provider, first_consumer, second_consumer, unrelated])
        .expect("declared dependent closure should produce a frozen graph");

    assert_eq!(
        graph
            .module_dependent_closure("DependentProvider")
            .expect("provider dependent closure"),
        vec!["DependentFirstConsumer", "DependentSecondConsumer"]
    );
}
