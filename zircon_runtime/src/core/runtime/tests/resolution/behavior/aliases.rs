use super::*;
use crate::core::manager::{manager_service_handle, resolve_manager_service};
use crate::core::{CoreResult, ModuleCleanupContext, ModuleLifecycle};
use crate::engine_module::dependency_on;
use std::sync::Weak;

const MODULE: &str = "BoundAliasModule";
const SOURCE: &str = "BoundAliasModule.Manager.Physical";
const FIRST: &str = "BoundAliasModule.Manager.FirstView";
const SECOND: &str = "BoundAliasModule.Manager.SecondView";

trait CounterView: Send + Sync {
    fn add(&self, value: usize);
    fn value(&self) -> usize;
}

struct Counter {
    value: AtomicUsize,
    drops: Arc<AtomicUsize>,
    core: crate::core::CoreWeak,
}

impl CounterView for Counter {
    fn add(&self, value: usize) {
        self.value.fetch_add(value, Ordering::SeqCst);
    }

    fn value(&self) -> usize {
        self.value.load(Ordering::SeqCst)
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

struct AliasFixture {
    runtime: CoreRuntime,
    factories: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
    original: Arc<Mutex<Option<Weak<Counter>>>>,
}

fn alias_fixture() -> AliasFixture {
    alias_fixture_with_lifecycle(Arc::new(crate::core::runtime::NoopModuleLifecycle))
}

fn alias_fixture_with_lifecycle(lifecycle: Arc<dyn ModuleLifecycle>) -> AliasFixture {
    alias_fixture_with_projection(lifecycle, |owner| owner)
}

fn alias_fixture_with_projection(
    lifecycle: Arc<dyn ModuleLifecycle>,
    project: for<'a> fn(&'a Counter) -> &'a (dyn CounterView + 'static),
) -> AliasFixture {
    let runtime = CoreRuntime::new();
    let factories = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let original = Arc::new(Mutex::new(None));
    let factory_count = Arc::clone(&factories);
    let factory_drops = Arc::clone(&drops);
    let factory_original = Arc::clone(&original);
    let mut descriptor = ModuleDescriptor::new(MODULE, "original owner and admitted aliases")
        .with_lifecycle(lifecycle)
        .with_manager(ManagerDescriptor::new(
            RegistryName::new(SOURCE).unwrap(),
            StartupMode::Immediate,
            Vec::new(),
            Arc::new(move |core| {
                factory_count.fetch_add(1, Ordering::SeqCst);
                let owner = Arc::new(Counter {
                    value: AtomicUsize::new(0),
                    drops: Arc::clone(&factory_drops),
                    core: core.clone(),
                });
                *factory_original.lock().unwrap() = Some(Arc::downgrade(&owner));
                Ok(owner as ServiceObject)
            }),
        ));
    for target in [FIRST, SECOND] {
        descriptor = descriptor.with_manager(ManagerDescriptor::new(
            RegistryName::new(target).unwrap(),
            StartupMode::Immediate,
            vec![dependency_on(MODULE, ServiceKind::Manager, "Physical")],
            Arc::new(move |core| {
                let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
                core.register_service_alias::<Counter, dyn CounterView>(
                    SOURCE,
                    ServiceKind::Manager,
                    target,
                    ServiceKind::Manager,
                    project,
                )
            }),
        ));
    }
    runtime.register_module(descriptor).unwrap();
    runtime.activate_module(MODULE).unwrap();
    AliasFixture {
        runtime,
        factories,
        drops,
        original,
    }
}

fn call_count(runtime: &CoreRuntime, name: &str) -> usize {
    runtime.handle().inner.services.lock().unwrap()[name].in_flight_calls
}

#[test]
fn alias_calls_use_one_original_object_and_admit_both_slots() {
    let fixture = alias_fixture();
    let core = fixture.runtime.handle();
    let first = manager_service_handle::<dyn CounterView>(&core, FIRST).unwrap();
    let second = manager_service_handle::<dyn CounterView>(&core, SECOND).unwrap();
    let first_call = resolve_manager_service(&core, first).unwrap();
    let physical = fixture.runtime.resolve_manager::<Counter>(SOURCE).unwrap();
    assert!(std::ptr::addr_eq(
        &*physical as &dyn CounterView,
        &*first_call,
    ));
    first_call.add(7);
    assert_eq!(physical.value(), 7);
    drop(physical);
    assert_eq!(call_count(&fixture.runtime, SOURCE), 1);
    assert_eq!(call_count(&fixture.runtime, FIRST), 1);
    let second_call = resolve_manager_service(&core, second).unwrap();
    assert_eq!(second_call.value(), 7);
    second_call.add(5);
    assert_eq!(first_call.value(), 12);
    assert_eq!(call_count(&fixture.runtime, SOURCE), 2);
    assert_eq!(call_count(&fixture.runtime, SECOND), 1);
    assert_eq!(fixture.factories.load(Ordering::SeqCst), 1);
    drop(first_call);
    drop(second_call);
    assert_eq!(call_count(&fixture.runtime, SOURCE), 0);
    assert_eq!(call_count(&fixture.runtime, FIRST), 0);
    assert_eq!(call_count(&fixture.runtime, SECOND), 0);
    fixture.runtime.deactivate_module(MODULE).unwrap();
}

