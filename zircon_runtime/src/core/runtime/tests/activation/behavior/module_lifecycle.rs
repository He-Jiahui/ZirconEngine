use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::super::super::super::*;
use super::super::super::fixtures::TestDriver;
use crate::core::runtime::ServiceObject;
use crate::core::CoreResult;
use crate::core::{
    CoreError, LifecycleState, RuntimeModuleLifecycleBlock, RuntimeModuleLifecycleObserver,
    ServiceKind, StartupMode,
};

// 此 fixture 把生命周期回调记成带模块名的有序轨迹；ready_after 控制前几次 ready 返回 false，fail_* 开关注入错误。
// *_once 计数器只让首次回调失败，以便在同一模块上继续观察清理重试。
#[derive(Debug)]
struct RecordingLifecycle {
    calls: Arc<Mutex<Vec<String>>>,
    ready_after: usize,
    ready_calls: AtomicUsize,
    fail_finish: bool,
    fail_finish_once: bool,
    finish_calls: AtomicUsize,
    fail_cleanup: bool,
    fail_cleanup_once: bool,
    cleanup_calls: AtomicUsize,
}

impl RecordingLifecycle {
    fn new(calls: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            calls,
            ready_after: 0,
            ready_calls: AtomicUsize::new(0),
            fail_finish: false,
            fail_finish_once: false,
            finish_calls: AtomicUsize::new(0),
            fail_cleanup: false,
            fail_cleanup_once: false,
            cleanup_calls: AtomicUsize::new(0),
        }
    }

    fn ready_after(mut self, ready_after: usize) -> Self {
        self.ready_after = ready_after;
        self
    }

    fn fail_finish(mut self) -> Self {
        self.fail_finish = true;
        self
    }

    fn fail_finish_once(mut self) -> Self {
        self.fail_finish_once = true;
        self
    }

    fn fail_cleanup(mut self) -> Self {
        self.fail_cleanup = true;
        self
    }

    fn fail_cleanup_once(mut self) -> Self {
        self.fail_cleanup_once = true;
        self
    }

    fn record(&self, call: &'static str, context: &ModuleContext) {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{}:{}", context.module_name, call));
    }
}

// 这些 trait 钩子与激活执行器实际调用的 build、ready、finish、cleanup 对齐；先记日志再返回，轨迹也暴露失败点。
impl ModuleLifecycle for RecordingLifecycle {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        self.record("build", context);
        Ok(())
    }

    fn ready(&self, context: &ModuleContext) -> CoreResult<bool> {
        self.record("ready", context);
        let call_index = self.ready_calls.fetch_add(1, Ordering::SeqCst);
        Ok(call_index >= self.ready_after)
    }

    fn finish(&self, context: &ModuleContext) -> CoreResult<()> {
        self.record("finish", context);
        if self.fail_finish
            || (self.fail_finish_once && self.finish_calls.fetch_add(1, Ordering::SeqCst) == 0)
        {
            return Err(CoreError::MissingConfig("module.finish".to_owned()));
        }
        Ok(())
    }

    fn cleanup(&self, context: &ModuleContext) -> CoreResult<()> {
        self.record("cleanup", context);
        if self.fail_cleanup
            || (self.fail_cleanup_once && self.cleanup_calls.fetch_add(1, Ordering::SeqCst) == 0)
        {
            return Err(CoreError::MissingConfig("module.cleanup".to_owned()));
        }
        Ok(())
    }
}

// activated 回调在模块已置为 Running 并加入关闭顺序后 panic；deactivating 放行，隔离验证激活通知失败的回滚。
#[derive(Debug)]
struct PanickingActivationObserver;

impl RuntimeModuleLifecycleObserver for PanickingActivationObserver {
    fn runtime_module_activated(&self, _module_name: &str) {
        panic!("activation observer failed");
    }

