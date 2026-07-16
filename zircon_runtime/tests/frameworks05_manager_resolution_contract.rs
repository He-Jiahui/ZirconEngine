use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreRuntime, DriverDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};

#[derive(Debug)]
struct ImmediateDriver {
    factory_order: usize,
}

#[test]
fn direct_immediate_service_resolution_reuses_module_activation_instance() {
    let runtime = CoreRuntime::new();
    let service_name = RegistryName::from_parts(
        "Frameworks05ImmediateModule",
        ServiceKind::Driver,
        "ImmediateDriver",
    );
    let factory_calls = Arc::new(AtomicUsize::new(0));
    let factory_calls_for_service = Arc::clone(&factory_calls);

    runtime
        .register_module(
            ModuleDescriptor::new(
                "Frameworks05ImmediateModule",
                "immediate service activation convergence contract",
            )
            .with_driver(DriverDescriptor::new(
                service_name.clone(),
                StartupMode::Immediate,
                Vec::new(),
                Arc::new(move |_| {
                    let factory_order = factory_calls_for_service.fetch_add(1, Ordering::SeqCst);
                    Ok(Arc::new(ImmediateDriver { factory_order }) as ServiceObject)
                }),
            )),
        )
        .unwrap();

    let driver = runtime
        .resolve_driver::<ImmediateDriver>(service_name.as_str())
        .unwrap();

    assert_eq!(driver.factory_order, 0);
    assert_eq!(factory_calls.load(Ordering::SeqCst), 1);
    let resolved_again = runtime
        .resolve_driver::<ImmediateDriver>(service_name.as_str())
        .unwrap();
    assert!(Arc::ptr_eq(&driver, &resolved_again));
}
