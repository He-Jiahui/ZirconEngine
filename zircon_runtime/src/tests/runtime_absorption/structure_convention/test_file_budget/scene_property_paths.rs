use super::*;

#[test]
fn runtime_15_scene_property_paths_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/property_paths.rs");
    let read_paths = read_runtime_src("scene/tests/property_paths/read_paths.rs");
    let runtime_mutation = read_runtime_src("scene/tests/property_paths/runtime_mutation.rs");
    let write_validation = read_runtime_src("scene/tests/property_paths/write_validation.rs");

    assert_contains_all(
        "scene property paths parent keeps shared imports and mounts children",
        &parent,
        &[
            "mod read_paths;",
            "mod runtime_mutation;",
            "mod write_validation;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/property_paths.rs should only keep shared imports and mount child owners"
    );
    for moved_test in [
        "fn world_resolves_entity_paths_and_mutates_component_properties",
        "fn world_entity_paths_suffix_duplicate_sibling_names",
        "fn world_property_reads_compare_normalized_segments_without_entry_vector_allocation",
        "fn component_property_path_constructor_pre_sizes_raw_path_buffer",
        "fn world_entity_path_resolution_compares_target_segments_directly",
        "fn world_property_entries_pre_size_projection_vector",
        "fn world_property_dynamic_json_number_projection_uses_direct_branches",
        "fn world_property_writes_use_direct_optional_state_branches",
        "fn world_property_writes_pre_size_normalized_segment_vector",
        "fn world_collider_shape_kind_write_matches_normalized_values_without_allocation",
        "fn world_property_write_segment_expectation_uses_direct_candidate_loop",
        "fn world_transform_rotation_validation_sums_quaternion_length_directly",
        "fn world_property_numeric_array_validation_uses_direct_finite_loop",
        "fn world_property_enum_parsers_match_normalized_values_without_allocation",
        "fn world_property_write_normalizer_pushes_identifier_characters_directly",
        "fn world_property_value_conversion_errors_use_direct_result_branches",
        "fn world_rejects_zero_length_transform_rotation_property_writes",
        "fn world_rejects_non_finite_transform_property_writes",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved scene property-path test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "runtime mutation child owns entity path and property mutation behavior",
        &runtime_mutation,
        &[
            "fn world_resolves_entity_paths_and_mutates_component_properties",
            "fn world_entity_paths_suffix_duplicate_sibling_names",
            "fn world_rejects_zero_length_transform_rotation_property_writes",
            "fn world_rejects_non_finite_transform_property_writes",
        ],
    );
    assert_contains_all(
        "read paths child owns read-side source guards",
        &read_paths,
        &[
            "fn world_property_reads_compare_normalized_segments_without_entry_vector_allocation",
            "fn component_property_path_constructor_pre_sizes_raw_path_buffer",
            "fn world_entity_path_resolution_compares_target_segments_directly",
            "fn world_property_entries_pre_size_projection_vector",
            "fn world_property_dynamic_json_number_projection_uses_direct_branches",
        ],
    );
    assert_contains_all(
        "write validation child owns write-side source guards",
        &write_validation,
        &[
            "fn world_property_writes_use_direct_optional_state_branches",
            "fn world_property_writes_pre_size_normalized_segment_vector",
            "fn world_collider_shape_kind_write_matches_normalized_values_without_allocation",
            "fn world_property_write_segment_expectation_uses_direct_candidate_loop",
            "fn world_transform_rotation_validation_sums_quaternion_length_directly",
            "fn world_property_numeric_array_validation_uses_direct_finite_loop",
            "fn world_property_enum_parsers_match_normalized_values_without_allocation",
            "fn world_property_write_normalizer_pushes_identifier_characters_directly",
            "fn world_property_value_conversion_errors_use_direct_result_branches",
        ],
    );
    let child_test_total = [
        read_paths.as_str(),
        runtime_mutation.as_str(),
        write_validation.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 18,
        "scene property-path children should preserve all 18 parent tests"
    );

    for (path, source) in [
        ("scene/tests/property_paths.rs", parent.as_str()),
        (
            "scene/tests/property_paths/read_paths.rs",
            read_paths.as_str(),
        ),
        (
            "scene/tests/property_paths/runtime_mutation.rs",
            runtime_mutation.as_str(),
        ),
        (
            "scene/tests/property_paths/write_validation.rs",
            write_validation.as_str(),
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
    let scene_ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");
}
