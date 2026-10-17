use super::*;

// TODO: [CR-RUNTIME-TESTS-STRUCT-0023] 内置导航管理器的状态锁访问未匹配旧导入和读取表达式；需追踪现行导航查询入口及中毒状态恢复测试，再更新结构期待。
#[test]
fn runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager() {
    let navigation_runtime = read_runtime_src("navigation/runtime.rs");
    let navigation_tests = read_runtime_src("navigation/runtime/tests/cases.rs");
    let _runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let _runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let _review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let _structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let _module_doc = read_repo("docs/crates/zircon_runtime/structure/module-convention.md");
    let _navigation_doc = read_repo("docs/crates/zircon_runtime/navigation/runtime.md");

    assert_contains_all(
        "BuiltinNavigationManager poison recovery helper",
        &navigation_runtime,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_state(&self) -> MutexGuard<'_, BuiltinNavigationState>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let mut state = self.lock_state();",
            "let mesh = self.lock_state().selected_mesh_snapshot(query.nav_mesh)?;",
            "Ok(mesh.find_path(query))",
            "Ok(mesh.sample_position(query))",
            "Ok(mesh.raycast(query))",
            "self.lock_state().stats.clone()",
        ],
    );
    assert!(
        !navigation_runtime.contains("expect(\"navigation state lock poisoned\")"),
        "BuiltinNavigationManager production paths should not panic on poisoned navigation state locks"
    );
    assert!(
        !navigation_runtime.contains(LOCK_UNWRAP_CALL),
        "BuiltinNavigationManager production paths should use lock_state() instead of {LOCK_UNWRAP_CALL}"
    );
    assert_contains_all(
        "BuiltinNavigationManager poison recovery test",
        &navigation_tests,
        &[
            "navigation_manager_accessors_recover_poisoned_state_lock",
            "manager.state.lock().unwrap()",
            "NavigationSettingsAsset::default_3d()",
            "manager.stats().loaded_nav_meshes",
        ],
    );
}

// BUG: [CR-RUNTIME-TESTS-STRUCT-0022] 此守卫经运行时源码定位器读取资源管理器旧目录，当前文件不存在，精确测试复现文件读取 panic；核对迁移后的真实 owner 并修订守卫。
#[test]
fn runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager() {
    let resource_manager =
        read_repo("zircon_runtime/crates/zr_resource/src/manager/resource_manager.rs");
    let registry_ops = read_repo("zircon_runtime/crates/zr_resource/src/manager/registry_ops.rs");
    let commit = read_repo("zircon_runtime/crates/zr_resource/src/manager/commit.rs");
    let payload_ops = read_repo("zircon_runtime/crates/zr_resource/src/manager/payload_ops.rs");
    let lease_ops = read_repo("zircon_runtime/crates/zr_resource/src/manager/lease_ops.rs");
    let events = read_repo("zircon_runtime/crates/zr_resource/src/event_stream.rs");
    let resource_manager_tests =
        read_repo("zircon_runtime/crates/zr_resource/src/manager/tests/resource_manager.rs");
    let _runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let _runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let _review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let _structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let _module_doc = read_repo("docs/crates/zircon_runtime/structure/module-convention.md");
    let _resource_doc = read_repo("docs/crates/zircon_runtime/core/resource.md");

    assert_contains_all(
        "ResourceManager poison recovery helpers",
        &resource_manager,
        &[
            "pub(super) authority: Arc<RwLock<ResourceAuthority>>",
            "pub(super) fn lock_authority_read(&self) -> RwLockReadGuard<'_, ResourceAuthority>",
            "pub(super) fn lock_authority_write(&self) -> RwLockWriteGuard<'_, ResourceAuthority>",
            "pub(super) fn lock_commit_serial(&self) -> MutexGuard<'_, ()>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.events.subscribe()",
        ],
    );
    assert_contains_all(
        "ResourceManager registry ops use helpers",
        &registry_ops,
        &[
            "self.commit(ResourceMutationBatch::new().upsert_lazy(record))?",
            "self.commit(ResourceMutationBatch::new().start_reload(id, diagnostics))?",
            "self.commit(ResourceMutationBatch::new().fail_reload(id, diagnostics))?",
            "self.commit(ResourceMutationBatch::new().remove(locator.clone()))?",
            "self.commit(ResourceMutationBatch::new().rename(from.clone(), to.clone()))?",
        ],
    );
    let prepared_commit = commit
        .split("pub(crate) fn prepare_commit(")
        .nth(1)
        .and_then(|tail| tail.split("fn preflight(").next())
        .expect("resource prepare commit owner");
    assert_contains_all(
        "resource mutation preflight lock chain",
        prepared_commit,
        &[
            "let commit_serial = self.lock_commit_serial();",
            "let authority = self.lock_authority_read();",
            "preflight(&authority.registry, batch.operations())?",
        ],
    );
    let publication = commit
        .split("impl PreparedResourceMutation<'_>")
        .nth(1)
        .and_then(|tail| tail.split("impl ResourceManager").next())
        .expect("resource publication owner");
    assert_contains_all(
        "resource mutation publication lock chain",
        publication,
        &[
            "let mut authority = self.manager.lock_authority_write();",
            "apply_staged(&mut authority, self.staged, self.events.len())",
        ],
    );
    let apply = commit
        .split("fn apply_staged(")
        .nth(1)
        .expect("resource staged application owner");
    assert_contains_all(
        "resource mutation readiness publication",
        apply,
        &["authority.refresh_readiness_many"],
    );
    assert_contains_all(
        "ResourceManager payload ops use helpers",
        &payload_ops,
        &["self.lock_authority_read().payloads.get(&id).cloned()"],
    );
    assert_contains_all(
        "ResourceManager lease ops use helpers",
        &lease_ops,
        &[
            "self.lock_authority_write()",
            "authority.runtime",
            "self.lock_authority_read()",
        ],
    );
    assert_contains_all(
        "ResourceManager event ops use helpers",
        &events,
        &[
            "fn lock_state(&self) -> MutexGuard<'_, ResourceEventLogState>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        ],
    );
    assert_contains_all(
        "ResourceManager poison recovery test",
        &resource_manager_tests,
        &[
            "resource_manager_accessors_recover_poisoned_state_locks",
            "manager.poison_event_stream_for_test();",
            "manager.lock_authority_write()",
            "recv_timeout(Duration::from_secs(1))",
            "manager.runtime_state(id)",
            "manager.ref_count(id)",
        ],
    );

    for (label, source) in [
        ("resource manager root", resource_manager.as_str()),
        ("resource registry ops", registry_ops.as_str()),
        ("resource mutation commit", commit.as_str()),
        ("resource payload ops", payload_ops.as_str()),
        ("resource lease ops", lease_ops.as_str()),
        ("resource event ops", events.as_str()),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
        assert!(
            !production_section(source).contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }
}
