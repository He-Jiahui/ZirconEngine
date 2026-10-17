use super::*;

// TODO: [CR-RUNTIME-TESTS-STRUCT-0012] 项目资源管理器的锁守卫类型及恢复测试文本未匹配；尚缺迁移后恢复入口的行为证据，需追踪访问器与对应中毒恢复测试再更新断言。
#[test]
fn runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager() {
    let construction = read_runtime_src("asset/pipeline/manager/assets/construction.rs");
    let runtime = read_runtime_src("asset/pipeline/manager/assets/runtime.rs");
    let asset_owner = read_runtime_src("asset/pipeline/manager/assets/assets.rs");
    let event_stream = read_runtime_src("asset/watch/event_stream.rs");

    assert_contains_all(
        "ProjectAssetManager poison recovery helpers",
        &runtime,
        &[
            "use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard, TryLockError};",
            "pub(in crate::asset::pipeline::manager) fn project_read(",
            "pub(in crate::asset::pipeline::manager) fn project_write(",
            "pub(in crate::asset::pipeline::manager) fn importer_registry_read(",
            "pub(in crate::asset::pipeline::manager) fn importer_registry_write(",
            "fn broadcast(&self, changes: Vec<AssetChange>) -> Vec<ChannelWakeCallback>",
            ".publish(changes, crate::asset::watch::change_bytes)",
            "pub(in crate::asset::pipeline::manager) fn subscribe_asset_changes_internal(",
            "self.change_events.subscribe(wake)",
            "fn lock_watchers(&self) -> MutexGuard<'_, Vec<AssetWatcher>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        ],
    );
    assert_contains_all(
        "asset event stream typed owners",
        &asset_owner,
        &[
            "crate::asset::watch::AssetEventLog<AssetChange>",
            "crate::asset::watch::AssetEventLog<AssetWatchError>",
        ],
    );
    assert_contains_all(
        "asset event stream poison recovery and cursor registration",
        &event_stream,
        &[
            "pub(crate) struct AssetEventLog<T>",
            "state: Mutex<State<T>>",
            "pub(crate) fn subscribe(&self, wake: Option<ChannelWakeCallback>)",
            "pub(crate) fn publish(",
            "let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());",
            "state.cursors.push(Arc::downgrade(&cursor));",
        ],
    );
    assert_contains_all(
        "ProjectAssetManager construction uses importer lock helpers",
        &construction,
        &[
            "self.importer_registry_write()",
            "fn active_importer_registry(&self) -> AssetImporterRegistry",
            "self.importer_registry_read().clone()",
        ],
    );

    for (label, source) in [
        ("project asset manager construction", construction.as_str()),
        ("project asset manager runtime", runtime.as_str()),
        ("asset event stream", event_stream.as_str()),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
        let production = production_section(source);
        assert!(
            !production.contains(".read().expect(")
                && !production.contains(".write().expect(")
                && !production.contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }
    let recovery_tests = read_runtime_src("asset/pipeline/manager/assets/runtime/tests/cases.rs");
    assert_contains_all(
        "mounted poison recovery regression",
        &recovery_tests,
        &["fn project_asset_manager_runtime_accessors_recover_poisoned_locks("],
    );
}

// TODO: [CR-RUNTIME-TESTS-STRUCT-0013] 资源工作池的进行中任务表与诊断锁访问器未匹配旧签名；需核对并发计数和中毒恢复的现行调用及行为覆盖，避免仅按名称补锚点。
#[test]
fn runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool() {
    let worker_pool = read_runtime_src("asset/pipeline/worker_pool.rs");
    let completion = read_runtime_src("asset/pipeline/worker_pool/completion.rs");
    let worker_tests = read_runtime_src("asset/pipeline/worker_pool/tests/cases.rs");
    let service_contract = read_runtime_src("asset/pipeline/manager/assets/open_project.rs");
    let project_runtime = read_runtime_src("asset/pipeline/manager/assets/runtime.rs");
    let asset_owner = read_runtime_src("asset/pipeline/manager/assets/assets.rs");
    let event_stream = read_runtime_src("asset/watch/event_stream.rs");

    assert_contains_all(
        "AssetWorkerPool poison recovery helpers",
        &worker_pool,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "lock_completion_registry(&self.completions)",
            "fn lock_diagnostics(&self) -> MutexGuard<'_, AssetWorkerPoolDiagnostics>",
            "lock_worker_diagnostics(&self.diagnostics)",
        ],
    );
    assert_contains_all(
        "ProjectAssetManager service contract uses shared lock helpers",
        &service_contract,
        &[
            "let installed_importers = self.importer_registry_read().clone();",
            "*project.importer_mut().registry_mut() = installed_importers;",
        ],
    );
    assert_contains_all(
        "ProjectAssetManager runtime exposes subscriber helpers inside manager owner",
        &project_runtime,
        &[
            "pub(in crate::asset::pipeline::manager) fn subscribe_asset_changes_internal(",
            "self.change_events.subscribe(wake)",
            "fn broadcast(&self, changes: Vec<AssetChange>) -> Vec<ChannelWakeCallback>",
            ".publish(changes, crate::asset::watch::change_bytes)",
            "pub(in crate::asset::pipeline::manager) fn broadcast_watch_error(",
            ".publish(std::iter::once(error), crate::asset::watch::error_bytes)",
        ],
    );
    assert_contains_all(
        "asset event stream typed owners",
        &asset_owner,
        &[
            "crate::asset::watch::AssetEventLog<AssetChange>",
            "crate::asset::watch::AssetEventLog<AssetWatchError>",
        ],
    );
    assert_contains_all(
        "asset event stream poison recovery and cursor registration",
        &event_stream,
        &[
            "pub(crate) struct AssetEventLog<T>",
            "state: Mutex<State<T>>",
            "pub(crate) fn subscribe(&self, wake: Option<ChannelWakeCallback>)",
            "pub(crate) fn publish(",
            "let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());",
            "state.cursors.push(Arc::downgrade(&cursor));",
        ],
    );
    assert_contains_all(
        "worker completion registry poison recovery",
        &completion,
        &[
            "fn lock_completion_registry(",
            "MutexGuard<'_, CompletionRegistryState>",
            "fn lock_worker_diagnostics(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        ],
    );
    assert_contains_all(
        "worker completion registry poison behavior",
        &worker_tests,
        &[
            "fn asset_worker_pool_accessors_recover_poisoned_locks(",
            "pool.completions.state.lock().unwrap()",
            "pool.diagnostics.lock().unwrap()",
            "ticket.wait_timeout(Duration::from_secs(2))",
            "pool.diagnostics().in_flight",
        ],
    );

    let worker_pool_production = production_section(&worker_pool);
    let project_runtime_production = production_section(&project_runtime);
    for (label, source) in [
        ("asset worker pool", worker_pool_production.as_str()),
        (
            "asset event stream",
            production_section(&event_stream).as_str(),
        ),
        (
            "worker completion registry",
            production_section(&completion).as_str(),
        ),
        ("asset manager service contract", service_contract.as_str()),
        (
            "project asset manager runtime",
            project_runtime_production.as_str(),
        ),
    ] {
        assert!(
            !source.contains(LOCK_UNWRAP_CALL)
                && !source.contains(".read().expect(")
                && !source.contains(".write().expect(")
                && !source.contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }
}
