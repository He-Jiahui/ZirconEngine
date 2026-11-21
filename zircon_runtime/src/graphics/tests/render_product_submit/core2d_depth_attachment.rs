//! Core2D submits its depth attachment through the exact compiled resource contract.

use std::sync::Arc;

use crate::asset::{AssetLoadState, AssetUri, ProjectAssetManager, TextureAsset};
use crate::core::framework::render::{
    CorePipelineKind, PostProcessGraphResourceNames, ProjectionMode, RenderCameraClearColor,
    RenderCaptureSource, RenderExtractContext, RenderExtractProducer, RenderFrameExtract,
    RenderFramework, RenderMaterialAlphaMode, RenderPhase, RenderPipelineHandle,
    RenderViewportDescriptor, SceneViewportExtractRequest,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, TextureMarker,
};
use crate::graphics::{RenderPipelineAsset, WgpuRenderFramework};
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};
use crate::scene::components::{CameraComponent, NodeKind, Sprite2dComponent};
use crate::scene::{DefaultLevelManager, World};

const SIZE: UVec2 = UVec2::new(128, 128);
const PHASES: [RenderPhase; 3] = [
    RenderPhase::Opaque2d,
    RenderPhase::AlphaMask2d,
    RenderPhase::Transparent2d,
];
const PASSES: [&str; 3] = ["opaque-sprite", "alpha-mask-sprite", "transparent-sprite"];
const EXECUTORS: [&str; 3] = ["sprite.opaque", "sprite.alpha-mask", "sprite.transparent"];
const URIS: [&str; 3] = [
    "mem://tests/core2d-depth-red",
    "mem://tests/core2d-depth-green",
    "mem://tests/core2d-depth-blue",
];
const COLORS: [[u8; 4]; 3] = [[240, 16, 8, 255], [8, 240, 16, 255], [16, 8, 240, 255]];

