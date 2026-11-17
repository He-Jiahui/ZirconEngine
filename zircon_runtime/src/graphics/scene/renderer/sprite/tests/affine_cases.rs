//! Test the ECS sprite's world coordinates at the vertex and product-pixel boundaries.
use std::sync::Arc;

use crate::asset::{AssetLoadState, AssetUri, ProjectAssetManager, TextureAsset};
use crate::core::framework::render::{
    CorePipelineKind, ProjectionMode, RenderCameraClearColor, RenderCaptureSource,
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderFramework,
    RenderPipelineHandle, RenderSpriteAtlasRegion, RenderViewportDescriptor,
    SceneViewportExtractRequest, SpriteExtract,
};
use crate::core::math::{Quat, Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, TextureMarker,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::WgpuRenderFramework;
use crate::scene::components::{CameraComponent, NodeKind, Sprite2dComponent};
use crate::scene::{DefaultLevelManager, LevelMetadata, World};

use super::{build_sprite_vertices, prepare_sprite_draw_batches, SpriteVertex};

const SIZE: UVec2 = UVec2::new(128, 128);
const HALF_HEIGHT: f32 = 4.0;
const TEXTURE_URI: &str = "mem://tests/sprite-affine-solid";
const ROOT_2: f32 = std::f32::consts::SQRT_2;

fn sprite_frame(scale_x: f32, image: ResourceHandle<TextureMarker>) -> RenderFrameExtract {
    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera).unwrap();
    world
        .insert(
            camera,
            CameraComponent {
                core_pipeline: CorePipelineKind::Core2d,
                projection_mode: ProjectionMode::Orthographic,
                ortho_size: HALF_HEIGHT,
                clear_color: RenderCameraClearColor::Color(Vec4::new(0.0, 0.0, 0.0, 1.0)),
                ..CameraComponent::default()
            },
        )
        .unwrap();
    world
        .update_transform(camera, Transform::from_translation(5.0 * Vec3::Z))
        .unwrap();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .update_transform(
            parent,
            Transform {
                scale: Vec3::new(scale_x, 1.0, 1.0),
                ..Transform::default()
            },
        )
        .unwrap();
    let child = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .update_transform(
            child,
            Transform {
                rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                ..Transform::default()
            },
        )
        .unwrap();
    assert!(world.set_parent_checked(child, Some(parent)).unwrap());
    world
        .insert(
            child,
            Sprite2dComponent {
                image,
                custom_size: Some(Vec2::splat(2.0)),
                atlas_region: Some(RenderSpriteAtlasRegion::new(
                    Vec2::splat(0.25),
                    Vec2::splat(0.75),
                )),
                flip_x: true,
                ..Sprite2dComponent::default()
            },
        )
        .unwrap();
    let scene_matrix = world.world_matrix(child).unwrap();
    assert!(
        (scene_matrix.x_axis.dot(scene_matrix.y_axis) - (1.0 - scale_x * scale_x) * 0.5).abs()
            < 1.0e-5
    );
    let source_world = world.render_world_snapshot_handle();
    let level = DefaultLevelManager::default().create_level(world, LevelMetadata::default());
    let extract = level.build_render_frame_extract(&RenderExtractContext::new(
        source_world,
        SceneViewportExtractRequest {
            active_camera_override: Some(camera),
            viewport_size: Some(SIZE),
            ..Default::default()
        },
    ));
    assert_eq!(extract.view.core_pipeline, CorePipelineKind::Core2d);
    assert_eq!(
        extract.view.camera.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(extract.sprites.sprites.len(), 1);
    assert_eq!(extract.sprites.sprites[0].entity, child);
    assert_eq!(extract.sprites.sprites[0].image, image);
    extract
}

fn test_sprite_frame(scale_x: f32) -> RenderFrameExtract {
    sprite_frame(
        scale_x,
        ResourceHandle::new(ResourceId::from_stable_label(TEXTURE_URI)),
    )
}

fn vertex_rows(extract: RenderFrameExtract) -> [[f32; 9]; 6] {
    let frame = ViewportRenderFrame::from_extract(extract, SIZE);
    let stage = RenderPassStage::Transparent2d;
    let vertices = build_sprite_vertices(&frame, stage);
    let batches = prepare_sprite_draw_batches(&frame, stage);
    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0].0, 0);
    assert_eq!(batches.len(), 1);
    let per_sprite: &[[f32; 9]] = bytemuck::cast_slice::<SpriteVertex, _>(&vertices[0].1);
    let batched: &[[f32; 9]] = bytemuck::cast_slice::<SpriteVertex, _>(batches[0].vertices());
    assert_eq!(
        per_sprite, batched,
        "batched and per-entity sprite positions must agree"
    );
    per_sprite.try_into().unwrap()
}