    fn runtime_module_deactivating(
        &self,
        _module_name: &str,
    ) -> Result<(), RuntimeModuleLifecycleBlock> {
        Ok(())
    }
}

// 覆盖带绝对 deadline 的 cleanup_until：release 前持续等待，到期返回运行时识别的清理超时，release 后才成功。
#[derive(Debug)]
struct DeadlineCleanupLifecycle {
    released: Arc<std::sync::atomic::AtomicBool>,
}

impl ModuleLifecycle for DeadlineCleanupLifecycle {
    fn cleanup_until(&self, context: &ModuleContext, deadline: Instant) -> CoreResult<()> {
        while !self.released.load(Ordering::Acquire) {
            if Instant::now() >= deadline {
                return Err(CoreError::ModuleCleanupTimeout {
                    module: context.module_name.clone(),
                    operation: "test_deadline_cleanup".to_owned(),
                    budget: Duration::ZERO,
                    incomplete_entries: 1,
                    failed: 0,
                    cancelled: 0,
                });
            }
            std::thread::yield_now();
        }
        Ok(())
    }
}

fn recorded_calls(calls: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    calls.lock().unwrap().clone()
}

fn expected_calls(calls: &[&str]) -> Vec<String> {
    calls.iter().map(|call| (*call).to_owned()).collect()
}

// 模块从 Registered 经 CoreRuntime 公共入口激活再卸载；默认 ready 立即成功，轨迹必须为 build、ready、finish、cleanup。
// 卸载后模块描述仍在注册表中，生命周期状态转为 Unloaded。
#[test]
fn module_lifecycle_hooks_wrap_activation_and_deactivation() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle = Arc::new(RecordingLifecycle::new(Arc::clone(&calls)));

    runtime
        .register_module(
            ModuleDescriptor::new("LifecycleModule", "lifecycle hooks").with_lifecycle(lifecycle),
        )
        .unwrap();

    runtime.activate_module("LifecycleModule").unwrap();
    runtime.deactivate_module("LifecycleModule").unwrap();

    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "LifecycleModule:build",
            "LifecycleModule:ready",
            "LifecycleModule:finish",
            "LifecycleModule:cleanup",
        ])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("LifecycleModule")
        .expect("module should stay registered after deactivation");
    assert_eq!(module.lifecycle, LifecycleState::Unloaded);
}

// 先激活到 Running，再以 25ms drain budget 调用真实 deactivation；受控 cleanup 超时后必须保留 Stopping 与关闭顺序。
// 释放回调后用新 deadline 重试同一模块，完成清理并卸载。
#[test]
fn astra_life_a4_module_cleanup_deadline_retains_stopping_module_for_retry() {
    let runtime = CoreRuntime::new();
    let released = Arc::new(std::sync::atomic::AtomicBool::new(false));
    runtime
        .register_module(
            ModuleDescriptor::new("DeadlineCleanupModule", "deadline cleanup").with_lifecycle(
                Arc::new(DeadlineCleanupLifecycle {
                    released: Arc::clone(&released),
                }),
            ),
        )
        .unwrap();
    runtime.activate_module("DeadlineCleanupModule").unwrap();

    let started_at = Instant::now();
    assert!(matches!(
        runtime.deactivate_module_with_drain_timeout(
            "DeadlineCleanupModule",
            Duration::from_millis(25)
        ),
        Err(CoreError::ModuleCleanupTimeout { .. })
    ));
    assert!(started_at.elapsed() < Duration::from_secs(1));
    assert_eq!(
        runtime
            .handle()
            .inner
            .modules
            .lock()
            .unwrap()
            .get("DeadlineCleanupModule")
            .unwrap()
            .lifecycle,
        LifecycleState::Stopping
    );

    released.store(true, Ordering::Release);
    runtime
        .deactivate_module_with_drain_timeout("DeadlineCleanupModule", Duration::from_secs(1))
        .unwrap();
}

