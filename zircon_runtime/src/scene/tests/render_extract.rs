#[cfg(feature = "animation")]
use std::collections::BTreeMap;

#[cfg(feature = "animation")]
use crate::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, DisplayMode, RenderBloomSettings,
    RenderCameraOrderAmbiguity, RenderCameraTarget, RenderCameraTargetOrderKey,
    RenderColorGradingSettings, RenderExtractContext, RenderExtractProducer, RenderLayerSet,
    RenderMaterialAlphaMode, RenderPhase, RenderPhaseMeshSource, RenderPostProcessVolumeProfile,
    RenderTonemapOperator, RenderTonemapSettings, RenderVirtualGeometryDebugState,
    RenderWorldSnapshotHandle, SceneViewportExtractRequest, ViewportCameraSnapshot,
    ViewportRenderSettings,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
#[cfg(feature = "animation")]
use crate::core::resource::AnimationSkeletonMarker;
use crate::core::resource::{MeshMarker, ResourceHandle, ResourceId, TextureMarker};
#[cfg(feature = "animation")]
use crate::scene::components::AnimationSkeletonComponent;
use crate::scene::components::{
    AmbientLight, CameraComponent, MeshRenderer, MeshRendererLodLevel, Mobility, NodeKind,
    PostProcessVolumeComponent, Sprite2dComponent,
};
use crate::scene::{DefaultLevelManager, World};

use super::support::{material_handle, model_handle};

mod camera_order;
mod direct_sections;
mod level_source_guards;
mod lighting_postprocess;
mod particles;

fn camera_descriptor_with_layers(mask: u32) -> CameraRenderDescriptor {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.culling_mask = RenderLayerSet::from_scene_schema_v1_mask(mask);
    camera.volume_mask = camera.culling_mask.clone();
    camera
}

fn spawn_camera_on_layer(world: &mut World, layer_mask: u32) -> crate::scene::EntityId {
    let camera = world
        .spawn_node(NodeKind::Camera)
        .expect("test scene spawn should succeed");
    world
        .insert(
            camera,
            CameraComponent {
                projection_mode: crate::core::framework::render::ProjectionMode::Perspective,
                ..CameraComponent::default()
            },
        )
        .unwrap();
    world.set_render_layer_mask(camera, layer_mask).unwrap();
    camera
}

fn spawn_mesh_on_layer(
    world: &mut World,
    layer_mask: u32,
    mobility: Mobility,
) -> crate::scene::EntityId {
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .insert(
            entity,
            MeshRenderer::from_handles(
                model_handle(&format!("res://models/direct-{entity}.obj")),
                material_handle(&format!("res://materials/direct-{entity}.zmaterial")),
            ),
        )
        .unwrap();
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    if mobility != Mobility::Dynamic {
        world.set_mobility(entity, mobility).unwrap();
    }
    entity
}

fn mesh_handle(label: &str) -> ResourceHandle<MeshMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn spawn_sprite_on_layer(world: &mut World, layer_mask: u32) -> crate::scene::EntityId {
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let _ = world.remove::<MeshRenderer>(entity).unwrap();
    world
        .insert(
            entity,
            Sprite2dComponent {
                image: ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
                    &format!("res://textures/sprite-{entity}.png"),
                )),
                material: Some(material_handle(&format!(
                    "res://materials/sprite-{entity}.zmaterial"
                ))),
                custom_size: Some(Vec2::new(2.0, 3.0)),
                color: Vec4::new(1.0, 0.5, 0.25, 0.75),
                z_order: 5,
                material_alpha_mode: RenderMaterialAlphaMode::Blend,
                ..Sprite2dComponent::default()
            },
        )
        .unwrap();
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    entity
}

fn spawn_light_on_layer(
    world: &mut World,
    kind: NodeKind,
    layer_mask: u32,
) -> crate::scene::EntityId {
    let entity = world
        .spawn_node(kind)
        .expect("test scene spawn should succeed");
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    entity
}

fn spawn_post_process_volume_on_layer(
    world: &mut World,
    layer_mask: u32,
    volume: PostProcessVolumeComponent,
) -> crate::scene::EntityId {
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let _ = world.remove::<MeshRenderer>(entity).unwrap();
    world.insert(entity, volume).unwrap();
    world.set_render_layer_mask(entity, layer_mask).unwrap();
    entity
}

#[cfg(feature = "animation")]
fn test_pose(bone: &str) -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: Some("Locomotion".to_string()),
        bones: vec![AnimationPoseBone {
            name: bone.to_string(),
            local_transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        }],
    }
}

fn assert_runtime_submit_tree_excludes_snapshot_adapters(root: &std::path::Path) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) == Some("tests") {
                continue;
            }
            assert_runtime_submit_tree_excludes_snapshot_adapters(&path);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("tests.rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let production_source = source.split("#[cfg(test)]").next().unwrap_or(&source);
        assert_source_excludes(
            &path,
            production_source,
            &[
                "RenderFrameExtract::from_snapshot",
                "ViewportRenderFrame::from_snapshot",
            ],
            "runtime submit_frame_extract production code must consume RenderFrameExtract through ViewportRenderFrame::from_extract; snapshot adapters are limited to tests/preview/synthetic validation",
        );
    }
}

fn assert_source_excludes_file(path: &std::path::Path, forbidden: &[&str], message: &str) {
    let source = std::fs::read_to_string(path).unwrap();
    assert_source_excludes(path, &source, forbidden, message);
}

fn assert_source_excludes(path: &std::path::Path, source: &str, forbidden: &[&str], message: &str) {
    for token in forbidden {
        assert!(
            !source.contains(token),
            "{message}: found `{token}` in {path:?}"
        );
    }
}

#[test]
fn viewport_extract_profiling_keeps_clone_projection_visit_and_sort_attribution_separate() {
    let world_source = include_str!("../world/world.rs");
    let render_source = include_str!("../world/render.rs");
    let compact_world_source = world_source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let compact_render_source = render_source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    for scope in ["world_clone", "world_projection_rebuild"] {
        let invocation = format!("crate::profile_scope!(\"runtime\",\"scene\",\"{scope}\");");
        assert!(
            compact_world_source.contains(&invocation),
            "World cloning must expose the `{scope}` profiling boundary"
        );
    }
    for scope in [
        "viewport_render_packet",
        "render_frame_extract",
        "render_mesh_visit",
        "render_mesh_sort",
    ] {
        let invocation = format!("crate::profile_scope!(\"runtime\",\"scene\",\"{scope}\");");
        assert!(
            compact_render_source.contains(&invocation),
            "Viewport extraction must expose the `{scope}` profiling boundary"
        );
    }
    for counter in [
        "viewport_packet_mesh_count",
        "viewport_packet_mesh_payload_bytes",
        "render_frame_mesh_count",
        "render_frame_mesh_payload_bytes",
    ] {
        let invocation_prefix = format!("crate::profile_counter!(\"runtime\",\"{counter}\",");
        assert!(
            compact_render_source.contains(&invocation_prefix),
            "Viewport extraction must expose the `{counter}` profiling counter"
        );
    }
}
