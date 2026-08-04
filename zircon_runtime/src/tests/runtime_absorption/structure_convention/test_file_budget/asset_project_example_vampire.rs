use super::*;

#[test]
fn runtime_15_asset_project_example_vampire_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/project/example_vampire.rs");
    let manifest_scene_imports =
        read_runtime_src("asset/tests/project/example_vampire/manifest_scene_imports.rs");
    let third_person_render_extract =
        read_runtime_src("asset/tests/project/example_vampire/third_person_render_extract.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let vampire_doc = read_repo("docs/assets-and-rendering/runtime-physics-animation-assets.md");

    assert_contains_all(
        "asset project example vampire parent test module mounts",
        &parent,
        &[
            "mod manifest_scene_imports;",
            "mod third_person_render_extract;",
            "fn vampire_root() -> PathBuf",
        ],
    );
    for moved_test in [
        "fn vampire_example_manifest_scene_and_scripts_are_importable",
        "fn vampire_example_scene_extracts_playable_third_person_meshes",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/project/example_vampire.rs should mount child owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/project/example_vampire.rs should not keep executable tests in the parent module"
    );
    assert_eq!(
        manifest_scene_imports.matches("#[test]").count(),
        1,
        "example vampire manifest/scene import child should preserve the original import test"
    );
    assert_eq!(
        third_person_render_extract.matches("#[test]").count(),
        1,
        "example vampire render-extract child should preserve the original render-extract test"
    );

    assert_contains_all(
        "example vampire manifest child owns project asset import coverage",
        &manifest_scene_imports,
        &[
            "use super::vampire_root;",
            "fn vampire_example_manifest_scene_and_scripts_are_importable",
            "discover_vm_plugin_packages",
            "ProjectManager::open",
        ],
    );
    assert_contains_all(
        "example vampire render child owns third-person frame extraction coverage",
        &third_person_render_extract,
        &[
            "use super::vampire_root;",
            "fn vampire_example_scene_extracts_playable_third_person_meshes",
            "World::load_scene_from_uri",
            "FallbackSkyboxKind::ProceduralGradient",
        ],
    );

    for (path, source) in [
        ("asset/tests/project/example_vampire.rs", parent.as_str()),
        (
            "asset/tests/project/example_vampire/manifest_scene_imports.rs",
            manifest_scene_imports.as_str(),
        ),
        (
            "asset/tests/project/example_vampire/third_person_render_extract.rs",
            third_person_render_extract.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
