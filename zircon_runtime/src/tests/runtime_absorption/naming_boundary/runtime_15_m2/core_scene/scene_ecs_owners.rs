use super::*;

#[test]
fn runtime_15_scene_ecs_observer_callback_registry_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let observer_dir = manifest_root.join("src/scene/ecs/observer");
    let retired_utils = observer_dir.join("utils.rs");
    let observer_mod = read_text(
        &observer_dir.join("mod.rs"),
        "scene ECS observer module entry should be readable",
    );
    let observer_store = read_text(
        &observer_dir.join("store.rs"),
        "scene ECS observer store should be readable",
    );
    let callback_registry = read_text(
        &observer_dir.join("callback_registry.rs"),
        "scene ECS observer callback registry should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let scene_ecs_doc = read_repo_text(manifest_root, "docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming boundary expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming boundary expected date map should be readable",
    );

    assert!(
        !retired_utils.exists(),
        "scene ECS observer should not keep banned-name module file {:?}",
        retired_utils
    );
    assert_contains_all(
        "scene ECS observer module entry",
        &observer_mod,
        &["mod callback_registry;"],
    );
    assert!(
        !observer_mod.contains("mod utils;"),
        "scene/ecs/observer/mod.rs should not preserve the banned utils module name"
    );
    assert_contains_all(
        "scene ECS observer store owner",
        &observer_store,
        &[
            "use super::callback_registry::{",
            "entity_event_callback_count",
            "event_callback_count",
            "lifecycle_callback_count",
            "remove_observer_by_id",
        ],
    );
    assert!(
        !observer_store.contains("super::utils"),
        "scene/ecs/observer/store.rs should consume callback_registry instead of utils"
    );
    assert_contains_all(
        "scene ECS observer callback registry",
        &callback_registry,
        &[
            "pub(super) fn lifecycle_callback_count",
            "pub(super) fn event_callback_count",
            "pub(super) fn entity_event_callback_count",
            "pub(super) fn remove_observer_by_id",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene ECS doc", scene_ecs_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 scene ECS observer callback registry module naming hard cutover",
                "runtime_15_scene_ecs_observer_callback_registry_naming_hard_cutover_static_passed_cargo_deferred",
                "scene/ecs/observer/callback_registry.rs",
                "runtime_15_scene_ecs_observer_callback_registry_uses_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let query_state_dir = manifest_root.join("src/scene/ecs/query/query_state");
    let retired_helpers = query_state_dir.join("helpers.rs");
    let query_state_mod = read_text(
        &query_state_dir.join("mod.rs"),
        "scene ECS query-state module entry should be readable",
    );
    let many_item_array = read_text(
        &query_state_dir.join("many_item_array.rs"),
        "scene ECS query-state many-item array owner should be readable",
    );
    let query_state_callers = [
        read_text(
            &query_state_dir.join("cached_direct.rs"),
            "scene ECS cached direct query-state owner should be readable",
        ),
        read_text(
            &query_state_dir.join("mutable.rs"),
            "scene ECS mutable query-state owner should be readable",
        ),
        read_text(
            &query_state_dir.join("read_only.rs"),
            "scene ECS read-only query-state owner should be readable",
        ),
        read_text(
            &query_state_dir.join("read_only_cached.rs"),
            "scene ECS cached read-only query-state owner should be readable",
        ),
    ];
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let scene_ecs_doc = read_repo_text(manifest_root, "docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

    assert!(
        !retired_helpers.exists(),
        "scene ECS query-state should not keep banned-name module file {:?}",
        retired_helpers
    );
    assert_contains_all(
        "scene ECS query-state module entry",
        &query_state_mod,
        &["mod many_item_array;"],
    );
    assert!(
        !query_state_mod.contains("mod helpers;"),
        "scene/ecs/query/query_state/mod.rs should not preserve the banned helpers module name"
    );
    assert_contains_all(
        "scene ECS query-state many-item array owner",
        &many_item_array,
        &[
            "pub(super) fn collect_many_query_items",
            "MaybeUninit<Item>",
            "assume_init_drop",
            "*const [Item; N]",
        ],
    );
    for caller in &query_state_callers {
        assert_contains_all(
            "scene ECS query-state caller",
            caller,
            &["super::many_item_array::collect_many_query_items"],
        );
        assert!(
            !caller.contains("super::helpers"),
            "query-state callers should consume many_item_array instead of helpers"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene ECS doc", scene_ecs_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 scene ECS query-state many-item array module naming hard cutover",
                "runtime_15_scene_ecs_query_state_many_item_array_naming_hard_cutover_static_passed_cargo_deferred",
                "scene/ecs/query/query_state/many_item_array.rs",
                "runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_scene_ecs_component_storage_component_results_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let component_storage_dir = manifest_root.join("src/scene/ecs/storage/component_storage");
    let retired_utils = component_storage_dir.join("utils.rs");
    let component_storage_mod = read_text(
        &component_storage_dir.join("mod.rs"),
        "scene ECS component-storage module entry should be readable",
    );
    let component_storage_store = read_text(
        &component_storage_dir.join("store.rs"),
        "scene ECS component-storage store should be readable",
    );
    let component_results = read_text(
        &component_storage_dir.join("component_results.rs"),
        "scene ECS component-storage component results owner should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let scene_ecs_doc = read_repo_text(manifest_root, "docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

    assert!(
        !retired_utils.exists(),
        "scene ECS component-storage should not keep banned-name module file {:?}",
        retired_utils
    );
    assert_contains_all(
        "scene ECS component-storage module entry",
        &component_storage_mod,
        &["mod component_results;"],
    );
    assert!(
        !component_storage_mod.contains("mod utils;"),
        "scene/ecs/storage/component_storage/mod.rs should not preserve the banned utils module name"
    );
    assert_contains_all(
        "scene ECS component-storage store owner",
        &component_storage_store,
        &[
            "use super::component_results::{",
            "downcast_component",
            "sort_component_ids_if_needed",
        ],
    );
    assert!(
        !component_storage_store.contains("super::utils"),
        "scene/ecs/storage/component_storage/store.rs should consume component_results instead of utils"
    );
    assert_contains_all(
        "scene ECS component-storage component results",
        &component_results,
        &[
            "pub(in crate::scene::ecs::storage) fn sort_component_ids_if_needed",
            "component_ids.sort_unstable();",
            "pub(in crate::scene::ecs::storage) fn downcast_component",
            "StorageError::ComponentTypeMismatch",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene ECS doc", scene_ecs_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 scene ECS component-storage component results module naming hard cutover",
                "runtime_15_scene_ecs_component_storage_component_results_naming_hard_cutover_static_passed_cargo_deferred",
                "scene/ecs/storage/component_storage/component_results.rs",
                "runtime_15_scene_ecs_component_storage_component_results_uses_owner_name",
            ],
        );
    }
}
