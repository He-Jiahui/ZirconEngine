use super::*;

#[test]
fn runtime_15_asset_mesh_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/assets/mesh.rs");
    let document_roundtrip = read_runtime_src("asset/tests/assets/mesh/document_roundtrip.rs");
    let validation = read_runtime_src("asset/tests/assets/mesh/validation.rs");
    let summaries = read_runtime_src("asset/tests/assets/mesh/summaries.rs");
    let conversion_import = read_runtime_src("asset/tests/assets/mesh/conversion_import.rs");
    let morph_targets = read_runtime_src("asset/tests/assets/mesh/morph_targets.rs");
    let normal_generation = read_runtime_src("asset/tests/assets/mesh/normal_generation.rs");
    let tangent_generation = read_runtime_src("asset/tests/assets/mesh/tangent_generation.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_asset_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");

    assert_contains_all(
        "asset mesh parent test module mounts",
        &parent,
        &[
            "mod conversion_import;",
            "mod document_roundtrip;",
            "mod morph_targets;",
            "mod normal_generation;",
            "mod summaries;",
            "mod tangent_generation;",
            "mod validation;",
            "fn sample_zmesh_document",
            "fn triangle_attributes",
            "fn quad_attributes",
            "fn sample_virtual_geometry",
        ],
    );
    for moved_test in [
        "fn zmesh_document_roundtrip_preserves_mesh_payload",
        "fn zmesh_document_roundtrip_preserves_morph_targets_and_skin_inverse_bindposes",
        "fn mesh_asset_rejects_missing_position_attribute",
        "fn mesh_asset_rejects_attribute_length_mismatch",
        "fn mesh_asset_rejects_builtin_attribute_format_mismatch",
        "fn mesh_asset_allows_custom_attribute_formats_when_lengths_match",
        "fn mesh_asset_rejects_out_of_range_indices",
        "fn mesh_asset_rejects_incomplete_list_topology_elements",
        "fn mesh_asset_reports_index_format_without_expanding_indices",
        "fn mesh_asset_reports_draw_element_and_primitive_counts_without_descriptor",
        "fn mesh_asset_reports_attribute_summaries_without_value_inspection",
        "fn mesh_asset_overview_reports_editor_ready_mesh_summary",
        "fn mesh_asset_management_record_wraps_id_and_strict_overview",
        "fn mesh_asset_management_record_set_summarizes_valid_and_invalid_rows",
        "fn model_primitive_converts_to_mesh_asset_with_builtin_attributes",
        "fn static_virtual_geometry_primitive_converts_to_mesh_asset_with_ordinal_channels",
        "fn mesh_render_descriptor_uses_bounds_topology_and_indices",
        "fn mesh_asset_bounds_can_be_read_without_render_descriptor",
        "fn mesh_asset_try_render_descriptor_reports_validation_errors",
        "fn default_importer_routes_zmesh_to_mesh_asset",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/assets/mesh.rs should mount child test owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/assets/mesh.rs should not keep executable tests in the parent module"
    );

    let migrated_child_sources = [
        document_roundtrip.as_str(),
        validation.as_str(),
        summaries.as_str(),
        conversion_import.as_str(),
    ];
    assert_eq!(
        migrated_child_sources
            .iter()
            .map(|source| source.matches("#[test]").count())
            .sum::<usize>(),
        20,
        "asset mesh migrated child modules should preserve the current 20 parent tests"
    );

    assert_contains_all(
        "asset mesh document child owns zmesh roundtrip contracts",
        &document_roundtrip,
        &[
            "use super::*;",
            "fn zmesh_document_roundtrip_preserves_mesh_payload",
            "fn zmesh_document_roundtrip_preserves_morph_targets_and_skin_inverse_bindposes",
        ],
    );
    assert_contains_all(
        "asset mesh validation child owns strict mesh validation contracts",
        &validation,
        &[
            "use super::*;",
            "fn mesh_asset_rejects_missing_position_attribute",
            "fn mesh_asset_rejects_builtin_attribute_format_mismatch",
            "fn mesh_asset_allows_custom_attribute_formats_when_lengths_match",
        ],
    );
    assert_contains_all(
        "asset mesh summaries child owns editor and management summaries",
        &summaries,
        &[
            "use super::*;",
            "fn mesh_asset_reports_index_format_without_expanding_indices",
            "fn mesh_asset_overview_reports_editor_ready_mesh_summary",
            "fn mesh_asset_management_record_set_summarizes_valid_and_invalid_rows",
        ],
    );
    assert_contains_all(
        "asset mesh conversion child owns model/import contracts",
        &conversion_import,
        &[
            "use super::*;",
            "fn model_primitive_converts_to_mesh_asset_with_builtin_attributes",
            "fn static_virtual_geometry_primitive_converts_to_mesh_asset_with_ordinal_channels",
            "fn mesh_render_descriptor_uses_bounds_topology_and_indices",
            "fn default_importer_routes_zmesh_to_mesh_asset",
        ],
    );

    for (path, source) in [
        ("asset/tests/assets/mesh.rs", parent.as_str()),
        (
            "asset/tests/assets/mesh/document_roundtrip.rs",
            document_roundtrip.as_str(),
        ),
        ("asset/tests/assets/mesh/validation.rs", validation.as_str()),
        ("asset/tests/assets/mesh/summaries.rs", summaries.as_str()),
        (
            "asset/tests/assets/mesh/conversion_import.rs",
            conversion_import.as_str(),
        ),
        (
            "asset/tests/assets/mesh/morph_targets.rs",
            morph_targets.as_str(),
        ),
        (
            "asset/tests/assets/mesh/normal_generation.rs",
            normal_generation.as_str(),
        ),
        (
            "asset/tests/assets/mesh/tangent_generation.rs",
            tangent_generation.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