// ready_after(1) 让第一次 ready 返回 false、下一次返回 true；CoreRuntime 的轮询成功后才应调用 finish。
// 日志中的两次 ready 保证测试覆盖了等待路径，而非默认就绪快路。
#[test]
fn module_ready_polling_allows_later_ready_result() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle = Arc::new(RecordingLifecycle::new(Arc::clone(&calls)).ready_after(1));

    runtime
        .register_module(
            ModuleDescriptor::new("PollingReadyModule", "poll ready").with_lifecycle(lifecycle),
        )
        .unwrap();

    runtime
        .activate_module_with_ready_timeout("PollingReadyModule", Duration::from_millis(50))
        .unwrap();

    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "PollingReadyModule:build",
            "PollingReadyModule:ready",
            "PollingReadyModule:ready",
            "PollingReadyModule:finish",
        ])
    );
}

// Immediate Driver 在等待 ready 前已解析；ready_after(usize::MAX) 配合零预算使首次检查失败并立即超时。
// 断言 rollback 调用 cleanup，且模块与已启动服务都回到 Registered、服务实例被清空。
#[derive(Debug)]
struct SharedReadyBudgetLifecycle {
    ready_calls: Arc<AtomicUsize>,
    first_ready_delay: Duration,
    ready_result: bool,
}

impl ModuleLifecycle for SharedReadyBudgetLifecycle {
    fn ready(&self, _context: &ModuleContext) -> CoreResult<bool> {
        let call = self.ready_calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 && !self.first_ready_delay.is_zero() {
            std::thread::sleep(self.first_ready_delay);
        }
        Ok(self.ready_result)
    }
}

// The first module consumes the shared budget but returns ready; the second
// module receives its initial probe only, with no fresh per-module wait window.
#[test]
fn batch_ready_timeout_is_shared_across_modules() {
    let runtime = CoreRuntime::new();
    let first_calls = Arc::new(AtomicUsize::new(0));
    let second_calls = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("SharedReadyBudgetFirst", "shared ready budget first")
                .with_init_level(InitLevel::Kernel)
                .with_lifecycle(Arc::new(SharedReadyBudgetLifecycle {
                    ready_calls: Arc::clone(&first_calls),
                    first_ready_delay: Duration::from_millis(30),
                    ready_result: true,
                })),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("SharedReadyBudgetSecond", "shared ready budget second")
                .with_init_level(InitLevel::Post)
                .with_lifecycle(Arc::new(SharedReadyBudgetLifecycle {
                    ready_calls: Arc::clone(&second_calls),
                    first_ready_delay: Duration::ZERO,
                    ready_result: false,
                })),
        )
        .unwrap();

    assert!(matches!(
        runtime
            .activate_registered_modules_with_ready_timeout(Duration::from_millis(10))
            .unwrap_err(),
        CoreError::ModuleBatchActivationRollback { .. }
    ));
    assert_eq!(first_calls.load(Ordering::SeqCst), 1);
    assert_eq!(second_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn batch_ready_failure_does_not_probe_later_modules() {
    let runtime = CoreRuntime::new();
    let first_calls = Arc::new(AtomicUsize::new(0));
    let second_calls = Arc::new(AtomicUsize::new(0));
    runtime
        .register_module(
            ModuleDescriptor::new("SharedReadyFailureFirst", "shared ready failure first")
                .with_init_level(InitLevel::Kernel)
                .with_lifecycle(Arc::new(SharedReadyBudgetLifecycle {
                    ready_calls: Arc::clone(&first_calls),
                    first_ready_delay: Duration::from_millis(30),
                    ready_result: false,
                })),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("SharedReadyFailureSecond", "shared ready failure second")
                .with_init_level(InitLevel::Post)
                .with_lifecycle(Arc::new(SharedReadyBudgetLifecycle {
                    ready_calls: Arc::clone(&second_calls),
                    first_ready_delay: Duration::ZERO,
                    ready_result: false,
                })),
        )
        .unwrap();

    assert!(matches!(
        runtime
            .activate_registered_modules_with_ready_timeout(Duration::from_millis(10))
            .unwrap_err(),
        CoreError::ModuleBatchActivationRollback { .. }
    ));
    assert_eq!(first_calls.load(Ordering::SeqCst), 1);
    assert_eq!(second_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn module_ready_timeout_resets_module_and_started_services() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle = Arc::new(RecordingLifecycle::new(Arc::clone(&calls)).ready_after(usize::MAX));
    let service_name =
        RegistryName::from_parts("TimeoutModule", ServiceKind::Driver, "ImmediateDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("TimeoutModule", "ready timeout")
                .with_lifecycle(lifecycle)
                .with_driver(DriverDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                )),
        )
        .unwrap();

    let error = runtime
        .activate_module_with_ready_timeout("TimeoutModule", Duration::ZERO)
        .unwrap_err();
    assert!(matches!(
        error,
        CoreError::ModuleReadyTimeout { module, budget }
            if module == "TimeoutModule" && budget == Duration::ZERO
    ));
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "TimeoutModule:build",
            "TimeoutModule:ready",
            "TimeoutModule:cleanup",
        ])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("TimeoutModule")
        .expect("timed-out module should remain registered");
    assert_eq!(module.lifecycle, LifecycleState::Registered);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    let service = services
        .get(service_name.as_str())
        .expect("timed-out module should keep service entry");
    assert_eq!(service.lifecycle, LifecycleState::Registered);
    assert!(service.instance.is_none());
}

