#[test]
fn review_f2_scene_eventbus_locks_recover_after_poison() {
    let level_system = include_str!("../../../../scene/level_system.rs");
    let default_level_manager = include_str!("../../../../scene/module/default_level_manager.rs");
    let level_manager_lifecycle =
        include_str!("../../../../scene/module/level_manager_lifecycle.rs");
    let event_bus = include_str!("../../../../core/runtime/events.rs");
    let event_publish = include_str!("../../../../core/runtime/events/publish.rs");
    let event_subscribe = include_str!("../../../../core/runtime/events/subscribe.rs");
    let event_prune = include_str!("../../../../core/runtime/events/prune.rs");
    let level_doc = include_str!("../../../../../../docs/zircon_runtime/scene/level_system.md");
    let event_doc = include_str!("../../../../../../docs/zircon_runtime/core/runtime/events.md");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let module_doc =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs"
    );
    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "fn lock_poison_recovered<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T>",
        ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        "fn lock_world(&self) -> MutexGuard<'_, World>",
        "fn lock_runtime_state(&self) -> MutexGuard<'_, WorldRuntimeState>",
        "fn lock_metadata(&self) -> MutexGuard<'_, LevelMetadata>",
        "fn lock_lifecycle(&self) -> MutexGuard<'_, LevelLifecycleState>",
        "fn lock_subsystems(&self) -> MutexGuard<'_, Vec<String>>",
        "level_system_accessors_recover_poisoned_state_locks",
    ] {
        assert!(
            level_system.contains(required),
            "LevelSystem should retain poison-safe lock recovery anchor `{required}`"
        );
    }

    for required in [
        "pub(super) fn lock_levels(&self) -> MutexGuard<'_, HashMap<WorldHandle, LevelSystem>>",
        ".unwrap_or_else(|poisoned| poisoned.into_inner())",
    ] {
        assert!(
            default_level_manager.contains(required),
            "DefaultLevelManager should retain poison-safe lock recovery anchor `{required}`"
        );
    }
    for required in [
        "self.lock_levels().insert(handle, level.clone())",
        "self.lock_levels().get(&handle).cloned()",
    ] {
        assert!(
            level_manager_lifecycle.contains(required),
            "level manager lifecycle should delegate level-map access through `{required}`"
        );
    }

    for required in [
        "fn lock_subscribers(&self) -> MutexGuard<'_, EventSubscriberMap>",
        "fn lock_delivery(&self) -> MutexGuard<'_, ()>",
        ".unwrap_or_else(|poisoned| poisoned.into_inner())",
    ] {
        assert!(
            event_bus.contains(required),
            "EventBus should retain poison-safe lock recovery anchor `{required}`"
        );
    }
    for required in [
        "self.lock_delivery()",
        "self.prune_topic_subscribers(",
        "let mut subscribers = self.lock_subscribers();",
    ] {
        assert!(
            event_publish.contains(required)
                || event_subscribe.contains(required)
                || event_prune.contains(required),
            "EventBus publish/subscribe/prune owners should keep helper usage `{required}`"
        );
    }

    for (label, source) in [
        ("level system", level_system),
        ("default level manager", default_level_manager),
        ("level manager lifecycle", level_manager_lifecycle),
        ("event bus root", event_bus),
        ("event publish", event_publish),
        ("event subscribe", event_subscribe),
        ("event prune", event_prune),
    ] {
        let production = source.split("\n#[cfg(test)]").next().unwrap_or(source);
        assert!(
            !production.contains(".lock().unwrap()"),
            "{label} production code should recover poisoned locks instead of direct lock unwrap"
        );
    }

    for doc_anchor in [
        "Runtime 15 M3 F2 lock poison recovery guard",
        "runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending",
        "review_f2_scene_eventbus_locks_recover_after_poison",
        "p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred",
        "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus",
        "level_system_accessors_recover_poisoned_state_locks",
        "scene/EventBus poison-safe lock recovery complete",
    ] {
        assert!(
            level_doc.contains(doc_anchor)
                || event_doc.contains(doc_anchor)
                || review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F2 scene/EventBus lock poison docs/status should record `{doc_anchor}`"
        );
    }
    let f2_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F2 |"))
        .expect("F2 row should exist");
    assert!(
        f2_row.ends_with("| Runtime 15 + Runtime 07 / review closed |"),
        "F2 row should mark the lock-poison recovery review state closed"
    );
}
