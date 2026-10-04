use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::{CoreError, CoreResult, LifecycleState, ModuleContext, ModuleLifecycle};
use crate::diagnostic_log::DynamicProcessLogLease;

use super::super::super::profile::RuntimeDynamicSessionProfile;
use super::super::RuntimeDynamicSession;

struct FailOnceCleanupLifecycle {
    inner: Arc<dyn ModuleLifecycle>,
    cleanup_attempts: Arc<AtomicUsize>,
    cleanup_successes: Arc<AtomicUsize>,
}

impl ModuleLifecycle for FailOnceCleanupLifecycle {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        self.inner.build(context)
    }

    fn ready(&self, context: &ModuleContext) -> CoreResult<bool> {
        self.inner.ready(context)
    }

    fn finish(&self, context: &ModuleContext) -> CoreResult<()> {
        self.inner.finish(context)
    }

    fn cleanup(&self, context: &ModuleContext) -> CoreResult<()> {
        self.inner.cleanup(context)
    }

    fn cleanup_until(&self, context: &ModuleContext, deadline: Instant) -> CoreResult<()> {
        if self.cleanup_attempts.fetch_add(1, Ordering::SeqCst) == 0 {
            return Err(CoreError::MissingConfig(
                "test-only immediate module cleanup failure".to_owned(),
            ));
        }

        let result = self.inner.cleanup_until(context, deadline);
        if result.is_ok() {
            self.cleanup_successes.fetch_add(1, Ordering::SeqCst);
        }
        result
    }
}