// Immediate Driver 已随模块启动，随后 finish 返回 MissingConfig；成功 cleanup 应保留原始错误并复位模块和服务。
// 轨迹确认失败发生在 build、ready 之后，cleanup 则是失败激活的回滚回调。
#[test]
fn module_finish_error_resets_module_and_started_services() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle = Arc::new(RecordingLifecycle::new(Arc::clone(&calls)).fail_finish());
    let service_name =
        RegistryName::from_parts("FinishErrorModule", ServiceKind::Driver, "ImmediateDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("FinishErrorModule", "finish error")
                .with_lifecycle(lifecycle)
                .with_driver(DriverDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                )),
        )
        .unwrap();

    let error = runtime.activate_module("FinishErrorModule").unwrap_err();
    assert!(matches!(
        error,
        CoreError::MissingConfig(key) if key == "module.finish"
    ));
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "FinishErrorModule:build",
            "FinishErrorModule:ready",
            "FinishErrorModule:finish",
            "FinishErrorModule:cleanup",
        ])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("FinishErrorModule")
        .expect("failed module should remain registered");
    assert_eq!(module.lifecycle, LifecycleState::Registered);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    let service = services
        .get(service_name.as_str())
        .expect("failed module should keep service entry");
    assert_eq!(service.lifecycle, LifecycleState::Registered);
    assert!(service.instance.is_none());
}

// 同一次激活中 finish 与 rollback cleanup 都失败；ModuleActivationRollback 必须同时携带 activation 主错和 cleanup 错。
// 有序日志确认 cleanup 在 finish 失败后确实执行。
#[test]
fn module_activation_reports_primary_and_cleanup_errors() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle = Arc::new(
        RecordingLifecycle::new(Arc::clone(&calls))
            .fail_finish()
            .fail_cleanup(),
    );

    runtime
        .register_module(
            ModuleDescriptor::new("RollbackErrorModule", "rollback error")
                .with_lifecycle(lifecycle),
        )
        .unwrap();

    let error = runtime.activate_module("RollbackErrorModule").unwrap_err();
    assert!(matches!(
        error,
        CoreError::ModuleActivationRollback { activation, cleanup }
            if matches!(*activation, CoreError::MissingConfig(ref key) if key == "module.finish")
                && matches!(*cleanup, CoreError::MissingConfig(ref key) if key == "module.cleanup")
    ));
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "RollbackErrorModule:build",
            "RollbackErrorModule:ready",
            "RollbackErrorModule:finish",
            "RollbackErrorModule:cleanup",
        ])
    );
}

