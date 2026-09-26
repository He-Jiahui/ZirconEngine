use std::sync::{Arc, Mutex};

use super::super::super::*;
use crate::core::{CoreError, CoreResult, ModuleContext, ModuleLifecycle};

mod activation;
mod batch_transaction;
mod deactivation;
mod module_lifecycle;
mod reactivation;

#[derive(Debug)]
// 以生命周期回调的可观测顺序验证单模块激活会先补齐依赖闭包；记录器只属于测试。
struct ActivationOrderLifecycle {
    module_name: &'static str,
    build_order: Arc<Mutex<Vec<&'static str>>>,
}

impl ModuleLifecycle for ActivationOrderLifecycle {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.build_order
            .lock()
            .expect("activation order recorder")
            .push(self.module_name);
        Ok(())
    }
}

#[test]
fn single_module_activation_validates_the_complete_declared_graph_before_callbacks() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(
            ModuleDescriptor::new("GraphConsumer", "single activation graph validation")
                .with_module_dependency(ModuleDependencySpec::named("MissingGraphProvider")),
        )
        .unwrap();

    assert!(matches!(
        runtime.activate_module("GraphConsumer"),
        Err(CoreError::MissingModuleDependency { module, dependency })
            if module == "GraphConsumer" && dependency == "MissingGraphProvider"
    ));
}

// 首次生命周期调用会生成不可变声明图，此后注册必须失败，避免回调执行时图和注册表分叉。
#[test]
fn first_lifecycle_operation_freezes_the_declared_module_graph() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(ModuleDescriptor::new("FrozenGraphModule", "graph freeze"))
        .unwrap();
    runtime.activate_module("FrozenGraphModule").unwrap();

    assert!(matches!(
        runtime.register_module(ModuleDescriptor::new(
            "LateModule",
            "must be a graph transaction"
        )),
        Err(CoreError::ModuleGraphFrozen)
    ));
}

#[test]
fn single_module_activation_runs_the_declared_module_closure_in_graph_order() {
    let runtime = CoreRuntime::new();
    let build_order = Arc::new(Mutex::new(Vec::new()));
    runtime
        .register_module(
            ModuleDescriptor::new("ClosureProvider", "provider").with_lifecycle(Arc::new(
                ActivationOrderLifecycle {
                    module_name: "ClosureProvider",
                    build_order: Arc::clone(&build_order),
                },
            )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("ClosureConsumer", "consumer")
                .with_module_dependency(ModuleDependencySpec::named("ClosureProvider"))
                .with_lifecycle(Arc::new(ActivationOrderLifecycle {
                    module_name: "ClosureConsumer",
                    build_order: Arc::clone(&build_order),
                })),
        )
        .unwrap();

    runtime.activate_module("ClosureConsumer").unwrap();

    assert_eq!(
        *build_order.lock().expect("activation order recorder"),
        vec!["ClosureProvider", "ClosureConsumer"]
    );
}
