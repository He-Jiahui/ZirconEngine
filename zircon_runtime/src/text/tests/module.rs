use std::sync::Arc;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::CoreRuntime;

use crate::text::{TextRuntimeContextLifecycleState, TextSystemFontPolicy};

use super::{module_descriptor, text_runtime_context_for_core, TextModule, TEXT_MODULE_NAME};

fn runtime_with_text_services() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(module_descriptor())
        .expect("text module should register");
    runtime
        .activate_module(TEXT_MODULE_NAME)
        .expect("text module should activate");
    runtime
}

#[test]
fn text_module_selects_system_font_policy_from_runtime_target() {
    assert_eq!(
        TextModule::for_target(RuntimeTargetMode::ClientRuntime).system_font_policy,
        TextSystemFontPolicy::DiscoverPlatform
    );
    assert_eq!(
        TextModule::for_target(RuntimeTargetMode::EditorHost).system_font_policy,
        TextSystemFontPolicy::DiscoverPlatform
    );
    assert_eq!(
        TextModule::for_target(RuntimeTargetMode::ServerRuntime).system_font_policy,
        TextSystemFontPolicy::PackagedOnly
    );
    assert_eq!(
        TextModule::default().system_font_policy,
        TextSystemFontPolicy::PackagedOnly
    );
}

#[test]
fn text_font_services_are_stable_within_one_runtime_and_isolated_across_runtimes() {
    let first_runtime = runtime_with_text_services();
    let second_runtime = runtime_with_text_services();
    let first_context =
        text_runtime_context_for_core(&first_runtime.handle()).expect("first runtime text context");
    let first_again_context = text_runtime_context_for_core(&first_runtime.handle())
        .expect("first runtime text context should remain resolvable");
    let second_context = text_runtime_context_for_core(&second_runtime.handle())
        .expect("second runtime text context");
    let first = first_context.font_collection();
    let first_again = first_again_context.font_collection();
    let second = second_context.font_collection();

    assert!(Arc::ptr_eq(&first, &first_again));
    assert!(!Arc::ptr_eq(&first, &second));
    assert_ne!(first.collection_id(), second.collection_id());
}

#[test]
fn text_runtime_context_is_stable_per_core_and_isolated_across_cores() {
    let first_runtime = runtime_with_text_services();
    let second_runtime = runtime_with_text_services();
    let first =
        text_runtime_context_for_core(&first_runtime.handle()).expect("first runtime text context");
    let first_again = text_runtime_context_for_core(&first_runtime.handle())
        .expect("first runtime text context should remain resolvable");
    let second = text_runtime_context_for_core(&second_runtime.handle())
        .expect("second runtime text context");

    assert_eq!(first.id(), first_again.id());
    assert_ne!(first.id(), second.id());
    assert_ne!(
        first.font_collection_revision().collection_id(),
        second.font_collection_revision().collection_id()
    );
}

#[test]
fn text_runtime_context_module_cleanup_closes_admission_and_reactivation_replaces_it() {
    let runtime = runtime_with_text_services();
    let core = runtime.handle();
    let handle = super::text_runtime_context_handle(&core).unwrap();
    let retained = text_runtime_context_for_core(&core).unwrap();
    let retained_id = retained.id();
    assert!(matches!(
        runtime.deactivate_module_with_drain_timeout(TEXT_MODULE_NAME, std::time::Duration::ZERO),
        Err(crate::core::CoreError::ServiceCallDrainTimeout { .. })
    ));
    assert!(text_runtime_context_for_core(&core).is_err());
    drop(retained);
    runtime
        .deactivate_module(TEXT_MODULE_NAME)
        .expect("drained text cleanup");
    runtime
        .activate_module(TEXT_MODULE_NAME)
        .expect("reactivate text");
    assert!(crate::core::manager::resolve_manager_service(&core, handle).is_err());
    let replacement = text_runtime_context_for_core(&core).unwrap();
    assert_ne!(retained_id, replacement.id());
    assert_eq!(
        replacement.lifecycle_state(),
        TextRuntimeContextLifecycleState::Active
    );
}

#[test]
fn cloned_text_descriptor_cleanup_is_isolated_across_cores() {
    let descriptor = module_descriptor();
    let first = CoreRuntime::new();
    let second = CoreRuntime::new();
    first.register_module(descriptor.clone()).unwrap();
    second.register_module(descriptor).unwrap();
    first.activate_module(TEXT_MODULE_NAME).unwrap();
    second.activate_module(TEXT_MODULE_NAME).unwrap();
    let second_context = text_runtime_context_for_core(&second.handle()).unwrap();
    let second_id = second_context.id();
    first.deactivate_module(TEXT_MODULE_NAME).unwrap();
    assert_eq!(
        second_context.lifecycle_state(),
        TextRuntimeContextLifecycleState::Active
    );
    assert!(second_context.create_layout_session().is_ok());
    first.activate_module(TEXT_MODULE_NAME).unwrap();
    first.deactivate_module(TEXT_MODULE_NAME).unwrap();
    assert_eq!(second_context.id(), second_id);
    drop(second_context);
    second.deactivate_module(TEXT_MODULE_NAME).unwrap();
}

