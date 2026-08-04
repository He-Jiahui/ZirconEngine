use super::*;

#[test]
fn runtime_15_asset_scene_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/assets/scene.rs");
    let foundation = read_runtime_src("asset/tests/assets/scene/foundation.rs");
    let camera = read_runtime_src("asset/tests/assets/scene/camera.rs");
    let post_process = read_runtime_src("asset/tests/assets/scene/post_process.rs");
    let physics_animation = read_runtime_src("asset/tests/assets/scene/physics_animation.rs");
    let lights = read_runtime_src("asset/tests/assets/scene/lights.rs");
    let script_bindings = read_runtime_src("asset/tests/assets/scene/script_bindings.rs");
    let management = read_runtime_src("asset/tests/assets/scene/management.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "asset scene parent test module mounts",
        &parent,
        &[
            "mod camera;",
            "mod foundation;",
            "mod lights;",
            "mod management;",
            "mod physics_animation;",
            "mod post_process;",
            "mod script_bindings;",
        ],
    );
    for moved_test in [
        "fn scene_asset_toml_roundtrip_preserves_entities_and_bindings",
        "fn scene_camera_asset_roundtrip_preserves_bevy_style_camera_fields",
        "fn scene_camera_asset_defaults_bevy_camera_fields_when_omitted",
        "fn scene_asset_toml_roundtrip_preserves_post_process_components",
        "fn scene_asset_toml_roundtrip_preserves_physics_and_animation_components",
        "fn scene_asset_parses_uuid_url_mesh_bindings",
        "fn scene_asset_defaults_new_runtime_foundation_fields_when_omitted",
        "fn scene_asset_toml_roundtrip_preserves_point_and_spot_lights",
        "fn scene_asset_toml_roundtrip_preserves_ambient_and_rect_lights",
        "fn scene_asset_toml_roundtrip_preserves_script_bindings",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/assets/scene.rs should mount child test owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/assets/scene.rs should not keep executable tests in the parent module"
    );
    let migrated_child_sources = [
        foundation.as_str(),
        camera.as_str(),
        post_process.as_str(),
        physics_animation.as_str(),
        lights.as_str(),
        script_bindings.as_str(),
    ];
    assert_eq!(
        migrated_child_sources
            .iter()
            .map(|source| source.matches("#[test]").count())
            .sum::<usize>(),
        10,
        "asset scene migrated child modules should preserve the original 10 parent tests"
    );

    assert_contains_all(
        "asset scene foundation child owns scene serialization defaults",
        &foundation,
        &[
            "use super::*;",
            "fn scene_asset_toml_roundtrip_preserves_entities_and_bindings",
            "fn scene_asset_parses_uuid_url_mesh_bindings",
            "fn scene_asset_defaults_new_runtime_foundation_fields_when_omitted",
        ],
    );
    assert_contains_all(
        "asset scene camera child owns camera contracts",
        &camera,
        &[
            "use super::*;",
            "fn scene_camera_asset_roundtrip_preserves_bevy_style_camera_fields",
            "fn scene_camera_asset_defaults_bevy_camera_fields_when_omitted",
        ],
    );
    assert_contains_all(
        "asset scene post-process child owns post-process contracts",
        &post_process,
        &[
            "use super::*;",
            "fn scene_asset_toml_roundtrip_preserves_post_process_components",
            "ScenePostProcessVolumeProfileAsset",
        ],
    );
    assert_contains_all(
        "asset scene physics-animation child owns physics and animation contracts",
        &physics_animation,
        &[
            "use super::*;",
            "fn scene_asset_toml_roundtrip_preserves_physics_and_animation_components",
            "AnimationParameterValue::Scalar",
        ],
    );
    assert_contains_all(
        "asset scene lights child owns point/spot/ambient/rect contracts",
        &lights,
        &[
            "use super::*;",
            "fn scene_asset_toml_roundtrip_preserves_point_and_spot_lights",
            "fn scene_asset_toml_roundtrip_preserves_ambient_and_rect_lights",
        ],
    );
    assert_contains_all(
        "asset scene script child owns script binding contracts",
        &script_bindings,
        &[
            "use super::*;",
            "fn scene_asset_toml_roundtrip_preserves_script_bindings",
            "vampire_game",
        ],
    );
    assert_contains_all(
        "asset scene management child remains a dedicated owner",
        &management,
        &[
            "fn scene_asset_overview_reports_entity_component_and_reference_counts",
            "fn scene_asset_overview_handles_empty_scenes",
            "fn scene_asset_management_record_set_sorts_and_summarizes_records",
        ],
    );

    for source in [parent.as_str(), management.as_str()]
        .into_iter()
        .chain(migrated_child_sources)
    {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "asset scene parent and child test owners should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