// Kernel、Services、Scene 通过 InitLevel 和 Scene 对 Services 的依赖形成冻结图顺序。
// 批量 API 先完成全体 build，再轮询全体 ready，最后才对全体 finish；状态断言验证三者均 Running。
#[test]
fn activate_registered_modules_finishes_only_after_all_modules_are_ready() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));

    runtime
        .register_module(
            ModuleDescriptor::new("SceneModule", "scene")
                .with_init_level(InitLevel::Scene)
                .with_module_dependency(ModuleDependencySpec::named("ServicesModule"))
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls)))),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("KernelModule", "kernel")
                .with_init_level(InitLevel::Kernel)
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls)))),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("ServicesModule", "services")
                .with_init_level(InitLevel::Services)
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls)))),
        )
        .unwrap();

    runtime.activate_registered_modules().unwrap();

    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "KernelModule:build",
            "ServicesModule:build",
            "SceneModule:build",
            "KernelModule:ready",
            "ServicesModule:ready",
            "SceneModule:ready",
            "KernelModule:finish",
            "ServicesModule:finish",
            "SceneModule:finish",
        ])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    for module_name in ["KernelModule", "ServicesModule", "SceneModule"] {
        let module = modules
            .get(module_name)
            .expect("batch activation should keep every module registered");
        assert_eq!(module.lifecycle, LifecycleState::Running);
    }
}

// First 依赖先于 Second，且二者的 Immediate Driver 已启动；Second finish 失败发生在整批 ready 之后。
// 批处理须按反向模块顺序 cleanup，并把两个模块与服务槽位复位到 Registered、无实例。
#[test]
fn activate_registered_modules_rolls_back_all_started_modules_on_finish_error() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let first_service_name =
        RegistryName::from_parts("FirstBatchModule", ServiceKind::Driver, "ImmediateDriver");
    let second_service_name =
        RegistryName::from_parts("SecondBatchModule", ServiceKind::Driver, "ImmediateDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("FirstBatchModule", "first")
                .with_init_level(InitLevel::Kernel)
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls))))
                .with_driver(DriverDescriptor::new(
                    first_service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("SecondBatchModule", "second")
                .with_init_level(InitLevel::Services)
                .with_module_dependency(ModuleDependencySpec::named("FirstBatchModule"))
                .with_lifecycle(Arc::new(
                    RecordingLifecycle::new(Arc::clone(&calls)).fail_finish(),
                ))
                .with_driver(DriverDescriptor::new(
                    second_service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                )),
        )
        .unwrap();

    let error = runtime.activate_registered_modules().unwrap_err();
    assert!(matches!(
        error,
        CoreError::MissingConfig(key) if key == "module.finish"
    ));
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "FirstBatchModule:build",
            "SecondBatchModule:build",
            "FirstBatchModule:ready",
            "SecondBatchModule:ready",
            "FirstBatchModule:finish",
            "SecondBatchModule:finish",
            "SecondBatchModule:cleanup",
            "FirstBatchModule:cleanup",
        ])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    for module_name in ["FirstBatchModule", "SecondBatchModule"] {
        let module = modules
            .get(module_name)
            .expect("failed batch activation should keep every module registered");
        assert_eq!(module.lifecycle, LifecycleState::Registered);
    }
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [&first_service_name, &second_service_name] {
        let service = services
            .get(service_name.as_str())
            .expect("failed batch activation should keep service entries");
        assert_eq!(service.lifecycle, LifecycleState::Registered);
        assert!(service.instance.is_none());
    }
}