fn assert_vertices(label: &str, rows: [[f32; 9]; 6], expected: [Vec3; 6]) {
    let expected_uv = [
        [0.75, 0.75],
        [0.75, 0.25],
        [0.25, 0.75],
        [0.25, 0.75],
        [0.75, 0.25],
        [0.25, 0.25],
    ];
    for (row, uv) in rows.iter().zip(expected_uv) {
        assert_eq!(
            row[3..5],
            uv,
            "anchor and atlas X flip must keep the original triangle mapping"
        );
        assert_eq!(row[5..9], [1.0; 4]);
    }
    let actual = rows.map(|row| Vec3::new(row[0], row[1], row[2]));
    assert!(actual.iter().all(|p| p.is_finite()));
    let max_error = actual
        .iter()
        .zip(expected)
        .map(|(a, e)| a.distance(e))
        .fold(0.0_f32, f32::max);
    eprintln!(
        "SPRITE_AFFINE_CPU label={label} max_error={max_error} tl={:?} br={:?}",
        actual[0], actual[5]
    );
    assert!(
        max_error < 1.0e-5,
        "sprite world corners {label}: {actual:?}; expected {expected:?}"
    );
}

fn diamond(scale_x: f32) -> [Vec3; 6] {
    // Independent scalar oracle for diag(scale_x,1,1) * Rz(45°) applied to
    // the four corners of a centered [−1, +1] × [−1, +1] sprite.
    let tl = Vec3::new(-scale_x * ROOT_2, 0.0, 0.0);
    let bl = Vec3::new(0.0, -ROOT_2, 0.0);
    let tr = Vec3::new(0.0, ROOT_2, 0.0);
    let br = Vec3::new(scale_x * ROOT_2, 0.0, 0.0);
    [tl, bl, tr, tr, bl, br]
}

#[test]
fn sprite_world_affine_rigid_vertices_keep_atlas_and_order() {
    assert_vertices("rigid", vertex_rows(test_sprite_frame(1.0)), diamond(1.0));
}

#[test]
fn sprite_world_affine_sheared_vertices_survive_shared_scene_submission() {
    let extract = test_sprite_frame(2.0);
    let descriptor = extract.view.selected_camera_descriptor().unwrap().clone();
    let camera_view = extract.for_camera_submission(descriptor);
    assert!(camera_view.shares_scene_with(&extract));
    let shared = RenderFrameExtract::from_shared_scene(
        camera_view.shared_scene(),
        camera_view.view.clone(),
        camera_view.timing,
    );
    let live = vertex_rows(extract);
    let reused = vertex_rows(shared);
    assert_eq!(
        live, reused,
        "sharing the scene may not discard its world mapping"
    );
    assert_vertices("shear", reused, diamond(2.0));
}

#[test]
fn sprite_world_affine_raw_replacement_does_not_reuse_a_prepared_matrix() {
    let extract = test_sprite_frame(2.0);
    let replacement = extract.sprites.sprites[0].clone();
    let matrix = replacement.transform.matrix();
    let tl = matrix.transform_point3(Vec3::new(-1.0, 1.0, 0.0));
    let bl = matrix.transform_point3(Vec3::new(-1.0, -1.0, 0.0));
    let tr = matrix.transform_point3(Vec3::new(1.0, 1.0, 0.0));
    let br = matrix.transform_point3(Vec3::new(1.0, -1.0, 0.0));
    let mut raw = extract.clone();
    // Same EntityId and compatibility TRS are insufficient provenance for a
    // raw domain that was independently supplied by a caller.
    *raw.sprites = SpriteExtract::from_sprites(CorePipelineKind::Core2d, vec![replacement]);
    assert!(!raw.sprites.ptr_eq(&extract.sprites));
    assert_vertices("raw", vertex_rows(raw), [tl, bl, tr, tr, bl, br]);
}

