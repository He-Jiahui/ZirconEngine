use super::*;

#[test]
fn runtime_15_scene_components_light_postprocess_are_child_owners() {
    let parent = read_runtime_src("scene/components/scene.rs");
    let lighting = read_runtime_src("scene/components/scene/lighting.rs");
    let post_process = read_runtime_src("scene/components/scene/post_process.rs");

    assert_contains_all(
        "scene component parent mounts and re-exports child owners",
        &parent,
        &[
            "mod lighting;",
            "mod post_process;",
            "pub use self::lighting::{AmbientLight, DirectionalLight, PointLight, RectLight, SpotLight};",
            "pub use self::post_process::{PostProcessSettingsComponent, PostProcessVolumeComponent};",
        ],
    );
    for moved_declaration in [
        "pub struct AmbientLight {",
        "pub struct DirectionalLight {",
        "pub struct PointLight {",
        "pub struct RectLight {",
        "pub struct SpotLight {",
        "pub struct PostProcessSettingsComponent {",
        "pub struct PostProcessVolumeComponent {",
    ] {
        assert!(
            !parent.contains(moved_declaration),
            "scene/components/scene.rs should re-export `{moved_declaration}` from child owners instead of owning it directly"
        );
    }
    assert_contains_all(
        "lighting child owns fixed scene light components",
        &lighting,
        &[
            "pub struct AmbientLight",
            "pub struct DirectionalLight",
            "pub struct PointLight",
            "pub struct RectLight",
            "pub struct SpotLight",
            "impl Default for AmbientLight",
            "impl Default for RectLight",
        ],
    );
    assert_contains_all(
        "post-process child owns scene post-process components",
        &post_process,
        &[
            "pub struct PostProcessSettingsComponent",
            "pub struct PostProcessVolumeComponent",
            "impl Default for PostProcessSettingsComponent",
            "impl PostProcessVolumeComponent",
            "pub fn global",
            "pub fn local",
            "pub const fn with_weight",
        ],
    );

    for (path, source) in [
        ("scene/components/scene.rs", parent.as_str()),
        ("scene/components/scene/lighting.rs", lighting.as_str()),
        (
            "scene/components/scene/post_process.rs",
            post_process.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let scene_ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");
    let render_extract_doc = read_repo("docs/zircon_runtime/scene/render_extract.md");
}