// 两个模块的 cleanup 都被注入失败，Second 的 finish 是激活主错；错误值仍须保留主错及反向顺序的两项清理错误。
// 日志同时核对回滚实际先清理 Second、再清理 First。
#[test]
fn batch_activation_cleanup_failures_keep_reverse_order_and_primary_error() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));

    runtime
        .register_module(
            ModuleDescriptor::new("FirstRollbackModule", "first")
                .with_init_level(InitLevel::Kernel)
                .with_lifecycle(Arc::new(
                    RecordingLifecycle::new(Arc::clone(&calls)).fail_cleanup(),
                )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("SecondRollbackModule", "second")
                .with_init_level(InitLevel::Services)
                .with_module_dependency(ModuleDependencySpec::named("FirstRollbackModule"))
                .with_lifecycle(Arc::new(
                    RecordingLifecycle::new(Arc::clone(&calls))
                        .fail_finish()
                        .fail_cleanup(),
                )),
        )
        .unwrap();

    let error = runtime.activate_registered_modules().unwrap_err();
    match error {
        CoreError::ModuleBatchActivationRollback {
            activation,
            cleanup_failures,
        } => {
            assert!(matches!(
                *activation,
                CoreError::MissingConfig(ref key) if key == "module.finish"
            ));
            assert_eq!(
                cleanup_failures
                    .iter()
                    .map(|(module, _)| module.as_str())
                    .collect::<Vec<_>>(),
                vec!["SecondRollbackModule", "FirstRollbackModule"]
            );
            assert!(cleanup_failures.iter().all(|(_, error)| matches!(
                error,
                CoreError::MissingConfig(key) if key == "module.cleanup"
            )));
        }
        other => panic!("expected typed batch rollback error, found {other:?}"),
    }
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "FirstRollbackModule:build",
            "SecondRollbackModule:build",
            "FirstRollbackModule:ready",
            "SecondRollbackModule:ready",
            "FirstRollbackModule:finish",
            "SecondRollbackModule:finish",
            "SecondRollbackModule:cleanup",
            "FirstRollbackModule:cleanup",
        ])
    );
}

// 首次 finish 和 cleanup 各失败一次；回滚失败让模块留在 Stopping 关闭账本中，服务实例还在但 admission 已关闭。
// 显式拒绝重新激活后由 shutdown 重试 cleanup；成功卸载后再次激活应能构建第二次。
#[test]
fn failed_single_activation_cleanup_stays_stopping_until_shutdown_retries_it() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let service_name = RegistryName::from_parts(
        "RetryableRollbackModule",
        ServiceKind::Driver,
        "ImmediateDriver",
    );
    runtime
        .register_module(
            ModuleDescriptor::new("RetryableRollbackModule", "retryable rollback")
                .with_lifecycle(Arc::new(
                    RecordingLifecycle::new(Arc::clone(&calls))
                        .fail_finish_once()
                        .fail_cleanup_once(),
                ))
                .with_driver(DriverDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                )),
        )
        .unwrap();

    let error = runtime
        .activate_module("RetryableRollbackModule")
        .unwrap_err();
    assert!(matches!(error, CoreError::ModuleActivationRollback { .. }));
    let handle = runtime.handle();
    assert_eq!(
        handle
            .inner
            .modules
            .lock()
            .unwrap()
            .get("RetryableRollbackModule")
            .unwrap()
            .lifecycle,
        LifecycleState::Stopping
    );
    assert_eq!(
        handle.active_module_shutdown_order(),
        vec!["RetryableRollbackModule"]
    );
    {
        let services = handle.inner.services.lock().unwrap();
        let service = services.get(service_name.as_str()).unwrap();
        assert_eq!(service.lifecycle, LifecycleState::Running);
        assert!(service.instance.is_some());
        assert!(!service.admission_open);
    }
    assert!(matches!(
        runtime.activate_module("RetryableRollbackModule"),
        Err(CoreError::InvalidModuleLifecycleTransition {
            state: LifecycleState::Stopping,
            ..
        })
    ));
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "RetryableRollbackModule:build",
            "RetryableRollbackModule:ready",
            "RetryableRollbackModule:finish",
            "RetryableRollbackModule:cleanup",
        ])
    );

    runtime
        .shutdown_registered_modules_with_drain_timeout(Duration::from_secs(1))
        .unwrap();
    assert_eq!(
        handle
            .inner
            .modules
            .lock()
            .unwrap()
            .get("RetryableRollbackModule")
            .unwrap()
            .lifecycle,
        LifecycleState::Unloaded
    );
    assert!(handle.active_module_shutdown_order().is_empty());
    assert_eq!(
        recorded_calls(&calls)
            .iter()
            .filter(|call| call.ends_with(":cleanup"))
            .count(),
        2
    );
    runtime.activate_module("RetryableRollbackModule").unwrap();
    assert_eq!(
        recorded_calls(&calls)
            .iter()
            .filter(|call| call.ends_with(":build"))
            .count(),
        2
    );
}

