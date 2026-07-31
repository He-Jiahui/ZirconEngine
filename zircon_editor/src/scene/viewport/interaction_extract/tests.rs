use std::{cell::Cell, sync::Arc};

use zircon_runtime::scene::{Scene, components::NodeKind};
use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::{
    SceneViewportSettings, ViewportCameraSnapshot, render_packet::build_render_packet,
};

use super::ViewportInteractionExtractCache;

#[test]
fn stable_generation_reuses_one_interaction_extract_and_handle_build() {
    let scene = Scene::new();
    let cache = ViewportInteractionExtractCache::default();
    let settings = SceneViewportSettings::default();
    let camera = ViewportCameraSnapshot::default();
    let viewport = UVec2::new(1280, 720);
    let handle_builds = Cell::new(0);

    let first = cache.resolve_for_pointer(
        &scene,
        None,
        &settings,
        &camera,
        viewport,
        || {
            handle_builds.set(handle_builds.get() + 1);
            Vec::new()
        },
        Vec::new,
    );
    let second = cache.resolve_for_pointer(
        &scene,
        None,
        &settings,
        &camera,
        viewport,
        || {
            handle_builds.set(handle_builds.get() + 1);
            Vec::new()
        },
        Vec::new,
    );

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(handle_builds.get(), 1);
}

#[test]
fn world_generation_change_rebuilds_the_shared_extract() {
    let mut scene = Scene::new();
    let cache = ViewportInteractionExtractCache::default();
    let settings = SceneViewportSettings::default();
    let camera = ViewportCameraSnapshot::default();
    let viewport = UVec2::new(1280, 720);

    let first = cache.resolve_for_pointer(
        &scene,
        None,
        &settings,
        &camera,
        viewport,
        Vec::new,
        Vec::new,
    );
    scene.spawn_node(NodeKind::Empty);
    let second = cache.resolve_for_pointer(
        &scene,
        None,
        &settings,
        &camera,
        viewport,
        Vec::new,
        Vec::new,
    );

    assert!(!Arc::ptr_eq(&first, &second));
}

#[test]
fn render_path_seeds_the_same_runtime_mesh_extract_used_by_pointer() {
    let scene = Scene::new();
    let cache = ViewportInteractionExtractCache::default();
    let settings = SceneViewportSettings::default();
    let camera = ViewportCameraSnapshot::default();
    let viewport = UVec2::new(1280, 720);
    let packet = build_render_packet(&scene, &settings, &camera, None, viewport);

    let from_render = cache.resolve_from_render_packet(
        &scene,
        None,
        &settings,
        &camera,
        viewport,
        &packet.scene.meshes,
        Vec::new,
        Vec::new,
    );
    let from_pointer = cache.resolve_for_pointer(
        &scene,
        None,
        &settings,
        &camera,
        viewport,
        Vec::new,
        Vec::new,
    );

    assert!(Arc::ptr_eq(&from_render, &from_pointer));
    assert_eq!(
        from_render
            .render_meshes()
            .iter()
            .map(|mesh| mesh.node_id)
            .collect::<Vec<_>>(),
        packet
            .scene
            .meshes
            .iter()
            .map(|mesh| mesh.node_id)
            .collect::<Vec<_>>()
    );
}
