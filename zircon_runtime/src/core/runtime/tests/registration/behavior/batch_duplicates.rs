use std::sync::Arc;

use super::super::super::super::*;
use super::super::super::fixtures::TestDriver;
use crate::core::runtime::ServiceObject;
use crate::core::{CoreError, ServiceKind, StartupMode};

#[test]
fn register_exact_two_services_rejects_duplicate_batch_key_without_partial_commit() {
    let module_name = "ExactTwoBatchDuplicateModule";
    let duplicate_name =
        RegistryName::from_parts(module_name, ServiceKind::Driver, "DuplicateClockDriver");
    let descriptor = ModuleDescriptor::new(module_name, "exact-two batch duplicate")
        .with_driver(test_driver_descriptor(duplicate_name.clone(), 1))
        .with_driver(test_driver_descriptor(duplicate_name.clone(), 2));

    assert_duplicate_batch_rejected(descriptor, duplicate_name);
}

#[test]
fn register_exact_five_services_rejects_first_last_duplicate_without_partial_commit() {
    let module_name = "ExactFiveBatchDuplicateModule";
    let duplicate_name =
        RegistryName::from_parts(module_name, ServiceKind::Driver, "FirstLastDriver");
    let second_name = RegistryName::from_parts(module_name, ServiceKind::Driver, "SecondDriver");
    let third_name = RegistryName::from_parts(module_name, ServiceKind::Driver, "ThirdDriver");
    let fourth_name = RegistryName::from_parts(module_name, ServiceKind::Driver, "FourthDriver");
    let descriptor = ModuleDescriptor::new(module_name, "exact-five batch duplicate")
        .with_driver(test_driver_descriptor(duplicate_name.clone(), 1))
        .with_driver(test_driver_descriptor(second_name, 2))
        .with_driver(test_driver_descriptor(third_name, 3))
        .with_driver(test_driver_descriptor(fourth_name, 4))
        .with_driver(test_driver_descriptor(duplicate_name.clone(), 5));

    assert_duplicate_batch_rejected(descriptor, duplicate_name);
}

#[test]
fn register_six_services_rejects_first_last_duplicate_without_partial_commit() {
    let module_name = "SixBatchDuplicateModule";
    let duplicate_name =
        RegistryName::from_parts(module_name, ServiceKind::Driver, "FirstLastDriver");
    let descriptor = ModuleDescriptor::new(module_name, "six-service batch duplicate")
        .with_driver(test_driver_descriptor(duplicate_name.clone(), 1))
        .with_driver(test_driver_descriptor(
            RegistryName::from_parts(module_name, ServiceKind::Driver, "SecondDriver"),
            2,
        ))
        .with_driver(test_driver_descriptor(
            RegistryName::from_parts(module_name, ServiceKind::Driver, "ThirdDriver"),
            3,
        ))
        .with_driver(test_driver_descriptor(
            RegistryName::from_parts(module_name, ServiceKind::Driver, "FourthDriver"),
            4,
        ))
        .with_driver(test_driver_descriptor(
            RegistryName::from_parts(module_name, ServiceKind::Driver, "FifthDriver"),
            5,
        ))
        .with_driver(test_driver_descriptor(duplicate_name.clone(), 6));

    assert_duplicate_batch_rejected(descriptor, duplicate_name);
}

#[test]
fn register_small_batch_reports_the_first_duplicate_key_deterministically() {
    let module_name = "SmallBatchDuplicatePrecedenceModule";
    let first_duplicate =
        RegistryName::from_parts(module_name, ServiceKind::Driver, "FirstDuplicateDriver");
    let second_duplicate =
        RegistryName::from_parts(module_name, ServiceKind::Driver, "SecondDuplicateDriver");
    let descriptor = ModuleDescriptor::new(module_name, "small-batch duplicate precedence")
        .with_driver(test_driver_descriptor(first_duplicate.clone(), 1))
        .with_driver(test_driver_descriptor(second_duplicate.clone(), 2))
        .with_driver(test_driver_descriptor(first_duplicate.clone(), 3))
        .with_driver(test_driver_descriptor(second_duplicate, 4));

    assert_duplicate_batch_rejected(descriptor, first_duplicate);
}

fn assert_duplicate_batch_rejected(descriptor: ModuleDescriptor, duplicate_name: RegistryName) {
    let module_name = descriptor.name.clone();
    let runtime = CoreRuntime::new();
    let error = runtime
        .register_module(descriptor)
        .expect_err("a module batch must not contain duplicate service keys");

    assert!(matches!(
        error,
        CoreError::DuplicateService(name) if name == duplicate_name.as_str()
    ));

    let handle = runtime.handle();
    assert!(!handle
        .inner
        .modules
        .lock()
        .unwrap()
        .contains_key(&module_name));
    assert!(handle.inner.services.lock().unwrap().is_empty());
}

fn test_driver_descriptor(name: RegistryName, order: usize) -> DriverDescriptor {
    DriverDescriptor::new(
        name,
        StartupMode::Lazy,
        Vec::new(),
        Arc::new(move |_| Ok(Arc::new(TestDriver { order }) as ServiceObject)),
    )
}
