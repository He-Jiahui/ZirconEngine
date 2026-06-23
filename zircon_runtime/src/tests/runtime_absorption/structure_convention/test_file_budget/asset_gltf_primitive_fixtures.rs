use super::*;

#[test]
fn runtime_15_asset_gltf_primitive_fixtures_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/assets/gltf_primitive_fixtures.rs");
    let basic = read_runtime_src("asset/tests/assets/gltf_primitive_fixtures/basic.rs");
    let vertex_channels =
        read_runtime_src("asset/tests/assets/gltf_primitive_fixtures/vertex_channels.rs");
    let materials = read_runtime_src("asset/tests/assets/gltf_primitive_fixtures/materials.rs");
    let animation = read_runtime_src("asset/tests/assets/gltf_primitive_fixtures/animation.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "asset glTF primitive fixture parent mounts fixture owners",
        &parent,
        &[
            "mod animation;",
            "mod basic;",
            "mod materials;",
            "mod vertex_channels;",
            "pub(super) use animation::write_node_animation_gltf;",
            "pub(super) use basic::{write_line_gltf, write_triangle_gltf};",
            "pub(super) use materials::{write_texture_transform_triangle_gltf, write_two_primitive_gltf};",
            "pub(super) use vertex_channels::{",
        ],
    );
    for moved_fixture in [
        "pub(super) fn write_triangle_gltf",
        "pub(super) fn write_tangent_color_triangle_gltf",
        "pub(super) fn write_uv_channel_triangle_gltf",
        "pub(super) fn write_texture_transform_triangle_gltf",
        "pub(super) fn write_line_gltf",
        "pub(super) fn write_two_primitive_gltf",
        "pub(super) fn write_skinned_triangle_gltf",
        "pub(super) fn write_node_animation_gltf",
    ] {
        assert!(
            !parent.contains(moved_fixture),
            "asset/tests/assets/gltf_primitive_fixtures.rs should mount child fixture owners instead of defining {moved_fixture}"
        );
    }
    let child_sources = [
        basic.as_str(),
        vertex_channels.as_str(),
        materials.as_str(),
        animation.as_str(),
    ];
    assert_eq!(
        child_sources
            .iter()
            .map(|source| source.matches("pub(super) fn ").count())
            .sum::<usize>(),
        8,
        "asset glTF primitive fixture children should preserve the original 8 fixture writers"
    );

    assert_contains_all(
        "asset glTF primitive basic fixture child owns triangle/topology writers",
        &basic,
        &[
            "pub(super) fn write_triangle_gltf",
            "pub(super) fn write_line_gltf",
        ],
    );
    assert_contains_all(
        "asset glTF primitive vertex child owns vertex-channel writers",
        &vertex_channels,
        &[
            "pub(super) fn write_tangent_color_triangle_gltf",
            "pub(super) fn write_uv_channel_triangle_gltf",
            "pub(super) fn write_skinned_triangle_gltf",
        ],
    );
    assert_contains_all(
        "asset glTF primitive material child owns material fixture writers",
        &materials,
        &[
            "pub(super) fn write_texture_transform_triangle_gltf",
            "pub(super) fn write_two_primitive_gltf",
        ],
    );
    assert_contains_all(
        "asset glTF primitive animation child owns animation fixture writer",
        &animation,
        &["pub(super) fn write_node_animation_gltf"],
    );

    for source in [parent.as_str()].into_iter().chain(child_sources) {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "asset glTF primitive fixture parent and child owners should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset glTF primitive fixture folder split",
                "runtime_15_asset_gltf_primitive_fixtures_folder_split_static_passed_cargo_lock_blocked",
                "asset/tests/assets/gltf_primitive_fixtures.rs",
                "asset/tests/assets/gltf_primitive_fixtures/vertex_channels.rs",
                "runtime_15_asset_gltf_primitive_fixtures_are_folder_backed",
            ],
        );
    }
}