#[test]
fn text_owner_cleanup_closes_only_matching_factory_resources_and_is_repeatable() {
    use crate::core::{CoreResult, ModuleCleanupContext, ModuleLifecycle};
    use std::sync::Mutex;

    type Observations = Arc<Mutex<Vec<(u64, TextRuntimeContextLifecycleState, bool)>>>;

    #[derive(Debug)]
    struct ObservedTextCleanup {
        observations: Observations,
    }

    impl ModuleLifecycle for ObservedTextCleanup {
        fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> CoreResult<()> {
            // Both calls use the actual Core-issued owner and production cleanup.
            super::TextModuleLifecycle.cleanup(context)?;
            super::TextModuleLifecycle.cleanup(context)?;
            context.with_existing_registered_manager::<crate::text::TextRuntimeContext>(
                super::TEXT_RUNTIME_CONTEXT_MANAGER_NAME,
                |registered| {
                    self.observations.lock().unwrap().push((
                        registered.id().get(),
                        registered.lifecycle_state(),
                        registered.create_layout_session().is_err(),
                    ));
                    Ok(())
                },
            )?;
            Ok(())
        }
    }

    let observations: Observations = Arc::new(Mutex::new(Vec::new()));
    let descriptor = module_descriptor().with_lifecycle(Arc::new(ObservedTextCleanup {
        observations: Arc::clone(&observations),
    }));
    let first = CoreRuntime::new();
    let second = CoreRuntime::new();
    first.register_module(descriptor.clone()).unwrap();
    second.register_module(descriptor).unwrap();
    first.activate_module(TEXT_MODULE_NAME).unwrap();
    second.activate_module(TEXT_MODULE_NAME).unwrap();
    let foreign = text_runtime_context_for_core(&second.handle()).unwrap();
    let foreign_id = foreign.id().get();

    let old_id = {
        let registered = text_runtime_context_for_core(&first.handle()).unwrap();
        registered.id().get()
    };
    first.deactivate_module(TEXT_MODULE_NAME).unwrap();
    assert_eq!(
        observations.lock().unwrap().as_slice(),
        &[(old_id, TextRuntimeContextLifecycleState::Closed, true)]
    );
    assert_eq!(
        foreign.lifecycle_state(),
        TextRuntimeContextLifecycleState::Active
    );

    first.activate_module(TEXT_MODULE_NAME).unwrap();
    let current_id = {
        let registered = text_runtime_context_for_core(&first.handle()).unwrap();
        registered.id().get()
    };
    assert_ne!(old_id, current_id);
    first.deactivate_module(TEXT_MODULE_NAME).unwrap();
    assert_eq!(
        observations.lock().unwrap().as_slice(),
        &[
            (old_id, TextRuntimeContextLifecycleState::Closed, true),
            (current_id, TextRuntimeContextLifecycleState::Closed, true),
        ]
    );
    assert_eq!(
        foreign.lifecycle_state(),
        TextRuntimeContextLifecycleState::Active
    );

    first.activate_module(TEXT_MODULE_NAME).unwrap();
    let replacement_id = {
        let registered = text_runtime_context_for_core(&first.handle()).unwrap();
        registered.id().get()
    };
    assert_ne!(replacement_id, old_id);
    assert_ne!(replacement_id, current_id);
    first.deactivate_module(TEXT_MODULE_NAME).unwrap();
    assert_eq!(
        observations.lock().unwrap().as_slice(),
        &[
            (old_id, TextRuntimeContextLifecycleState::Closed, true),
            (current_id, TextRuntimeContextLifecycleState::Closed, true),
            (
                replacement_id,
                TextRuntimeContextLifecycleState::Closed,
                true
            ),
        ]
    );
    assert_eq!(foreign.id().get(), foreign_id);
    assert_eq!(
        foreign.lifecycle_state(),
        TextRuntimeContextLifecycleState::Active
    );
    assert!(foreign.create_layout_session().is_ok());
    drop(foreign);
    second.deactivate_module(TEXT_MODULE_NAME).unwrap();
    assert_eq!(
        observations.lock().unwrap().last(),
        Some(&(foreign_id, TextRuntimeContextLifecycleState::Closed, true))
    );
}

#[test]
fn text_factory_cleanup_after_failed_activation_allows_retry_without_affecting_other_core() {
    use crate::core::{DependencySpec, ManagerDescriptor, ServiceKind, StartupMode};
    use crate::engine_module::{factory, qualified_name};
    use std::sync::atomic::{AtomicBool, Ordering};
    let fail_once = Arc::new(AtomicBool::new(true));
    let descriptor = module_descriptor().with_manager(ManagerDescriptor::new(
        qualified_name(TEXT_MODULE_NAME, ServiceKind::Manager, "FailOnce"),
        StartupMode::Immediate,
        vec![DependencySpec::named(qualified_name(
            TEXT_MODULE_NAME,
            ServiceKind::Manager,
            "FontServices",
        ))],
        factory(move |_| {
            if fail_once.swap(false, Ordering::AcqRel) {
                Err(crate::core::CoreError::Initialization(
                    TEXT_MODULE_NAME.to_owned(),
                    "injected factory failure".to_owned(),
                ))
            } else {
                Ok(Arc::new(()) as crate::core::runtime::ServiceObject)
            }
        }),
    ));
    let first = CoreRuntime::new();
    let second = CoreRuntime::new();
    first.register_module(descriptor.clone()).unwrap();
    second.register_module(descriptor).unwrap();
    assert!(first.activate_module(TEXT_MODULE_NAME).is_err());
    second.activate_module(TEXT_MODULE_NAME).unwrap();
    let second_context = text_runtime_context_for_core(&second.handle()).unwrap();
    first
        .activate_module(TEXT_MODULE_NAME)
        .expect("retry after compensated factory failure");
    first.deactivate_module(TEXT_MODULE_NAME).unwrap();
    assert_eq!(
        second_context.lifecycle_state(),
        TextRuntimeContextLifecycleState::Active
    );
    assert!(second_context.create_layout_session().is_ok());
    drop(second_context);
    second.deactivate_module(TEXT_MODULE_NAME).unwrap();
}
