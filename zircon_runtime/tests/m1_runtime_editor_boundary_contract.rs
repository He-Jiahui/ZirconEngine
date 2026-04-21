use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    FallbackSkyboxKind, OverlayLineSegment, OverlayPickShape, PreviewEnvironmentExtract,
    ProjectionMode, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
    SceneGizmoKind, SceneGizmoOverlayExtract, SceneViewportExtractRequest, ViewportCameraSnapshot,
    ViewportRenderSettings,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3, Vec4};
use zircon_runtime::graphics::SceneRenderer;
use zircon_runtime::scene::world::World;

#[test]
fn runtime_world_render_extract_keeps_authoring_overlay_fields_defaulted() {
    let source = read_repo_file("zircon_runtime/src/scene/world/render.rs");

    assert!(
        source.contains("overlays: RenderOverlayExtract {"),
        "runtime world render extract should still build a neutral overlay DTO"
    );
    assert!(
        source.contains("display_mode: request.settings.display_mode"),
        "runtime world render extract should forward host-controlled display mode"
    );
    assert!(
        source.contains("active_camera_override: None"),
        "runtime world default extract should not retain any editor-owned camera override"
    );
    assert!(
        source.contains("camera: None"),
        "runtime world default extract should not inject an editor-owned camera snapshot"
    );
    assert!(
        source.contains("..RenderOverlayExtract::default()"),
        "runtime world render extract should default the remaining authoring overlay fields"
    );
    for forbidden in [
        "SelectionHighlightExtract",
        "SelectionAnchorExtract",
        "GridOverlayExtract",
        "HandleOverlayExtract",
        "SceneGizmoOverlayExtract",
    ] {
        assert!(
            !source.contains(forbidden),
            "runtime world render extract should not construct authoring overlay payloads directly; found {forbidden}"
        );
    }
}

#[test]
fn editor_viewport_render_packet_remains_authoring_overlay_owner() {
    let source = read_repo_file("zircon_editor/src/scene/viewport/render_packet.rs");

    for required in [
        "settings: settings.render_settings()",
        "selection: build_selection_highlights",
        "selection_anchors: build_selection_anchors",
        "grid: build_grid_extract",
        "handles,",
        "scene_gizmos: build_scene_gizmos",
        "camera: Some(camera.clone())",
        "lighting_enabled: settings.preview_lighting",
        "skybox_enabled: settings.preview_skybox",
    ] {
        assert!(
            source.contains(required),
            "editor viewport render packet should continue to own authoring overlay assembly; missing `{required}`"
        );
    }
}

#[test]
fn runtime_world_render_packet_only_applies_neutral_request_fields() {
    let world = World::new();
    let request = SceneViewportExtractRequest {
        settings: ViewportRenderSettings {
            projection_mode: ProjectionMode::Orthographic,
            display_mode: zircon_runtime::core::framework::render::DisplayMode::WireOverlay,
            preview_lighting: false,
            preview_skybox: false,
        },
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(640, 360)),
        virtual_geometry_debug: None,
    };

    let packet = world.build_viewport_render_packet(&request);

    assert_eq!(
        packet.scene.camera.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(
        packet.overlays.display_mode,
        zircon_runtime::core::framework::render::DisplayMode::WireOverlay
    );
    assert!(
        packet.overlays.selection.is_empty()
            && packet.overlays.selection_anchors.is_empty()
            && packet.overlays.grid.is_none()
            && packet.overlays.handles.is_empty()
            && packet.overlays.scene_gizmos.is_empty(),
        "runtime world render packet should only project neutral scene data plus neutral render settings"
    );
    assert!(
        !packet.preview.lighting_enabled
            && !packet.preview.skybox_enabled
            && packet.preview.fallback_skybox == FallbackSkyboxKind::None,
        "runtime world render packet should only mirror neutral preview request defaults instead of injecting editor authoring preview state"
    );
}

#[test]
fn authoring_overlay_constructors_stay_out_of_runtime_production_sources() {
    let repo_root = repo_root();
    let runtime_root = repo_root.join("zircon_runtime").join("src");
    let editor_root = repo_root.join("zircon_editor").join("src");

    assert_no_matches(
        &runtime_root,
        "SelectionHighlightExtract {",
        &[runtime_root
            .join("core")
            .join("framework")
            .join("render")
            .join("overlay.rs")],
    );
    assert_no_matches(
        &runtime_root,
        "SelectionAnchorExtract {",
        &[runtime_root
            .join("core")
            .join("framework")
            .join("render")
            .join("overlay.rs")],
    );
    assert_no_matches(
        &runtime_root,
        "GridOverlayExtract {",
        &[runtime_root
            .join("core")
            .join("framework")
            .join("render")
            .join("overlay.rs")],
    );
    assert_no_matches(
        &runtime_root,
        "HandleOverlayExtract {",
        &[runtime_root
            .join("core")
            .join("framework")
            .join("render")
            .join("overlay.rs")],
    );

    assert_expected_matches(
        &editor_root,
        "SelectionHighlightExtract {",
        &[editor_root
            .join("scene")
            .join("viewport")
            .join("render_packet.rs")],
    );
    assert_expected_matches(
        &editor_root,
        "SelectionAnchorExtract {",
        &[editor_root
            .join("scene")
            .join("viewport")
            .join("render_packet.rs")],
    );
    assert_expected_matches(
        &editor_root,
        "GridOverlayExtract {",
        &[editor_root
            .join("scene")
            .join("viewport")
            .join("render_packet.rs")],
    );
    assert_expected_matches(
        &editor_root,
        "HandleOverlayExtract {",
        &[
            editor_root
                .join("scene")
                .join("viewport")
                .join("handles")
                .join("move_handle_tool_impl.rs"),
            editor_root
                .join("scene")
                .join("viewport")
                .join("handles")
                .join("rotate_handle_tool_impl.rs"),
            editor_root
                .join("scene")
                .join("viewport")
                .join("handles")
                .join("scale_handle_tool_impl.rs"),
        ],
    );
}