#[test]
fn idle_owned_and_alias_handles_do_not_retain_the_physical_object() {
    let fixture = alias_fixture();
    let core = fixture.runtime.handle();
    let owned = fixture
        .runtime
        .resolve_manager_handle::<Counter>(SOURCE)
        .unwrap();
    let alias = manager_service_handle::<dyn CounterView>(&core, FIRST).unwrap();
    let weak = fixture.original.lock().unwrap().as_ref().unwrap().clone();
    assert!(weak.upgrade().is_some());
    fixture.runtime.deactivate_module(MODULE).unwrap();
    assert!(
        weak.upgrade().is_none(),
        "idle tokens must not retain the owner"
    );
    assert_eq!(fixture.drops.load(Ordering::SeqCst), 1);
    assert!(matches!(
        owned.enter(),
        Err(CoreError::StaleServiceHandle { .. })
    ));
    assert!(matches!(
        resolve_manager_service(&core, alias.clone()),
        Err(CoreError::StaleServiceHandle { .. })
    ));
    fixture.runtime.activate_module(MODULE).unwrap();
    assert_eq!(fixture.factories.load(Ordering::SeqCst), 2);
    assert!(matches!(
        resolve_manager_service(&core, alias),
        Err(CoreError::StaleServiceHandle { .. })
    ));
    let fresh = manager_service_handle::<dyn CounterView>(&core, FIRST).unwrap();
    assert_eq!(resolve_manager_service(&core, fresh).unwrap().value(), 0);
    fixture.runtime.deactivate_module(MODULE).unwrap();
    assert_eq!(fixture.drops.load(Ordering::SeqCst), 2);
}

#[test]
fn closed_physical_source_rejects_alias_without_partial_admission() {
    let fixture = alias_fixture();
    let core = fixture.runtime.handle();
    let alias = manager_service_handle::<dyn CounterView>(&core, FIRST).unwrap();
    core.inner
        .services
        .lock()
        .unwrap()
        .get_mut(SOURCE)
        .unwrap()
        .close_admission();
    assert!(matches!(
        resolve_manager_service(&core, alias),
        Err(CoreError::ServiceUnavailable(name)) if name == SOURCE
    ));
    assert_eq!(call_count(&fixture.runtime, SOURCE), 0);
    assert_eq!(call_count(&fixture.runtime, FIRST), 0);
    fixture.runtime.deactivate_module(MODULE).unwrap();
}

#[test]
fn replaced_physical_generation_rejects_live_alias_without_admitting_it() {
    let fixture = alias_fixture();
    let core = fixture.runtime.handle();
    let alias = manager_service_handle::<dyn CounterView>(&core, FIRST).unwrap();
    let retired = {
        let mut services = core.inner.services.lock().unwrap();
        let physical = services.get_mut(SOURCE).unwrap();
        physical.close_admission();
        physical.invalidate_for_unload()
    };
    drop(retired);
    assert!(matches!(
        resolve_manager_service(&core, alias),
        Err(CoreError::StaleServiceHandle { name, .. }) if name == SOURCE
    ));
    assert_eq!(call_count(&fixture.runtime, FIRST), 0);
    assert_eq!(call_count(&fixture.runtime, SOURCE), 0);
    assert_eq!(fixture.drops.load(Ordering::SeqCst), 1);
    fixture.runtime.deactivate_module(MODULE).unwrap();
}

struct AliasCleanup {
    calls: Arc<AtomicUsize>,
}

