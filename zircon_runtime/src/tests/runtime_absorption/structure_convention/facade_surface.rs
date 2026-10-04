use super::{assert_contains_all, runtime_src_path};

#[test]
fn runtime_15_prelude_covers_required_types() {
    let crate_prelude = read_runtime_src("prelude.rs");
    let asset_mod = read_runtime_src("asset/mod.rs");
    let scene_mod = read_runtime_src("scene/mod.rs");
    let ui_mod = read_runtime_src("ui/mod.rs");
    let graphics_mod = read_runtime_src("graphics/mod.rs");
    let asset_prelude = read_runtime_src("asset/prelude.rs");
    let scene_prelude = read_runtime_src("scene/prelude.rs");
    let ui_prelude = read_runtime_src("ui/prelude.rs");
    let graphics_prelude = read_runtime_src("graphics/prelude.rs");
    let prelude_tests = read_runtime_src("tests/prelude.rs");

    assert_contains_all(
        "crate prelude subsystem aggregation",
        &crate_prelude,
        &[
            "pub use crate::asset::prelude::*;",
            "pub use crate::scene::prelude::*;",
            "pub use crate::ui::prelude::*;",
            "pub use crate::graphics::prelude::*;",
        ],
    );

    for (label, module_source) in [
        ("asset", asset_mod.as_str()),
        ("scene", scene_mod.as_str()),
        ("ui", ui_mod.as_str()),
        ("graphics", graphics_mod.as_str()),
    ] {
        assert!(
            module_source.contains("pub mod prelude;"),
            "{label} module should expose a subsystem prelude before crate-level aggregation"
        );
    }

    assert_contains_all(
        "asset prelude required gameplay and authoring imports",
        &asset_prelude,
        &[
            "TextureAssetDescriptor",
            "AssetLoadState",
            "Assets",
            "Handle",
            "AssetManager",
            "ProjectAssetManager",
            "RGBA8_UNORM_SRGB_FORMAT",
        ],
    );
    assert_contains_all(
        "scene prelude required ECS imports",
        &scene_prelude,
        &[
            "World",
            "EntityId",
            "Bundle",
            "Component",
            "Resource",
            "Commands",
            "Query",
            "Res",
            "ResMut",
            "SceneError",
            "SceneResult",
            "SystemStage",
            "Schedule",
        ],
    );
    assert_contains_all(
        "ui prelude required surface/template imports",
        &ui_prelude,
        &[
            "UiSurface",
            "UiTreeId",
            "UiConfig",
            "UiModule",
            "UiAssetLoader",
            "UiDocumentCompiler",
            "UiV2DocumentCompiler",
            "UiV2SurfaceBuilder",
        ],
    );
    assert_contains_all(
        "graphics prelude required render imports",
        &graphics_prelude,
        &[
            "GraphicsModule",
            "WgpuRenderFramework",
            "RenderPipelineAsset",
            "RenderFeatureDescriptor",
            "ViewportFrame",
            "ViewportRenderRegion",
            "GraphicsError",
        ],
    );
    assert_contains_all(
        "prelude behavior test",
        &prelude_tests,
        &["runtime_prelude_exports_asset_scene_ui_and_graphics_contracts"],
    );
}

#[test]
fn runtime_15_mixed_visibility_has_facade_note() {
    let graphics_mod = read_runtime_src("graphics/mod.rs");

    assert_contains_all(
        "graphics facade visibility notes",
        &graphics_mod,
        &[
            "Crate-private implementation owners",
            "Public module entries",
            "Public facade exports",
            "Crate-visible bridge",
            "Test-only access",
            "pub(crate) mod backend;",
            "pub(crate) mod scene;",
            "pub mod prelude;",
            "pub mod runtime_builtin_graphics;",
        ],
    );
    for public_leak in ["pub mod backend;", "pub mod scene;", "pub mod types;"] {
        assert!(
            !graphics_mod.contains(public_leak),
            "graphics facade should not expose implementation module entry {public_leak}"
        );
    }
}

#[test]
fn runtime_15_facades_do_not_hide_unused_imports_or_stale_forwarding_exports() {
    let asset_mod = read_runtime_src("asset/mod.rs");
    let graphics_backend_mod = read_runtime_src("graphics/backend/mod.rs");
    let scene_mod = read_runtime_src("scene/mod.rs");

    for (label, source) in [
        ("asset facade", asset_mod.as_str()),
        ("graphics backend facade", graphics_backend_mod.as_str()),
        ("scene facade", scene_mod.as_str()),
    ] {
        let compact: String = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        assert!(
            !compact.contains("#[allow(unused_imports)]"),
            "{label} should expose only live curated imports instead of suppressing unused-import diagnostics"
        );
    }

    for stale_export in [
        "AssetRequest",
        "CpuAssetPayload",
        "CpuMeshPayload",
        "CpuTexturePayload",
        "MeshSource",
        "TextureSource",
        "AssetMetaEntry",
        "AssetMetaResult",
        "PackageAssetRegistry",
        "PreviewState",
    ] {
        assert!(
            !asset_mod
                .lines()
                .skip_while(|line| !line.contains("pub use pipeline::types::MeshVertex;"))
                .take_while(|line| !line.contains("pub use project::{ProjectImportReceipt"))
                .any(|line| line.contains(stale_export)),
            "asset root should not retain stale crate-private forwarding export `{stale_export}`"
        );
    }

    assert_contains_all(
        "live asset project facade",
        &asset_mod,
        &[
            "pub(crate) use project::{",
            "AssetMetaDocument, AssetMetaError, AssetSourceUnit, ProjectManager, ProjectPaths,",
        ],
    );
    assert_contains_all(
        "live graphics IBL readback bridge",
        &graphics_backend_mod,
        &[
            "IblBakeArtifactWgpuPendingReadback",
            "IblBakeArtifactWgpuReadbackResources",
            "request_ibl_bake_artifact_wgpu_readback",
        ],
    );
    assert_contains_all(
        "public scene component facade",
        &scene_mod,
        &[
            "default_render_layer_mask",
            "Mobility",
            "NodeKind",
            "NodeRecord",
        ],
    );
}

#[test]
fn runtime_15_facade_surface_guard_is_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let child = read_runtime_src("tests/runtime_absorption/structure_convention/facade_surface.rs");

    assert_contains_all(
        "structure convention parent facade surface mount",
        &parent,
        &[
            "#[path = \"structure_convention/facade_surface.rs\"]",
            "mod facade_surface;",
        ],
    );

    for moved_guard in [
        "fn runtime_15_prelude_covers_required_types",
        "fn runtime_15_mixed_visibility_has_facade_note",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "top-level structure_convention.rs should mount facade surface guards instead of defining {moved_guard}"
        );
        assert!(
            child.contains(moved_guard),
            "facade_surface.rs should own moved guard {moved_guard}"
        );
    }

    let parent_lines = parent.lines().count();
    assert!(
        parent_lines < 500,
        "structure_convention.rs should remain a small aggregator after facade surface split; got {parent_lines} lines"
    );
    let child_lines = child.lines().count();
    assert!(
        child_lines < 700,
        "facade_surface.rs should stay below the local guard module limit; got {child_lines} lines"
    );
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}
