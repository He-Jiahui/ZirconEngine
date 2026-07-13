use super::*;

#[test]
fn runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager() {
    let construction =
        read_runtime_src("asset/pipeline/manager/project_asset_manager/construction.rs");
    let runtime = read_runtime_src("asset/pipeline/manager/project_asset_manager/runtime.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let importer_doc = read_repo("docs/zircon_runtime/asset/importer.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
        "ProjectAssetManager poison recovery helpers",
        &runtime,
        &[
            "use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};",
            "pub(in crate::asset::pipeline::manager) fn project_read(",
            "pub(in crate::asset::pipeline::manager) fn project_write(",
            "pub(in crate::asset::pipeline::manager) fn importer_registry_read(",
            "pub(in crate::asset::pipeline::manager) fn importer_registry_write(",
            "pub(in crate::asset::pipeline::manager) fn lock_change_subscribers(",
            "MutexGuard<'_, Vec<ChannelSender<AssetChange>>>",
            "pub(in crate::asset::pipeline::manager) fn lock_watch_error_subscribers(",
            "MutexGuard<'_, Vec<ChannelSender<AssetWatchError>>>",
            "fn lock_watchers(&self) -> MutexGuard<'_, Vec<AssetWatcher>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "project_asset_manager_runtime_accessors_recover_poisoned_locks",
        ],
    );
    assert_contains_all(
        "ProjectAssetManager construction uses importer lock helpers",
        &construction,
        &[
            "self.importer_registry_write()",
            "self.importer_registry_read().importers()",
        ],
    );

    for (label, source) in [
        ("project asset manager construction", construction.as_str()),
        ("project asset manager runtime", runtime.as_str()),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
        assert!(
            !production_section(source).contains(".read().expect(")
                && !production_section(source).contains(".write().expect(")
                && !production_section(source).contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("asset importer doc", importer_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset project manager lock poison recovery",
                "runtime_15_asset_project_manager_lock_poison_recovery_static_passed_cargo_deferred",
                "asset/pipeline/manager/project_asset_manager/runtime.rs",
                "asset/pipeline/manager/project_asset_manager/construction.rs",
                "project_asset_manager_runtime_accessors_recover_poisoned_locks",
                "runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager",
            ],
        );
    }
}

#[test]
fn runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool() {
    let worker_pool = read_runtime_src("asset/pipeline/worker_pool.rs");
    let service_contract =
        read_runtime_src("asset/pipeline/manager/service_contracts/asset_manager_contract.rs");
    let project_runtime =
        read_runtime_src("asset/pipeline/manager/project_asset_manager/runtime.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let worker_doc = read_repo("docs/zircon_runtime/asset/worker_pool.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
        "AssetWorkerPool poison recovery helpers",
        &worker_pool,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_in_flight(&self) -> MutexGuard<'_, HashMap<AssetRequest, usize>>",
            "fn lock_diagnostics(&self) -> MutexGuard<'_, AssetWorkerPoolDiagnostics>",
            "fn lock_in_flight_map(",
            "fn lock_worker_diagnostics(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "asset_worker_pool_accessors_recover_poisoned_locks",
        ],
    );
    assert_contains_all(
        "ProjectAssetManager service contract uses shared lock helpers",
        &service_contract,
        &[
            "let installed_importers = self.importer_registry_read().clone();",
            "self.lock_change_subscribers().push(sender);",
            "self.lock_watch_error_subscribers().push(sender);",
        ],
    );
    assert_contains_all(
        "ProjectAssetManager runtime exposes subscriber helpers inside manager owner",
        &project_runtime,
        &[
            "pub(in crate::asset::pipeline::manager) fn lock_change_subscribers(",
            "pub(in crate::asset::pipeline::manager) fn lock_watch_error_subscribers(",
        ],
    );

    let worker_pool_production = worker_pool
        .split("\n#[cfg(test)]\nmod tests")
        .next()
        .unwrap_or(worker_pool.as_str());
    for (label, source) in [
        ("asset worker pool", worker_pool_production),
        ("asset manager service contract", service_contract.as_str()),
        (
            "project asset manager runtime",
            production_section(&project_runtime),
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

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("asset worker doc", worker_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset worker pool lock poison recovery",
                "runtime_15_asset_worker_pool_lock_poison_recovery_static_passed_cargo_deferred",
                "asset/pipeline/worker_pool.rs",
                "asset/pipeline/manager/service_contracts/asset_manager_contract.rs",
                "asset_worker_pool_accessors_recover_poisoned_locks",
                "runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool",
            ],
        );
    }
}