impl ModuleLifecycle for AliasCleanup {
    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> CoreResult<()> {
        assert!(
            context.with_existing_manager::<Counter>(SOURCE, |physical| {
                physical.add(1);
                assert_eq!(physical.value(), 1);
                Ok(())
            })?
        );
        for view in [FIRST, SECOND] {
            assert!(matches!(
                context.with_existing_manager::<Counter>(view, |_| {
                    panic!("declared aliases are not privileged cleanup owners");
                }),
                Err(CoreError::ServiceUnavailable(name)) if name == view
            ));
        }
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn cleanup_borrows_only_the_original_physical_registration() {
    let calls = Arc::new(AtomicUsize::new(0));
    let fixture = alias_fixture_with_lifecycle(Arc::new(AliasCleanup {
        calls: calls.clone(),
    }));
    let core = fixture.runtime.handle();
    let idle = manager_service_handle::<dyn CounterView>(&core, FIRST).unwrap();
    let weak = fixture.original.lock().unwrap().as_ref().unwrap().clone();
    fixture.runtime.deactivate_module(MODULE).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.drops.load(Ordering::SeqCst), 1);
    assert!(weak.upgrade().is_none());
    assert!(matches!(
        resolve_manager_service(&core, idle),
        Err(CoreError::StaleServiceHandle { .. })
    ));
}

#[test]
fn alias_factory_rejects_an_alias_source_without_leaving_an_admission() {
    let runtime = CoreRuntime::new();
    let drops = Arc::new(AtomicUsize::new(0));
    let physical_drops = drops.clone();
    let rejected = Arc::new(AtomicUsize::new(0));
    let factory_rejected = rejected.clone();
    runtime.register_module(
        ModuleDescriptor::new(MODULE, "single physical owner with a rejected chain")
            .with_manager(ManagerDescriptor::new(
                RegistryName::new(SOURCE).unwrap(), StartupMode::Immediate, Vec::new(),
                Arc::new(move |core| Ok(Arc::new(Counter {
                    value: AtomicUsize::new(0), drops: physical_drops.clone(), core: core.clone(),
                }) as ServiceObject)),
            ))
            .with_manager(ManagerDescriptor::new(
                RegistryName::new(FIRST).unwrap(), StartupMode::Immediate,
                vec![dependency_on(MODULE, ServiceKind::Manager, "Physical")],
                Arc::new(|core| {
                    core.upgrade().ok_or(CoreError::RuntimeUnavailable)?
                        .register_service_alias::<Counter, dyn CounterView>(
                            SOURCE, ServiceKind::Manager, FIRST, ServiceKind::Manager, |owner| owner,
                        )
                }),
            ))
            .with_manager(ManagerDescriptor::new(
                RegistryName::new(SECOND).unwrap(), StartupMode::Lazy,
                vec![dependency_on(MODULE, ServiceKind::Manager, "FirstView")],
                Arc::new(move |core| {
                    let result = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?
                        .register_service_alias::<Counter, dyn CounterView>(
                            FIRST, ServiceKind::Manager, SECOND, ServiceKind::Manager, |owner| owner,
                        );
                    assert!(matches!(&result, Err(CoreError::ServiceUnavailable(name)) if name == FIRST));
                    factory_rejected.fetch_add(1, Ordering::SeqCst);
                    result
                }),
            ))
    ).unwrap();
    runtime.activate_module(MODULE).unwrap();
    let core = runtime.handle();
    let token = manager_service_handle::<dyn CounterView>(&core, SECOND).unwrap();
    assert!(
        matches!(resolve_manager_service(&core, token), Err(CoreError::Initialization(name, _)) if name == SECOND)
    );
    assert_eq!(rejected.load(Ordering::SeqCst), 1);
    for name in [SOURCE, FIRST, SECOND] {
        assert_eq!(call_count(&runtime, name), 0);
    }
    runtime.deactivate_module(MODULE).unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

fn panic_projection(owner: &Counter) -> &(dyn CounterView + 'static) {
    let core = owner.core.upgrade().unwrap();
    assert!(
        core.inner.services.try_lock().is_ok(),
        "projection must run outside the registry lock"
    );
    panic!("controlled borrowed alias projection panic");
}

#[test]
fn projection_panic_releases_both_actual_admissions() {
    let fixture = alias_fixture_with_projection(
        Arc::new(crate::core::runtime::NoopModuleLifecycle),
        panic_projection,
    );
    let core = fixture.runtime.handle();
    let alias = manager_service_handle::<dyn CounterView>(&core, FIRST).unwrap();
    let call = resolve_manager_service(&core, alias).unwrap();
    assert_eq!(call_count(&fixture.runtime, SOURCE), 1);
    assert_eq!(call_count(&fixture.runtime, FIRST), 1);
    let panic =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || call.value())).unwrap_err();
    assert_eq!(
        panic.downcast_ref::<&str>().copied(),
        Some("controlled borrowed alias projection panic")
    );
    assert_eq!(call_count(&fixture.runtime, SOURCE), 0);
    assert_eq!(call_count(&fixture.runtime, FIRST), 0);
    fixture.runtime.deactivate_module(MODULE).unwrap();
    assert_eq!(fixture.drops.load(Ordering::SeqCst), 1);
}

#[test]
fn foreign_core_rejects_alias_tokens_and_factory_sources_before_admission() {
    let original = alias_fixture();
    let other = alias_fixture();
    let core = original.runtime.handle();
    let foreign_core = other.runtime.handle();
    let token = manager_service_handle::<dyn CounterView>(&core, FIRST).unwrap();
    assert!(
        matches!(resolve_manager_service(&foreign_core, token), Err(CoreError::ServiceUnavailable(name)) if name == FIRST)
    );
    let source = manager_service_handle::<Counter>(&core, SOURCE).unwrap();
    assert!(matches!(
        crate::core::manager::register_manager_alias::<Counter, dyn CounterView>(
            &foreign_core, source, SECOND, |owner| owner,
        ),
        Err(CoreError::ServiceUnavailable(name)) if name == SOURCE
    ));
    for fixture in [&original, &other] {
        for name in [SOURCE, FIRST, SECOND] {
            assert_eq!(call_count(&fixture.runtime, name), 0);
        }
        assert_eq!(fixture.factories.load(Ordering::SeqCst), 1);
        fixture.runtime.deactivate_module(MODULE).unwrap();
    }
}
