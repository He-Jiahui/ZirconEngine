//! Native control for the task ownership used before browser module activation.
#![cfg(not(target_arch = "wasm32"))]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use zircon_runtime::core::{
    CoreError, CoreResult, CoreRuntime, EngineTaskGraphOptions, ModuleContext,
    ModuleDependencySpec, ModuleDescriptor, ModuleLifecycle, TaskDescriptor,
    TaskGraphScopeDescriptor, TaskId, TaskPoolKind, TasksModule, TASKS_MODULE_NAME,
};
use zircon_runtime::engine_module::EngineModule;

struct StartupAssetControl {
    catalog_version: u32,
    builds: Arc<AtomicUsize>,
}

impl ModuleLifecycle for StartupAssetControl {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        assert_eq!(self.catalog_version, 1);
        self.builds.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

#[test]
fn early_io_does_not_freeze_module_registration() {
    let timeout = Duration::from_secs(5);
    let core =
        CoreRuntime::try_with_task_graph_options(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("create the runtime's execution owner");
    core.register_module(TasksModule.descriptor()).unwrap();

    let (sender, receiver) = mpsc::sync_channel(1);
    let scope = core
        .handle()
        .task_graph()
        .create_scope(TaskGraphScopeDescriptor::new("catalog-bootstrap").with_task_capacity(1))
        .unwrap();
    let ticket = scope
        .submit(
            TaskDescriptor::new(TaskId::new(1), TaskPoolKind::Io, "catalog-bootstrap"),
            move |_| sender.send(1_u32).unwrap(),
        )
        .unwrap();
    let catalog_version = receiver
        .recv_timeout(timeout)
        .expect("complete pre-activation IO work");
    scope.close_admission();
    assert!(scope.wait_until_quiescent(timeout));
    assert!(ticket.is_complete() && !ticket.is_cancelled());

    let builds = Arc::new(AtomicUsize::new(0));
    core.register_module(
        ModuleDescriptor::new("BootstrapAssetControl", "catalog-fed startup control")
            .with_module_dependency(ModuleDependencySpec::named(TASKS_MODULE_NAME))
            .with_lifecycle(Arc::new(StartupAssetControl {
                catalog_version,
                builds: Arc::clone(&builds),
            })),
    )
    .unwrap();
    assert_eq!(builds.load(Ordering::Relaxed), 0);
    core.activate_registered_modules().unwrap();
    assert_eq!(builds.load(Ordering::Relaxed), 1);
    assert!(matches!(
        core.register_module(ModuleDescriptor::new("AfterActivation", "too late")),
        Err(CoreError::ModuleGraphFrozen)
    ));

    drop(ticket);
    drop(scope);
    core.shutdown_registered_modules_with_drain_timeout(timeout)
        .unwrap();
    assert!(!core
        .shutdown_task_graph(timeout)
        .unwrap()
        .has_in_flight_work());
}
