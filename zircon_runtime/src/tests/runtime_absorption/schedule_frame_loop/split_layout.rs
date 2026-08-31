const PARENT_SOURCE: &str = include_str!("../schedule_frame_loop.rs");
const INVENTORY_SOURCE: &str = include_str!("inventory.rs");
const RUNTIME_ANCHORS_SOURCE: &str = include_str!("runtime_anchors.rs");
const MIRROR_DOCS_SOURCE: &str = include_str!("mirror_docs.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");
const SCHEDULE_RUNNER_SOURCE: &str = include_str!("../../../scene/ecs/schedule_runner.rs");
const SCHEDULE_RUNNER_TEST_ROOT_SOURCE: &str =
    include_str!("../../../scene/ecs/schedule_runner/tests/mod.rs");
const SCHEDULE_RUNNER_WORKER_DISPATCH_TESTS: &str =
    include_str!("../../../scene/ecs/schedule_runner/tests/worker_dispatch.rs");
const SCHEDULE_RUNNER_PANIC_RECOVERY_TESTS: &str =
    include_str!("../../../scene/ecs/schedule_runner/tests/panic_recovery.rs");
const SCHEDULE_RUNNER_SUPPORT_TESTS: &str =
    include_str!("../../../scene/ecs/schedule_runner/tests/support.rs");
const SCHEDULE_RUNNER_TYPED_WORKER_STRUCTURAL_TESTS: &str =
    include_str!("../../../scene/ecs/schedule_runner/tests/typed_worker_structural.rs");
const SCHEDULE_RUNNER_WORKER_CALLBACK_ORDER_TESTS: &str =
    include_str!("../../../scene/ecs/schedule_runner/tests/worker_callback_order.rs");

const FRAMEWORKS_02_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
);
const RUNTIME_15_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");

#[test]
fn runtime_15_schedule_frame_loop_route_owner_is_folder_backed() {
    assert_contains_all(
        "schedule_frame_loop route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"schedule_frame_loop/inventory.rs\"]",
            "#[path = \"schedule_frame_loop/mirror_docs.rs\"]",
            "#[path = \"schedule_frame_loop/runtime_anchors.rs\"]",
            "#[path = \"schedule_frame_loop/split_layout.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owners_are_focused();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

#[test]
fn runtime_03_schedule_runner_behavior_tests_are_folder_backed() {
    assert_contains_all(
        "schedule runner test route",
        SCHEDULE_RUNNER_SOURCE,
        &["#[path = \"schedule_runner/tests/mod.rs\"]", "mod tests;"],
    );
    for forbidden in ["mod tests {", "include!(\"schedule_runner/tests/"] {
        assert!(
            !SCHEDULE_RUNNER_SOURCE.contains(forbidden),
            "schedule_runner.rs should not retain inline behavior test `{forbidden}`"
        );
    }
    assert_contains_all(
        "schedule runner test root",
        SCHEDULE_RUNNER_TEST_ROOT_SOURCE,
        &[
            "mod panic_recovery;",
            "mod typed_worker_structural;",
            "mod worker_callback_order;",
            "mod worker_dispatch;",
            "mod support;",
        ],
    );
    for (label, source) in [
        (
            "schedule runner test root",
            SCHEDULE_RUNNER_TEST_ROOT_SOURCE,
        ),
        (
            "schedule runner worker dispatch tests",
            SCHEDULE_RUNNER_WORKER_DISPATCH_TESTS,
        ),
        (
            "schedule runner panic recovery tests",
            SCHEDULE_RUNNER_PANIC_RECOVERY_TESTS,
        ),
        (
            "schedule runner support tests",
            SCHEDULE_RUNNER_SUPPORT_TESTS,
        ),
        (
            "schedule runner typed worker structural tests",
            SCHEDULE_RUNNER_TYPED_WORKER_STRUCTURAL_TESTS,
        ),
        (
            "schedule runner worker callback order tests",
            SCHEDULE_RUNNER_WORKER_CALLBACK_ORDER_TESTS,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= 800,
            "{label} has {line_count} lines; expected at most 800"
        );
    }
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "schedule_frame_loop.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "EXPECTED_RUNTIME_03_SOURCE_FILES",
        "SystemStage authority should retain",
        "mirror_docs_match_structure_audit_counts",
        "include_str!(",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "schedule_frame_loop.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "inventory child",
        INVENTORY_SOURCE,
        &[
            "EXPECTED_RUNTIME_03_SOURCE_FILES",
            "EXPECTED_RUNTIME_03_GUARD_FILES",
            "EXPECTED_RUNTIME_03_BEHAVIOR_TEST_ANCHORS",
        ],
    );
    assert_contains_all(
        "runtime anchors child",
        RUNTIME_ANCHORS_SOURCE,
        &[
            "assert_runtime_03_sources_and_anchors",
            "assert_system_stage_contract",
            "assert_dynamic_session_time_handoff",
            "assert_behavior_test_anchors",
        ],
    );
    assert_contains_all(
        "mirror docs child",
        MIRROR_DOCS_SOURCE,
        &[
            "runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts",
            "assert_mirror_docs_match_structure_audit",
            "Runtime 03 plan",
            "interface convergence",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 10),
        ("inventory child", INVENTORY_SOURCE, 70),
        ("runtime anchors child", RUNTIME_ANCHORS_SOURCE, 130),
        ("mirror docs child", MIRROR_DOCS_SOURCE, 70),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 220),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected at most {max_lines}"
        );
    }
}

fn assert_docs_and_status_mirror_split() {
    assert!(
        RUNTIME_15_OUTPUT_RECORDS.contains(
            "runtime_15_schedule_frame_loop_route_owner_split_static_passed_cargo_deferred"
        ),
        "Runtime 15 output records should own the schedule_frame_loop route-owner split status"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "schedule_frame_loop/inventory.rs",
            "schedule_frame_loop/runtime_anchors.rs",
            "schedule_frame_loop/mirror_docs.rs",
            "schedule_frame_loop/split_layout.rs",
            "runtime_15_schedule_frame_loop_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 output records",
        FRAMEWORKS_02_OUTPUT_RECORDS,
        &[
            "frameworks_02_m3_schedule_frame_loop_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 schedule-frame-loop route-owner split",
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain split anchor `{anchor}`"
        );
    }
}
