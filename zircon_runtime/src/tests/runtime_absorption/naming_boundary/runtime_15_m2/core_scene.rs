use std::path::Path;

use super::super::{assert_contains_all, read_repo_text, read_text};

#[test]
fn runtime_15_core_runtime_state_module_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let state_dir = manifest_root.join("src/core/runtime/state");
    let retired_runtime_inner = state_dir.join("runtime_inner.rs");
    let state_mod = read_text(
        &state_dir.join("mod.rs"),
        "core runtime state module entry should be readable",
    );
    let core_runtime_state = read_text(
        &state_dir.join("core_runtime_state.rs"),
        "core runtime state owner should be readable",
    );
    let registration_structure = read_text(
        &manifest_root.join("src/core/runtime/tests/registration/structure/mod.rs"),
        "registration structure source fixture should be readable",
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
    let core_state_doc = read_repo_text(manifest_root, "docs/zircon_runtime/core/state.md");
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
        !retired_runtime_inner.exists(),
        "core runtime state owner should not keep banned-name module file {:?}",
        retired_runtime_inner
    );
    assert_contains_all(
        "core runtime state mod entry",
        &state_mod,
        &[
            "mod core_runtime_state;",
            "pub(crate) use core_runtime_state::CoreRuntimeInner;",
        ],
    );
    assert!(
        !state_mod.contains("runtime_inner"),
        "core/runtime/state/mod.rs should not preserve the banned runtime_inner module name"
    );
    assert_contains_all(
        "core runtime state owner",
        &core_runtime_state,
        &[
            "pub(crate) struct CoreRuntimeInner",
            "HashMap<RegistryName, ServiceEntry>",
            "plugin_bridge_lifecycle",
        ],
    );
    assert_contains_all(
        "core runtime registration structure fixture",
        &registration_structure,
        &[
            "pub(super) runtime_state: &'static str",
            "include_str!(\"../../../state/core_runtime_state.rs\")",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("core state doc", core_state_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 core runtime state module naming hard cutover",
                "runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred",
                "core/runtime/state/core_runtime_state.rs",
                "runtime_15_core_runtime_state_module_uses_owner_name",
            ],
        );
    }
}

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

#[test]
fn runtime_15_scene_render_layer_schema_v1_masks_use_versioned_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let camera_source = read_text(
        &manifest_root.join("src/core/framework/render/camera.rs"),
        "render camera source should be readable",
    );
    let scene_render_files = [
        "src/scene/world/render.rs",
        "src/scene/world/render/lights.rs",
        "src/scene/world/render_particles.rs",
        "src/scene/world/render_post_process.rs",
        "src/scene/world/render_visibility.rs",
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
    let camera_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/core/framework/render/camera.md",
    );
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

    assert_contains_all(
        "RenderLayerSet scene schema v1 mask API",
        &camera_source,
        &[
            "pub fn from_scene_schema_v1_mask(mask: u32) -> Self",
            "pub fn to_scene_schema_v1_mask_lossy(&self) -> u32",
            "pub fn intersects_scene_schema_v1_mask(&self, mask: u32) -> bool",
        ],
    );

    for relative_path in scene_render_files {
        let source = read_text(
            &manifest_root.join(relative_path),
            "scene render source should be readable",
        );
        assert!(
            !source.contains("legacy"),
            "{relative_path} should not keep legacy scene schema/render layer naming"
        );
        assert!(
            !source.contains("from_legacy_mask")
                && !source.contains("to_legacy_mask_lossy")
                && !source.contains("intersects_legacy_mask"),
            "{relative_path} should use scene_schema_v1 render layer mask APIs"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render camera doc", camera_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 scene render layer schema-v1 mask naming hard cutover",
                "runtime_15_scene_render_layer_schema_v1_mask_naming_hard_cutover_static_passed_cargo_deferred",
                "from_scene_schema_v1_mask",
                "runtime_15_scene_render_layer_schema_v1_masks_use_versioned_names",
            ],
        );
    }
}

#[test]
fn runtime_15_render_shader_definition_uses_bare_flag_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shader_definition_source = read_text(
        &manifest_root.join("src/core/framework/render/shader/definition_value.rs"),
        "render shader definition value source should be readable",
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
    let render_assets_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/asset/render-assets.md");
    let zmeta_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/asset/zmeta-shader-material.md",
    );
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

    assert_contains_all(
        "render shader definition bare flag serde branch",
        &shader_definition_source,
        &[
            "BareFlag(String)",
            "DefinitionValueRepr::BareFlag(name) => Self::from(name)",
        ],
    );
    assert!(
        !shader_definition_source.contains("LegacyFlag"),
        "render shader definition value serde branch should not use legacy naming"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render assets doc", render_assets_doc.as_str()),
        ("zmeta shader material doc", zmeta_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 render shader definition bare-flag naming hard cutover",
                "runtime_15_render_shader_definition_bare_flag_naming_hard_cutover_static_passed_cargo_deferred",
                "BareFlag",
                "runtime_15_render_shader_definition_uses_bare_flag_names",
            ],
        );
    }
}

#[test]
fn runtime_15_frame_extract_snapshot_adapter_uses_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let frame_extract_source = read_text(
        &manifest_root.join("src/core/framework/render/frame_extract.rs"),
        "render frame extract source should be readable",
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
    let scene_render_extract_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/scene/render_extract.md");
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

    assert_contains_all(
        "frame extract snapshot adapter source names",
        &frame_extract_source,
        &[
            "Builds a frame DTO from the scene viewport snapshot packet for preview,",
            "pub fn from_snapshot(world: RenderWorldSnapshotHandle, snapshot: RenderSceneSnapshot)",
            "from a `SceneViewportRenderPacket`",
        ],
    );
    assert!(
        !frame_extract_source.contains("legacy viewport packet"),
        "RenderFrameExtract::from_snapshot should describe the snapshot adapter without legacy packet wording"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "scene render extract doc",
            scene_render_extract_doc.as_str(),
        ),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 frame extract snapshot adapter naming hard cutover",
                "runtime_15_frame_extract_snapshot_adapter_naming_hard_cutover_static_passed_cargo_deferred",
                "core/framework/render/frame_extract.rs",
                "scene viewport snapshot packet",
                "runtime_15_frame_extract_snapshot_adapter_uses_current_names",
            ],
        );
    }
}
