use super::*;

#[test]
fn runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus() {
    let level_system = read_runtime_src("scene/level_system.rs");
    let default_level_manager = read_runtime_src("scene/module/default_level_manager.rs");
    let level_manager_lifecycle = read_runtime_src("scene/module/level_manager_lifecycle.rs");
    let event_bus = read_runtime_src("core/runtime/events.rs");
    let event_publish = read_runtime_src("core/runtime/events/publish.rs");
    let event_subscribe = read_runtime_src("core/runtime/events/subscribe.rs");
    let event_prune = read_runtime_src("core/runtime/events/prune.rs");
    let structure_parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let level_doc = read_repo("docs/zircon_runtime/scene/level_system.md");
    let event_doc = read_repo("docs/zircon_runtime/core/runtime/events.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "structure convention parent lock-poison mount",
        &structure_parent,
        &[
            "#[path = \"structure_convention/lock_poison_policy.rs\"]",
            "mod lock_poison_policy;",
        ],
    );

    assert_contains_all(
        "LevelSystem poison recovery helper",
        &level_system,
        &[
            "fn lock_poison_recovered<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "fn lock_world(&self) -> MutexGuard<'_, World>",
            "fn lock_runtime_state(&self) -> MutexGuard<'_, WorldRuntimeState>",
            "fn lock_metadata(&self) -> MutexGuard<'_, LevelMetadata>",
            "fn lock_lifecycle(&self) -> MutexGuard<'_, LevelLifecycleState>",
            "fn lock_subsystems(&self) -> MutexGuard<'_, Vec<String>>",
            "level_system_accessors_recover_poisoned_state_locks",
        ],
    );
    assert_contains_all(
        "DefaultLevelManager poison recovery helper",
        &default_level_manager,
        &[
            "pub(super) fn lock_levels(&self) -> MutexGuard<'_, HashMap<WorldHandle, LevelSystem>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        ],
    );
    assert_contains_all(
        "level manager lifecycle delegates to shared helper",
        &level_manager_lifecycle,
        &[
            "self.lock_levels().insert(handle, level.clone())",
            "self.lock_levels().get(&handle).cloned()",
        ],
    );

    assert_contains_all(
        "EventBus poison recovery helpers",
        &event_bus,
        &[
            "fn lock_subscribers(&self) -> MutexGuard<'_, EventSubscriberMap>",
            "fn lock_delivery(&self) -> MutexGuard<'_, ()>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        ],
    );
    assert_contains_all(
        "EventBus publish/subscribe/prune helpers",
        &event_publish,
        &["self.lock_delivery()", "self.prune_topic_subscribers("],
    );
    assert_contains_all(
        "EventBus subscribe helper",
        &event_subscribe,
        &["let mut subscribers = self.lock_subscribers();"],
    );
    assert_contains_all(
        "EventBus prune helper",
        &event_prune,
        &["let mut subscribers = self.lock_subscribers();"],
    );

    for (label, source) in [
        ("level system", level_system.as_str()),
        ("default level manager", default_level_manager.as_str()),
        ("level manager lifecycle", level_manager_lifecycle.as_str()),
        ("event bus root", event_bus.as_str()),
        ("event publish", event_publish.as_str()),
        ("event subscribe", event_subscribe.as_str()),
        ("event prune", event_prune.as_str()),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("level system doc", level_doc.as_str()),
        ("event bus doc", event_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 F2 lock poison recovery guard",
                "runtime_15_f2_lock_poison_recovery_guard_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy.rs",
                "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus",
            ],
        );
    }
}
