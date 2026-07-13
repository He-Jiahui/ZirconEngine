use super::*;

#[test]
fn runtime_15_scene_ecs_reflect_foundation_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/ecs_reflect/foundation.rs");
    let address_routing = read_runtime_src("scene/tests/ecs_reflect/foundation/address_routing.rs");
    let fixed_lights_name =
        read_runtime_src("scene/tests/ecs_reflect/foundation/fixed_lights_name.rs");
    let fixed_registry = read_runtime_src("scene/tests/ecs_reflect/foundation/fixed_registry.rs");
    let fixed_render_physics =
        read_runtime_src("scene/tests/ecs_reflect/foundation/fixed_render_physics.rs");
    let fixed_transform_active =
        read_runtime_src("scene/tests/ecs_reflect/foundation/fixed_transform_active.rs");
    let registry = read_runtime_src("scene/tests/ecs_reflect/foundation/registry.rs");
    let value_conversion =
        read_runtime_src("scene/tests/ecs_reflect/foundation/value_conversion.rs");
    let versioned_json = read_runtime_src("scene/tests/ecs_reflect/foundation/versioned_json.rs");

    assert_contains_all(
        "scene ECS reflect foundation parent mounts folder-backed children",
        &parent,
        &[
            "mod address_routing;",
            "mod fixed_lights_name;",
            "mod fixed_registry;",
            "mod fixed_render_physics;",
            "mod fixed_transform_active;",
            "mod registry;",
            "mod value_conversion;",
            "mod versioned_json;",
            "fn metadata_registration(",
            "fn typed_registration(",
            "fn fixed_component_address(",
            "fn dummy_component_adapter()",
            "fn dummy_resource_adapter()",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/ecs_reflect/foundation.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "empty_world_builds_runtime_only_type_registry",
        "scene_property_values_convert_to_reflected_values",
        "world_reflection_routes_component_and_resource_addresses",
        "fixed_component_registrations_exist_in_empty_world",
        "ambient_and_rect_light_reflection_roundtrips_authoring_fields",
        "local_transform_reflection_write_marks_transform_dirty_state",
        "rigid_body_reflection_exposes_selected_safe_fields",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved ECS reflect foundation test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "registry child owns type-registry tests",
        &registry,
        &[
            "fn empty_world_builds_runtime_only_type_registry",
            "fn type_registry_rejects_duplicate_full_type_paths",
            "fn type_registry_short_path_lookup_reports_ambiguity",
            "fn runtime_type_registration_compares_adapter_presence_not_identity",
            "fn world_serialization_skips_reflection_registry_and_rebuilds_it_on_load",
        ],
    );
    assert_contains_all(
        "value-conversion child owns reflection value conversion tests",
        &value_conversion,
        &[
            "fn scene_property_values_convert_to_reflected_values",
            "fn reflected_values_convert_to_scene_property_values_when_supported",
            "fn animation_parameter_conversion_returns_structured_error",
        ],
    );
    assert_contains_all(
        "versioned-json child owns reflected JSON migration and writer boundary tests",
        &versioned_json,
        &[
            "fn reflected_json_v0_migrates_asset_refs_and_resaves_idempotently",
            "fn retired_asset_ref_migration_only_rewrites_the_exact_retired_shape",
            "fn reflected_json_rejects_future_headers_before_payload_decode",
            "fn reflected_json_writer_rejects_non_finite_values_with_typed_source",
        ],
    );
    assert_contains_all(
        "address-routing child owns component/resource routing test",
        &address_routing,
        &["fn world_reflection_routes_component_and_resource_addresses"],
    );
    assert_contains_all(
        "fixed-registry child owns fixed registration test",
        &fixed_registry,
        &["fn fixed_component_registrations_exist_in_empty_world"],
    );
    assert_contains_all(
        "fixed-lights-name child owns light and name reflection tests",
        &fixed_lights_name,
        &[
            "fn ambient_and_rect_light_reflection_roundtrips_authoring_fields",
            "fn name_component_reads_and_writes_through_world_reflection",
        ],
    );
    assert_contains_all(
        "fixed-transform-active child owns active/transform reflection tests",
        &fixed_transform_active,
        &[
            "fn active_self_reflection_write_marks_active_dirty_state",
            "fn local_transform_reflection_write_marks_transform_dirty_state",
            "fn local_transform_rotation_is_readable_but_not_writable_in_m8",
        ],
    );
    assert_contains_all(
        "fixed-render-physics child owns render/physics/error reflection tests",
        &fixed_render_physics,
        &[
            "fn render_layer_mask_reflection_roundtrips_unsigned_mask",
            "fn rigid_body_reflection_exposes_selected_safe_fields",
            "fn unknown_fixed_field_returns_structured_error",
            "fn missing_fixed_component_returns_structured_error",
        ],
    );

    let child_test_total = [
        address_routing.as_str(),
        fixed_lights_name.as_str(),
        fixed_registry.as_str(),
        fixed_render_physics.as_str(),
        fixed_transform_active.as_str(),
        registry.as_str(),
        value_conversion.as_str(),
        versioned_json.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 23,
        "ECS reflect foundation children should preserve all 23 tests"
    );

    for (path, source) in [
        ("scene/tests/ecs_reflect/foundation.rs", parent.as_str()),
        (
            "scene/tests/ecs_reflect/foundation/address_routing.rs",
            address_routing.as_str(),
        ),
        (
            "scene/tests/ecs_reflect/foundation/fixed_lights_name.rs",
            fixed_lights_name.as_str(),
        ),
        (
            "scene/tests/ecs_reflect/foundation/fixed_registry.rs",
            fixed_registry.as_str(),
        ),
        (
            "scene/tests/ecs_reflect/foundation/fixed_render_physics.rs",
            fixed_render_physics.as_str(),
        ),
        (
            "scene/tests/ecs_reflect/foundation/fixed_transform_active.rs",
            fixed_transform_active.as_str(),
        ),
        (
            "scene/tests/ecs_reflect/foundation/registry.rs",
            registry.as_str(),
        ),
        (
            "scene/tests/ecs_reflect/foundation/value_conversion.rs",
            value_conversion.as_str(),
        ),
        (
            "scene/tests/ecs_reflect/foundation/versioned_json.rs",
            versioned_json.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene ECS doc", ecs_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 scene ECS reflect foundation test folder split",
                "runtime_15_scene_ecs_reflect_foundation_tests_folder_split_static_passed_cargo_deferred",
                "scene/tests/ecs_reflect/foundation.rs",
                "scene/tests/ecs_reflect/foundation/value_conversion.rs",
                "scene/tests/ecs_reflect/foundation/fixed_render_physics.rs",
                "runtime_15_scene_ecs_reflect_foundation_tests_are_folder_backed",
            ],
        );
    }
}