// observer 在激活钩子完成后 panic；公共激活路径应转成类型化错误并执行 rollback cleanup。
// 清理成功后模块回到 Registered 且不留关闭账本项；移除 observer 后再次激活验证入口恢复。
#[test]
fn activation_observer_panic_restores_module_and_shutdown_ledger() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    runtime
        .register_module(
            ModuleDescriptor::new("ObserverPanicModule", "observer panic")
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls)))),
        )
        .unwrap();
    runtime.install_runtime_module_lifecycle_observer(Arc::new(PanickingActivationObserver));

    assert!(matches!(
        runtime.activate_module("ObserverPanicModule"),
        Err(CoreError::ModuleLifecycleCallbackPanicked { .. })
    ));
    let handle = runtime.handle();
    assert_eq!(
        handle
            .inner
            .modules
            .lock()
            .unwrap()
            .get("ObserverPanicModule")
            .unwrap()
            .lifecycle,
        LifecycleState::Registered
    );
    assert!(handle.active_module_shutdown_order().is_empty());
    runtime
        .shutdown_registered_modules_with_drain_timeout(Duration::from_secs(1))
        .unwrap();
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "ObserverPanicModule:build",
            "ObserverPanicModule:ready",
            "ObserverPanicModule:finish",
            "ObserverPanicModule:cleanup",
        ])
    );
    handle.clear_runtime_module_lifecycle_observer();
    runtime.activate_module("ObserverPanicModule").unwrap();
    assert_eq!(
        handle.active_module_shutdown_order(),
        vec!["ObserverPanicModule"]
    );
}

// 先正常激活，再直接向 CoreHandle 传入已过期的绝对 deadline；入口应在 lifecycle 转换前返回 cleanup timeout。
// 没有 cleanup 轨迹且状态仍为 Running，证明本次调用未提交停止状态。
#[test]
fn expired_absolute_deactivation_deadline_does_not_start_cleanup() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    runtime
        .register_module(
            ModuleDescriptor::new("ExpiredDeadlineModule", "expired shutdown deadline")
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls)))),
        )
        .unwrap();
    runtime.activate_module("ExpiredDeadlineModule").unwrap();

    let deadline = Instant::now() - Duration::from_millis(1);
    assert!(matches!(
        runtime
            .handle()
            .deactivate_module_until("ExpiredDeadlineModule", deadline),
        Err(CoreError::ModuleCleanupTimeout { .. })
    ));
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "ExpiredDeadlineModule:build",
            "ExpiredDeadlineModule:ready",
            "ExpiredDeadlineModule:finish",
        ])
    );
    assert_eq!(
        runtime
            .handle()
            .inner
            .modules
            .lock()
            .unwrap()
            .get("ExpiredDeadlineModule")
            .unwrap()
            .lifecycle,
        LifecycleState::Running
    );
}

mod shutdown;
