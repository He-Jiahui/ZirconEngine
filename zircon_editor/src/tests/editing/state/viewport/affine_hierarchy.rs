use std::fs;
use std::sync::Arc;

use zircon_runtime::asset::{AssetManager, AssetUri, ProjectAssetManager, ProjectManifest};
use zircon_runtime::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderQualityProfile, RenderViewportDescriptor,
};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime::scene::{NodeKind, Scene};
use zircon_runtime_interface::math::{Quat, Transform, UVec2, Vec3};
use zircon_runtime_interface::ui::layout::UiFrame;

use crate::scene::viewport::{GridMode, ProjectionMode, SceneViewportController};

use super::{count_changed_pixels_in_frame, editor_render_asset_access};

#[test]
fn live_editor_viewport_draws_child_affine_silhouette_offscreen() {
    const SIZE: u32 = 256;
    const FRINGE: (u32, u32) = (185, 126);
    const PATCH: u32 = 4;
    let size = UVec2::splat(SIZE);
    let mut scene = Scene::empty();
    let camera = scene.spawn_node(NodeKind::Camera).unwrap();
    scene.set_active_camera(camera);
    scene
        .update_transform(camera, Transform::default())
        .unwrap();
    scene.spawn_node(NodeKind::DirectionalLight).unwrap();
    let parent = scene.spawn_node(NodeKind::Empty).unwrap();
    let cube = scene.spawn_node(NodeKind::Cube).unwrap();
    scene.set_parent_checked(cube, Some(parent)).unwrap();
    scene
        .update_transform(
            parent,
            Transform::default().with_scale(Vec3::new(4.0, 1.0, 1.0)),
        )
        .unwrap();
    scene
        .update_transform(
            cube,
            Transform::from_translation(Vec3::new(0.0, 0.0, -5.0))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        )
        .unwrap();

    let mut controller = SceneViewportController::new(size);
    let settings = controller.settings_mut();
    settings.projection_mode = ProjectionMode::Orthographic;
    settings.grid_mode = GridMode::Hidden;
    settings.gizmos_enabled = false;
    settings.preview_skybox = false;
    controller.reset_from_scene(Some(&scene));
    let live = controller.build_prepared_render_extract(&mut scene);
    assert_eq!(live.geometry.meshes.len(), 1);
    assert_eq!(live.geometry.meshes[0].node_id, cube);
    let view = &live.view.camera;
    assert_eq!(view.transform, Transform::default());
    assert_eq!(view.projection_mode, ProjectionMode::Orthographic);
    assert_eq!(view.aspect_ratio, 1.0);

    // The +Z face of the authored unit cube projects to the diamond
    // |x|/4 + |y| <= 1/sqrt(2). The 4x4 patch lies inside it but outside
    // the old TRS reconstruction, independently of the renderer output.
    let lossy_inverse = scene.world_transform(cube).unwrap().matrix().inverse();
    for py in FRINGE.1..FRINGE.1 + PATCH {
        for px in FRINGE.0..FRINGE.0 + PATCH {
            let x = (2.0 * (px as f32 + 0.5) / SIZE as f32 - 1.0) * view.ortho_size;
            let y = (1.0 - 2.0 * (py as f32 + 0.5) / SIZE as f32) * view.ortho_size;
            assert!(x.abs() / 4.0 + y.abs() < std::f32::consts::FRAC_1_SQRT_2 - 0.02);
            let old_local = lossy_inverse.transform_point3(Vec3::new(x, y, -4.5));
            assert!(
                old_local.x.abs().max(old_local.y.abs()) > 0.52,
                "{old_local:?}"
            );
        }
    }
    // Runtime is a normal dependency here, so its cfg(test) project-less
    // constructor does not apply. Use a fresh project without authoring assets.
    let project_root =
        std::env::temp_dir().join(format!("zr-editor-affine-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&project_root).unwrap();
    ProjectManifest::new(
        "Editor Affine Viewport Fixture",
        AssetUri::parse("res://scenes/unused.zrscene.json").unwrap(),
        1,
    )
    .save(project_root.join("zircon-project.toml"))
    .unwrap();
    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(project_root.to_str().expect("fixture project path"))
        .expect("activate an isolated project for the offscreen renderer");
    let (asset_runtime, assets) = editor_render_asset_access(asset_manager);
    let framework =
        WgpuRenderFramework::new(assets, asset_runtime.task_graph().worker_pool().clone())
            .expect("hardware offscreen renderer");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(size))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("editor-affine-geometry-witness")
                .with_temporal_history(false)
                .with_anti_alias(false)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();
    let render = |frame: RenderFrameExtract| {
        framework.submit_frame_extract(viewport, frame).unwrap();
        framework
            .capture_frame(viewport)
            .unwrap()
            .expect("actual color readback")
    };
    let drawn = render(live);
    assert!(framework.query_stats().unwrap().last_mesh_draw_count > 0);
    scene.set_active_self(cube, false).unwrap();
    let hidden = controller.build_prepared_render_extract(&mut scene);
    assert!(hidden.geometry.meshes.is_empty());
    let empty = render(hidden);
    assert_eq!(framework.query_stats().unwrap().last_mesh_draw_count, 0);
    assert_eq!((drawn.width, drawn.height), (SIZE, SIZE));
    assert_eq!((empty.width, empty.height), (SIZE, SIZE));
    let pixels = |x, y| {
        count_changed_pixels_in_frame(
            &drawn.rgba,
            &empty.rgba,
            SIZE,
            SIZE,
            UiFrame::new(x as f32, y as f32, PATCH as f32, PATCH as f32),
            12,
        )
    };
    let center = pixels(126, 126);
    let outside = pixels(218, 126);
    let fringe = pixels(FRINGE.0, FRINGE.1);
    println!("editor_affine_pixels center={center} fringe={fringe} outside={outside}");
    assert!(center >= 12, "center control must actually draw the cube");
    assert_eq!(outside, 0, "outside control must not draw the cube");
    assert!(
        fringe >= 12,
        "editor viewport dropped inherited affine silhouette: {fringe}/16 pixels"
    );
}