#[test]
fn dynamic_session_shutdown_retains_core_ownership_after_immediate_module_failure() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    let handle = session.runtime.handle();
    let active_order_before = handle.active_module_shutdown_order();
    assert!(
        active_order_before.len() > 1,
        "headless session must own a dependency-bearing module and its providers"
    );

    let (target_module, direct_provider_modules) = {
        let modules = handle.inner.modules.lock().expect("test module registry");
        let target_module = active_order_before
            .iter()
            .rev()
            .find(|module_name| {
                modules
                    .get(module_name.as_str())
                    .is_some_and(|entry| !entry.descriptor.module_dependencies.is_empty())
            })
            .cloned()
            .expect("headless session must expose a dependency-bearing module");
        let direct_provider_modules = modules
            .get(&target_module)
            .expect("selected module remains registered")
            .descriptor
            .module_dependencies
            .iter()
            .map(|dependency| dependency.module_name.clone())
            .collect::<Vec<_>>();
        assert!(
            !direct_provider_modules.is_empty(),
            "selected module must retain direct provider identities"
        );
        (target_module, direct_provider_modules)
    };

    let cleanup_attempts = Arc::new(AtomicUsize::new(0));
    let cleanup_successes = Arc::new(AtomicUsize::new(0));
    {
        let mut modules = handle.inner.modules.lock().expect("test module registry");
        let entry = modules
            .get_mut(&target_module)
            .expect("selected module remains registered");
        let original = Arc::clone(&entry.descriptor.lifecycle);
        entry.descriptor.lifecycle = Arc::new(FailOnceCleanupLifecycle {
            inner: original,
            cleanup_attempts: Arc::clone(&cleanup_attempts),
            cleanup_successes: Arc::clone(&cleanup_successes),
        });
        assert_eq!(entry.lifecycle, LifecycleState::Running);
    }

    let process_log_calls = Arc::new(AtomicUsize::new(0));
    let process_log_calls_for_shutdown = Arc::clone(&process_log_calls);
    session.dynamic_process_log = Some(DynamicProcessLogLease::from_test_shutdown(
        move |_remaining| {
            process_log_calls_for_shutdown.fetch_add(1, Ordering::SeqCst);
            true
        },
    ));

    let catalog_identity = Arc::as_ptr(&session._runtime_plugin_catalog_snapshot);
    let compiled_plan_identity = Arc::as_ptr(&session._compiled_project_plugin_plan);
    let runtime_identity = Arc::as_ptr(&handle.inner);
    let task_scope_descriptor_identity = session.task_graph_scope.descriptor() as *const _;
    let task_scope_census_before = session.task_graph_scope.census();

    let first_deadline = Instant::now()
        .checked_add(Duration::from_secs(5))
        .expect("test deadline should be representable");
    assert!(!session.shutdown_before_library_unload_until(first_deadline));
    assert!(
        Instant::now() < first_deadline,
        "module failure must leave time in the shared deadline"
    );
    assert_eq!(process_log_calls.load(Ordering::SeqCst), 0);
    assert!(session.dynamic_process_log.is_some());
    assert_eq!(
        Arc::as_ptr(&session._runtime_plugin_catalog_snapshot),
        catalog_identity
    );
    assert_eq!(
        Arc::as_ptr(&session._compiled_project_plugin_plan),
        compiled_plan_identity
    );
    assert_eq!(
        Arc::as_ptr(&session.runtime.handle().inner),
        runtime_identity
    );
    assert_eq!(
        session.task_graph_scope.descriptor() as *const _,
        task_scope_descriptor_identity
    );
    let task_scope_census_after_failure = session.task_graph_scope.census();
    assert_eq!(
        task_scope_census_after_failure.owner,
        task_scope_census_before.owner
    );
    assert_eq!(
        task_scope_census_after_failure.task_capacity,
        task_scope_census_before.task_capacity
    );
    assert_eq!(
        task_scope_census_after_failure.submitted,
        task_scope_census_before.submitted
    );
    assert_eq!(
        task_scope_census_after_failure.queued,
        task_scope_census_before.queued
    );
    assert_eq!(
        task_scope_census_after_failure.running,
        task_scope_census_before.running
    );
    assert_eq!(
        task_scope_census_after_failure.completed,
        task_scope_census_before.completed
    );
    assert_eq!(
        task_scope_census_after_failure.failed,
        task_scope_census_before.failed
    );
    assert_eq!(
        task_scope_census_after_failure.cancelled,
        task_scope_census_before.cancelled
    );
    assert!(!task_scope_census_after_failure.accepting);
    assert!(task_scope_census_after_failure.is_quiescent());

    {
        let modules = handle.inner.modules.lock().expect("test module registry");
        assert_eq!(
            modules
                .get(&target_module)
                .expect("selected module remains registered")
                .lifecycle,
            LifecycleState::Stopping
        );
        assert!(
            handle
                .active_module_shutdown_order()
                .iter()
                .any(|module_name| module_name == &target_module),
            "failed module must remain in the retry ledger"
        );
        assert!(
            direct_provider_modules.iter().all(|provider_module| {
                modules
                    .get(provider_module)
                    .is_some_and(|entry| entry.lifecycle == LifecycleState::Running)
            }),
            "every direct provider of the stopping dependent must remain owned"
        );
    }

    assert_eq!(cleanup_attempts.load(Ordering::SeqCst), 1);
    assert_eq!(cleanup_successes.load(Ordering::SeqCst), 0);

    let retry_deadline = Instant::now()
        .checked_add(Duration::from_secs(5))
        .expect("test deadline should be representable");
    assert!(session.shutdown_before_library_unload_until(retry_deadline));
    assert!(session.dynamic_process_log.is_none());
    assert_eq!(process_log_calls.load(Ordering::SeqCst), 1);
    assert_eq!(cleanup_attempts.load(Ordering::SeqCst), 2);
    assert_eq!(cleanup_successes.load(Ordering::SeqCst), 1);
    assert_eq!(
        Arc::as_ptr(&session._runtime_plugin_catalog_snapshot),
        catalog_identity
    );
    assert_eq!(
        Arc::as_ptr(&session._compiled_project_plugin_plan),
        compiled_plan_identity
    );
    assert_eq!(
        Arc::as_ptr(&session.runtime.handle().inner),
        runtime_identity
    );
    assert_eq!(
        session.task_graph_scope.descriptor() as *const _,
        task_scope_descriptor_identity
    );
    let task_scope_census_after_retry = session.task_graph_scope.census();
    assert_eq!(
        task_scope_census_after_retry.owner,
        task_scope_census_before.owner
    );
    assert_eq!(
        task_scope_census_after_retry.task_capacity,
        task_scope_census_before.task_capacity
    );
    assert_eq!(
        task_scope_census_after_retry.submitted,
        task_scope_census_before.submitted
    );
    assert_eq!(
        task_scope_census_after_retry.queued,
        task_scope_census_before.queued
    );
    assert_eq!(
        task_scope_census_after_retry.running,
        task_scope_census_before.running
    );
    assert_eq!(
        task_scope_census_after_retry.completed,
        task_scope_census_before.completed
    );
    assert_eq!(
        task_scope_census_after_retry.failed,
        task_scope_census_before.failed
    );
    assert_eq!(
        task_scope_census_after_retry.cancelled,
        task_scope_census_before.cancelled
    );
    assert!(!task_scope_census_after_retry.accepting);
    assert!(task_scope_census_after_retry.is_quiescent());
    assert!(handle.active_module_shutdown_order().is_empty());
    assert!(handle
        .inner
        .modules
        .lock()
        .expect("test module registry")
        .values()
        .all(|entry| entry.lifecycle == LifecycleState::Unloaded));

    let repeat_deadline = Instant::now()
        .checked_add(Duration::from_secs(5))
        .expect("test deadline should be representable");
    assert!(session.shutdown_before_library_unload_until(repeat_deadline));
    assert_eq!(process_log_calls.load(Ordering::SeqCst), 1);
    assert_eq!(cleanup_attempts.load(Ordering::SeqCst), 2);
    assert_eq!(cleanup_successes.load(Ordering::SeqCst), 1);
    assert!(session.dynamic_process_log.is_none());
}
