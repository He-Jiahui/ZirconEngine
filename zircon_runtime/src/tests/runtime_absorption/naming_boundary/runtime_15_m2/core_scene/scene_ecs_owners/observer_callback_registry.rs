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
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let scene_ecs_doc = read_repo_text(manifest_root, "docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/core_scene_asset_dynamic.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/scene_asset_runtime.rs",
        ),
        "Runtime 15 naming boundary expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/scene_asset_runtime.rs",
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
