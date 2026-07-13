use super::*;

#[path = "script_vm_tests/gameplay_host.rs"]
mod gameplay_host;
#[path = "script_vm_tests/hot_reload.rs"]
mod hot_reload;
#[path = "script_vm_tests/primary.rs"]
mod primary;

#[test]
fn runtime_15_script_vm_tests_are_folder_backed() {
    primary::assert_script_vm_tests_are_folder_backed();
}

#[test]
fn runtime_15_script_vm_primary_guard_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
    );
    let child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/primary.rs",
    );

    assert_contains_all(
        "script VM test-budget guard mounts primary child owner",
        &parent,
        &[
            "#[path = \"script_vm_tests/primary.rs\"]",
            "mod primary;",
            "fn runtime_15_script_vm_tests_are_folder_backed",
            "primary::assert_script_vm_tests_are_folder_backed",
            "fn runtime_15_script_vm_primary_guard_is_child_owner",
        ],
    );
    for moved_anchor in [
        "let bridge_host = read_runtime_src(\"script/vm/tests/bridge_host.rs\")",
        concat!(
            "fn host_export_registry_validates_descriptors_",
            "and_dispatches_callbacks"
        ),
        concat!(
            "runtime_15_script_vm_tests_folder_split_static_",
            "passed_cargo_timeout_no_result"
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "script_vm_tests.rs should delegate primary script VM anchor `{moved_anchor}` to script_vm_tests/primary.rs"
        );
    }
    assert_contains_all(
        "script VM primary child owns folder-backed structure checks",
        &child,
        &[
            "pub(super) fn assert_script_vm_tests_are_folder_backed",
            "script/vm/tests/bridge_host.rs",
            "script/vm/tests/reflection_docs.rs",
            "fn host_reflection_docs_render_synthetic_descriptor_deterministically",
            concat!(
                "runtime_15_script_vm_tests_folder_split_static_",
                "passed_cargo_timeout_no_result"
            ),
            "runtime_15_script_vm_primary_guard_is_child_owner",
        ],
    );
    primary::assert_script_vm_tests_are_folder_backed();

    for (path, source) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/primary.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed() {
    hot_reload::assert_hot_reload_coordinator_tests_are_folder_backed();
}

#[test]
fn runtime_15_script_vm_hot_reload_guard_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
    );
    let child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/hot_reload.rs",
    );

    assert_contains_all(
        "script VM test-budget guard mounts hot-reload child owner",
        &parent,
        &[
            "#[path = \"script_vm_tests/hot_reload.rs\"]",
            "mod hot_reload;",
            "fn runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed",
            "hot_reload::assert_hot_reload_coordinator_tests_are_folder_backed",
            "fn runtime_15_script_vm_hot_reload_guard_is_child_owner",
        ],
    );
    for moved_anchor in [
        "let child = read_runtime_src(\"script/vm/runtime/hot_reload_coordinator/tests.rs\")",
        concat!(
            "fn hot_reload_policy_preserves_state_and_",
            "increments_generation_by_default"
        ),
        concat!(
            "runtime_15_script_vm_hot_reload_coordinator_tests_",
            "folder_split_static_passed_cargo_deferred"
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "script_vm_tests.rs should delegate hot-reload anchor `{moved_anchor}` to script_vm_tests/hot_reload.rs"
        );
    }
    assert_contains_all(
        "script VM hot-reload child owns coordinator structure checks",
        &child,
        &[
            "pub(super) fn assert_hot_reload_coordinator_tests_are_folder_backed",
            "script/vm/runtime/hot_reload_coordinator.rs",
            "script/vm/runtime/hot_reload_coordinator/tests.rs",
            concat!(
                "fn hot_reload_policy_preserves_state_and_",
                "increments_generation_by_default"
            ),
            concat!(
                "runtime_15_script_vm_hot_reload_coordinator_tests_",
                "folder_split_static_passed_cargo_deferred"
            ),
            "runtime_15_script_vm_hot_reload_guard_is_child_owner",
        ],
    );
    hot_reload::assert_hot_reload_coordinator_tests_are_folder_backed();

    for (path, source) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/hot_reload.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_gameplay_host_tests_are_folder_backed() {
    gameplay_host::assert_gameplay_host_tests_are_folder_backed();
}

#[test]
fn runtime_15_script_vm_gameplay_host_guard_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
    );
    let child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/gameplay_host.rs",
    );

    assert_contains_all(
        "script VM test-budget guard mounts gameplay host child owner",
        &parent,
        &[
            "#[path = \"script_vm_tests/gameplay_host.rs\"]",
            "mod gameplay_host;",
            "fn runtime_15_gameplay_host_tests_are_folder_backed",
            "gameplay_host::assert_gameplay_host_tests_are_folder_backed",
            "fn runtime_15_script_vm_gameplay_host_guard_is_child_owner",
        ],
    );
    for moved_anchor in [
        "let spawn_transform = read_runtime_src(\"script/vm/gameplay_host/tests/spawn_transform.rs\")",
        concat!("fn gameplay_pose_exports_update_entity_", "transform"),
        concat!(
            "runtime_15_gameplay_host_tests_folder_split_",
            "static_passed_cargo_deferred"
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "script_vm_tests.rs should delegate gameplay host anchor `{moved_anchor}` to script_vm_tests/gameplay_host.rs"
        );
    }
    assert_contains_all(
        "script VM gameplay host child owns gameplay structure checks",
        &child,
        &[
            "pub(super) fn assert_gameplay_host_tests_are_folder_backed",
            "script/vm/gameplay_host/tests.rs",
            "script/vm/gameplay_host/tests/spawn_transform.rs",
            concat!("fn gameplay_pose_exports_update_entity_", "transform"),
            concat!(
                "runtime_15_gameplay_host_tests_folder_split_",
                "static_passed_cargo_deferred"
            ),
            "runtime_15_script_vm_gameplay_host_guard_is_child_owner",
        ],
    );
    gameplay_host::assert_gameplay_host_tests_are_folder_backed();

    for (path, source) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/gameplay_host.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
