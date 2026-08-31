use super::super::super::super::*;
use crate::core::CoreError;

#[test]
fn module_descriptor_defaults_to_post_without_module_dependencies() {
    let descriptor = ModuleDescriptor::new("DefaultModule", "default lifecycle metadata");

    assert_eq!(descriptor.init_level, InitLevel::Post);
    assert!(descriptor.module_dependencies.is_empty());
}

#[test]
fn module_activation_order_sorts_levels_and_declared_dependencies() {
    let descriptors = vec![
        ModuleDescriptor::new("EditorModule", "editor")
            .with_init_level(InitLevel::Editor)
            .with_module_dependency(ModuleDependencySpec::named("SceneModule")),
        ModuleDescriptor::new("KernelModule", "kernel").with_init_level(InitLevel::Kernel),
        ModuleDescriptor::new("SceneModule", "scene")
            .with_init_level(InitLevel::Scene)
            .with_module_dependency(ModuleDependencySpec::named("ServicesModule")),
        ModuleDescriptor::new("ServicesModule", "services").with_init_level(InitLevel::Services),
        ModuleDescriptor::new("PostModule", "post")
            .with_module_dependency(ModuleDependencySpec::named("EditorModule")),
    ];

    let order = sort_module_activation_order(&descriptors).unwrap();

    assert_eq!(
        order,
        vec![
            "KernelModule".to_owned(),
            "ServicesModule".to_owned(),
            "SceneModule".to_owned(),
            "EditorModule".to_owned(),
            "PostModule".to_owned(),
        ]
    );
}

#[test]
fn module_activation_order_rejects_missing_module_dependency() {
    let descriptors = [ModuleDescriptor::new("SceneModule", "scene")
        .with_init_level(InitLevel::Scene)
        .with_module_dependency(ModuleDependencySpec::named("AssetModule"))];

    let error = sort_module_activation_order(&descriptors).unwrap_err();

    assert!(matches!(
        error,
        CoreError::MissingModuleDependency { module, dependency }
            if module == "SceneModule" && dependency == "AssetModule"
    ));
}

#[test]
fn module_activation_order_rejects_dependency_on_later_init_level() {
    let descriptors = [
        ModuleDescriptor::new("KernelModule", "kernel")
            .with_init_level(InitLevel::Kernel)
            .with_module_dependency(ModuleDependencySpec::named("SceneModule")),
        ModuleDescriptor::new("SceneModule", "scene").with_init_level(InitLevel::Scene),
    ];

    let error = sort_module_activation_order(&descriptors).unwrap_err();

    assert!(matches!(
        error,
        CoreError::ModuleInitLevelViolation {
            module,
            module_level,
            dependency,
            dependency_level,
        } if module == "KernelModule"
            && module_level == "Kernel"
            && dependency == "SceneModule"
            && dependency_level == "Scene"
    ));
}

#[test]
fn module_activation_order_reports_same_level_cycles() {
    let descriptors = [
        ModuleDescriptor::new("AssetModule", "asset")
            .with_init_level(InitLevel::Services)
            .with_module_dependency(ModuleDependencySpec::named("InputModule")),
        ModuleDescriptor::new("InputModule", "input")
            .with_init_level(InitLevel::Services)
            .with_module_dependency(ModuleDependencySpec::named("AssetModule")),
    ];

    let error = sort_module_activation_order(&descriptors).unwrap_err();

    assert!(matches!(
        error,
        CoreError::ModuleDependencyCycle { path }
            if path == vec![
                "AssetModule".to_owned(),
                "InputModule".to_owned(),
                "AssetModule".to_owned(),
            ]
    ));
}

#[test]
fn module_activation_order_handles_a_one_hundred_thousand_module_deep_chain() {
    const MODULE_COUNT: usize = 100_000;

    let descriptors = (0..MODULE_COUNT)
        .map(|index| {
            let descriptor = ModuleDescriptor::new(
                format!("DeepModule{index:06}"),
                "deep activation-order regression",
            );
            if index + 1 == MODULE_COUNT {
                descriptor
            } else {
                descriptor.with_module_dependency(ModuleDependencySpec::named(format!(
                    "DeepModule{:06}",
                    index + 1
                )))
            }
        })
        .collect::<Vec<_>>();

    let order = sort_module_activation_order(&descriptors)
        .expect("a valid 100k module chain must not consume the native call stack");

    assert_eq!(order.len(), MODULE_COUNT);
    assert_eq!(order.first(), Some(&"DeepModule099999".to_owned()));
    assert_eq!(order.last(), Some(&"DeepModule000000".to_owned()));
}

#[test]
fn module_lifecycle_default_hooks_are_noop_and_ready() {
    #[derive(Debug)]
    struct DefaultLifecycle;

    impl ModuleLifecycle for DefaultLifecycle {}

    let runtime = CoreRuntime::new();
    let context = ModuleContext {
        module_name: "DefaultLifecycleModule".to_owned(),
        core: runtime.weak(),
    };
    let lifecycle = DefaultLifecycle;

    lifecycle.build(&context).unwrap();
    assert!(lifecycle.ready(&context).unwrap());
    lifecycle.finish(&context).unwrap();
    lifecycle.cleanup(&context).unwrap();
}