fn rigid_sprite_frame(images: [ResourceHandle<TextureMarker>; 3]) -> RenderFrameExtract {
    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera).unwrap();
    world
        .insert(
            camera,
            CameraComponent {
                core_pipeline: CorePipelineKind::Core2d,
                projection_mode: ProjectionMode::Orthographic,
                ortho_size: 4.0,
                msaa_samples: 1,
                clear_color: RenderCameraClearColor::Color(Vec4::new(0.0, 0.0, 0.0, 1.0)),
                ..CameraComponent::default()
            },
        )
        .unwrap();
    world
        .update_transform(camera, Transform::from_translation(5.0 * Vec3::Z))
        .unwrap();
    for (index, (image, alpha_mode)) in images
        .into_iter()
        .zip([
            RenderMaterialAlphaMode::Opaque,
            RenderMaterialAlphaMode::Mask { cutoff: 0.5 },
            RenderMaterialAlphaMode::Blend,
        ])
        .enumerate()
    {
        let sprite = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .update_transform(
                sprite,
                Transform::from_translation(Vec3::new(2.0 * index as f32 - 2.0, 0.0, 0.0)),
            )
            .unwrap();
        world
            .insert(
                sprite,
                Sprite2dComponent {
                    image,
                    custom_size: Some(Vec2::splat(1.25)),
                    material_alpha_mode: alpha_mode,
                    ..Sprite2dComponent::default()
                },
            )
            .unwrap();
    }
    let identity = world.render_world_snapshot_handle();
    let level = DefaultLevelManager::default().create_level(world, Default::default());
    let extract = level.build_render_frame_extract(&RenderExtractContext::new(
        identity,
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
    assert_eq!(extract.sprites.sprites.len(), 3);
    assert_eq!(
        extract
            .geometry
            .scene_changes
            .as_ref()
            .unwrap()
            .world()
            .raw(),
        extract.world.raw()
    );
    for phase in PHASES {
        assert_eq!(
            extract.sprites.phase_queue.items_for_phase(phase).count(),
            1
        );
    }
    extract
}

#[test]
fn core2d_sprite_depth_graph_preserves_exact_attachment_successors() {
    use RenderGraphResourceAccessKind::{Read, Write};

    let extract =
        rigid_sprite_frame(URIS.map(|uri| ResourceHandle::new(ResourceId::from_stable_label(uri))));
    let compiled = RenderPipelineAsset::default_core2d()
        .compile(&extract)
        .unwrap();
    let graph = compiled.graph();
    let depth_name = PostProcessGraphResourceNames::SCENE_DEPTH;
    let depth = graph
        .resource_declaration_by_name(depth_name)
        .unwrap()
        .resource;
    let mut previous = None;
    for name in PASSES {
        let pass = graph
            .passes()
            .iter()
            .find(|pass| pass.name == name)
            .unwrap();
        assert!(!pass.culled);
        let write = graph
            .access_id_for(pass.id, depth, Write)
            .expect("depth attachment owns a write access");
        let output = graph.versioned_access_key(write).unwrap();
        let attachment = pass
            .resources
            .iter()
            .find(|r| r.name == depth_name && r.access == Write)
            .unwrap();
        assert_eq!(graph.input_version_for_id(write), previous);
        if let Some(input) = previous {
            let read = graph
                .access_id_for(pass.id, depth, Read)
                .expect("later phase also declares a prior depth read");
            assert_eq!(graph.versioned_access_key(read).unwrap().version, input);
            assert_ne!(output.version, input);
            assert_eq!(
                attachment.attachment_ops,
                Some(RenderGraphAttachmentOps::load_store())
            );
        } else {
            assert!(graph.access_id_for(pass.id, depth, Read).is_none());
            assert_eq!(
                attachment.attachment_ops,
                Some(RenderGraphAttachmentOps::clear_store())
            );
        }
        previous = Some(output.version);
    }
}

#[test]
fn core2d_sprite_depth_attachment_renders_three_rigid_phases() {
    let assets = Arc::new(ProjectAssetManager::default());
    let store = assets.assets::<TextureAsset>();
    let handles = std::array::from_fn(|i| {
        let uri = AssetUri::parse(URIS[i]).unwrap();
        store
            .insert(
                ResourceRecord::new(
                    ResourceId::from_locator(&uri),
                    ResourceKind::Texture,
                    uri.clone(),
                ),
                TextureAsset::new_rgba8(uri, 2, 2, COLORS[i].repeat(4)),
            )
            .unwrap()
    });
    let _leases = handles.map(|handle| {
        let lease = store.acquire(handle).expect("real resident sprite texture");
        assert_eq!(store.load_state(handle), AssetLoadState::Loaded);
        lease
    });
    let extract = rigid_sprite_frame(handles.map(|handle| handle.resource_handle()));
    let framework = WgpuRenderFramework::new_for_test(assets).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(SIZE))
        .unwrap();
    let backend = framework
        .query_graphics_debugger_status()
        .unwrap()
        .backend_name;
    let device = framework
        .query_stats()
        .unwrap()
        .device_diagnostics
        .expect("render device identity");
    eprintln!(
        "CORE2D_DEPTH_READY backend={backend} adapter={:?} device_type={} sprite_phases=3 world={}",
        device.adapter_name,
        device.adapter_device_type,
        extract.world.raw()
    );
    framework
        .submit_frame_extract(viewport, extract)
        .unwrap_or_else(|error| {
            panic!("rigid Core2D sprite attachment submission must succeed: {error}")
        });
    let captured = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("submitted rigid sprite capture");
    let stats = framework.query_stats().unwrap();
    framework.destroy_viewport(viewport).unwrap();
    assert!(framework.first_device_fault().is_none());
    assert_eq!(
        captured.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );
    assert_eq!((captured.width, captured.height), (SIZE.x, SIZE.y));
    assert_eq!(stats.last_generation, Some(captured.generation));
    assert_eq!(stats.last_pipeline, Some(RenderPipelineHandle::new(3)));
    assert_eq!(stats.last_sprite_count, 3);
    assert_eq!(stats.last_sprite_ready_count, 3);
    assert_eq!(stats.last_sprite_texture_fallback_count, 0);
    assert_eq!(stats.last_sprite_vertex_count, 18);
    assert_eq!(
        [
            stats.last_sprite_opaque_draw_batch_count,
            stats.last_sprite_alpha_mask_draw_batch_count,
            stats.last_sprite_transparent_draw_batch_count,
        ],
        [1, 1, 1]
    );
    for executor in EXECUTORS {
        assert!(stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|id| id == executor));
    }
    let red = coverage(&captured.rgba, 30, Some(0));
    let green = coverage(&captured.rgba, 62, Some(1));
    let blue = coverage(&captured.rgba, 94, Some(2));
    let outside = coverage(&captured.rgba, 116, None);
    eprintln!("CORE2D_DEPTH_PIXELS ready=3 fallback=0 vertices=18 opaque={red} mask={green} blend={blue} outside={outside}");
    assert_eq!([red, green, blue, outside], [16, 16, 16, 16]);
}

fn coverage(rgba: &[u8], left: u32, primary: Option<usize>) -> usize {
    assert_eq!(rgba.len(), (SIZE.x * SIZE.y * 4) as usize);
    (62..66)
        .flat_map(|y| (left..left + 4).map(move |x| ((y * SIZE.x + x) * 4) as usize))
        .filter(|&index| {
            let rgb = &rgba[index..index + 3];
            match primary {
                Some(channel) => {
                    rgb[channel] > 110
                        && rgb.iter().enumerate().all(|(i, &v)| i == channel || v < 48)
                }
                None => rgb.iter().all(|&v| v < 12),
            }
        })
        .count()
}