#[test]
fn scene_gizmo_overlay_constructors_are_limited_to_editor_viewport_and_runtime_debug_submission() {
    let repo_root = repo_root();
    let runtime_root = repo_root.join("zircon_runtime").join("src");
    let editor_root = repo_root.join("zircon_editor").join("src");

    assert_expected_matches(
        &runtime_root,
        "SceneGizmoOverlayExtract {",
        &[
            runtime_root
                .join("core")
                .join("framework")
                .join("render")
                .join("overlay.rs"),
            runtime_root
                .join("graphics")
                .join("runtime")
                .join("render_framework")
                .join("submit_frame_extract")
                .join("submit")
                .join("build_runtime_frame.rs"),
        ],
    );
    assert_expected_matches(
        &editor_root,
        "SceneGizmoOverlayExtract {",
        &[editor_root
            .join("scene")
            .join("viewport")
            .join("render_packet.rs")],
    );
}

#[test]
fn neutral_overlay_packet_renders_scene_gizmo_without_runtime_world_context() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let viewport_size = UVec2::new(320, 240);
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let frame = renderer
        .render(overlay_only_snapshot(viewport_size), viewport_size)
        .unwrap();

    let green_pixels = dominant_green_pixels(&frame.rgba);
    assert!(
        green_pixels > 24,
        "expected overlay-only neutral scene gizmo DTOs to render without runtime-world authoring state; green_pixels={green_pixels}"
    );
}

fn overlay_only_snapshot(viewport_size: UVec2) -> RenderSceneSnapshot {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            scene_gizmos: vec![SceneGizmoOverlayExtract {
                owner: 77,
                kind: SceneGizmoKind::DirectionalLight,
                selected: false,
                lines: vec![
                    OverlayLineSegment {
                        start: Vec3::new(-0.65, 0.0, 0.0),
                        end: Vec3::new(0.65, 0.0, 0.0),
                        color: Vec4::new(0.1, 1.0, 0.22, 1.0),
                    },
                    OverlayLineSegment {
                        start: Vec3::new(0.0, -0.65, 0.0),
                        end: Vec3::new(0.0, 0.65, 0.0),
                        color: Vec4::new(0.1, 1.0, 0.22, 1.0),
                    },
                ],
                wire_shapes: Vec::new(),
                icons: Vec::new(),
                pick_shapes: vec![OverlayPickShape::Sphere {
                    center: Vec3::ZERO,
                    radius: 0.3,
                }],
            }],
            ..RenderOverlayExtract::default()
        },
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}

fn dominant_green_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] == 255 && pixel[1] > 20 && pixel[1] > pixel[0] + 8)
        .count()
}

fn read_repo_file(relative: &str) -> String {
    std::fs::read_to_string(repo_root().join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should live under the repo root")
        .to_path_buf()
}

fn assert_no_matches(root: &Path, needle: &str, allowed: &[PathBuf]) {
    let matches = collect_matches(root, needle);
    let allowed = normalize_paths(allowed);
    let unexpected = matches
        .into_iter()
        .filter(|path| !allowed.contains(path))
        .collect::<Vec<_>>();
    assert!(
        unexpected.is_empty(),
        "expected `{needle}` to stay out of {} outside {:?}, found {:?}",
        root.display(),
        allowed,
        unexpected
    );
}

fn assert_expected_matches(root: &Path, needle: &str, expected: &[PathBuf]) {
    let actual = normalize_paths(&collect_matches(root, needle));
    let expected = normalize_paths(expected);
    assert_eq!(
        actual,
        expected,
        "expected `{needle}` matches under {} to stay locked to {:?}, found {:?}",
        root.display(),
        expected,
        actual
    );
}

fn normalize_paths(paths: &[PathBuf]) -> BTreeSet<PathBuf> {
    paths.iter().map(|path| normalize_path(path)).collect()
}

fn normalize_path(path: &Path) -> PathBuf {
    path.components().collect()
}

fn collect_matches(root: &Path, needle: &str) -> Vec<PathBuf> {
    let mut matches = Vec::new();
    collect_matches_recursive(root, needle, &mut matches);
    matches
}

fn collect_matches_recursive(root: &Path, needle: &str, matches: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == "tests" || name == "target")
            {
                continue;
            }
            collect_matches_recursive(&path, needle, matches);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs")
            && std::fs::read_to_string(&path)
                .map(|source| source.contains(needle))
                .unwrap_or(false)
        {
            matches.push(normalize_path(&path));
        }
    }
}