fn coverage(rgba: &[u8], x: u32, y: u32) -> (usize, usize) {
    let mut red = 0;
    let mut black = 0;
    for py in y..y + 4 {
        for px in x..x + 4 {
            let i = ((py * SIZE.x + px) * 4) as usize;
            let rgb = &rgba[i..i + 3];
            red += usize::from(rgb[0] > 110 && rgb[0] as u16 >= rgb[1] as u16 + 65 && rgb[2] < 48);
            black += usize::from(rgb.iter().all(|c| *c < 12));
        }
    }
    (red, black)
}

#[test]
fn sprite_world_affine_gpu_keeps_center_and_rasterizes_the_shear_fringe() {
    let assets = Arc::new(ProjectAssetManager::default());
    let uri = AssetUri::parse(TEXTURE_URI).unwrap();
    let store = assets.assets::<TextureAsset>();
    let texture = store
        .insert(
            ResourceRecord::new(
                ResourceId::from_locator(&uri),
                ResourceKind::Texture,
                uri.clone(),
            ),
            TextureAsset::new_rgba8(uri, 2, 2, [240, 24, 8, 255].repeat(4)),
        )
        .unwrap();
    let _residency = store
        .acquire(texture)
        .expect("keep the real texture resident through capture");
    assert_eq!(store.load_state(texture), AssetLoadState::Loaded);
    let framework = WgpuRenderFramework::new_for_test(assets).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(SIZE))
        .unwrap();
    // Let the first real World extract select its Core2D pipeline, rather
    // than applying a pipeline override before any camera has been submitted.
    let backend = framework
        .query_graphics_debugger_status()
        .unwrap()
        .backend_name;
    let mut sheared_fringe = (0, 0);
    let mut last_capture_generation = None;
    for scale_x in [1.0, 2.0] {
        framework
            .submit_frame_extract(viewport, sprite_frame(scale_x, texture.resource_handle()))
            .unwrap();
        let captured = framework
            .capture_frame_if_newer(viewport, last_capture_generation)
            .unwrap()
            .expect("fresh offscreen GPU readback after the current submission");
        last_capture_generation = Some(captured.generation);
        assert!(framework.first_device_fault().is_none());
        assert_eq!(
            captured.capture_report.source,
            RenderCaptureSource::FrameworkOffscreen
        );
        assert_eq!((captured.width, captured.height), (SIZE.x, SIZE.y));
        assert_eq!(captured.rgba.len(), (SIZE.x * SIZE.y * 4) as usize);
        let stats = framework.query_stats().unwrap();
        assert_eq!(stats.last_pipeline, Some(RenderPipelineHandle::new(3)));
        let adapter = stats
            .device_diagnostics
            .as_ref()
            .expect("runtime GPU identity");
        assert!(stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|id| id == "sprite.transparent"));
        assert_eq!(stats.last_sprite_count, 1);
        assert_eq!(stats.last_sprite_ready_count, 1);
        assert_eq!(stats.last_sprite_texture_fallback_count, 0);
        assert_eq!(stats.last_sprite_image_slice_count, 1);
        let center = coverage(&captured.rgba, 62, 62);
        let fringe = coverage(&captured.rgba, 102, 62);
        let outside = coverage(&captured.rgba, 116, 62);
        // Fringe spans world X=2.40625..2.59375, |Y|<=0.09375; strictly
        // inside the sheared diamond, but outside the rigid 2x2 sprite.
        eprintln!("SPRITE_AFFINE_GPU scale_x={scale_x} backend={backend} adapter={:?} device_type={} ready=1 fallback=0 pass=sprite.transparent center={center:?} fringe={fringe:?} outside={outside:?}", adapter.adapter_name, adapter.adapter_device_type);
        assert_eq!(
            center,
            (16, 0),
            "ready texture must cover the center in both frames"
        );
        assert_eq!(outside, (0, 16), "far outside must remain background");
        if scale_x == 1.0 {
            assert_eq!(
                fringe,
                (0, 16),
                "rigid control should not cover the shear-only fringe"
            );
        } else {
            sheared_fringe = fringe;
        }
    }
    framework.destroy_viewport(viewport).unwrap();
    assert_eq!(
        sheared_fringe,
        (16, 0),
        "sprite shear fringe must be rasterized by the real affine footprint"
    );
}
