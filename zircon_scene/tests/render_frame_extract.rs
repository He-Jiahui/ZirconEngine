use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_scene::{Mobility, RenderFrameExtract, RenderWorldSnapshotHandle, World};

#[test]
fn render_frame_extract_adapter_preserves_legacy_snapshot_content() {
    let world = World::new();
    let snapshot = world.to_render_snapshot();
    let extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(9), snapshot.clone());

    assert_eq!(extract.world, RenderWorldSnapshotHandle::new(9));
    assert_eq!(extract.geometry.meshes.len(), snapshot.scene.meshes.len());
    assert_eq!(
        extract.lighting.directional_lights.len(),
        snapshot.scene.lights.len()
    );
    assert_eq!(
        extract.debug.overlays.display_mode,
        snapshot.overlays.display_mode
    );
    assert_eq!(extract.to_legacy_snapshot(), snapshot);
}

#[test]
fn world_render_frame_extract_preserves_visibility_metadata() {
    let mut world = World::new();
    let default_cube = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .expect("default world should contain a renderable cube")
        .id;
    assert!(world.remove_entity(default_cube));

    let static_mesh = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.material.toml"),
    );
    let dynamic_mesh = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.material.toml"),
    );
    world
        .set_mobility(static_mesh, Mobility::Static)
        .expect("static mobility assignment should succeed");
    world
        .set_render_layer_mask(static_mesh, 0x0000_0004)
        .expect("render layer assignment should succeed");
    world
        .set_render_layer_mask(dynamic_mesh, 0x0000_0002)
        .expect("render layer assignment should succeed");

    let extract = world.to_render_frame_extract();

    assert_eq!(
        extract.visibility.renderable_entities,
        vec![static_mesh, dynamic_mesh]
    );
    assert_eq!(extract.visibility.static_entities, vec![static_mesh]);
    assert_eq!(extract.visibility.dynamic_entities, vec![dynamic_mesh]);
    assert_eq!(
        extract
            .visibility
            .renderables
            .iter()
            .map(|entry| (entry.entity, entry.mobility, entry.render_layer_mask))
            .collect::<Vec<_>>(),
        vec![
            (static_mesh, Mobility::Static, 0x0000_0004),
            (dynamic_mesh, Mobility::Dynamic, 0x0000_0002),
        ]
    );
    assert_eq!(
        extract
            .geometry
            .meshes
            .iter()
            .map(|mesh| (mesh.node_id, mesh.mobility, mesh.render_layer_mask))
            .collect::<Vec<_>>(),
        vec![
            (static_mesh, Mobility::Static, 0x0000_0004),
            (dynamic_mesh, Mobility::Dynamic, 0x0000_0002),
        ]
    );
}

#[test]
fn render_frame_extract_preserves_camera_aspect_ratio() {
    let world = World::new();
    let mut snapshot = world.to_render_snapshot();
    snapshot.scene.camera.aspect_ratio = 2.0;

    let extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(17), snapshot.clone());

    assert!((extract.view.camera.aspect_ratio - 2.0).abs() < 0.0001);
    assert!((extract.to_legacy_snapshot().scene.camera.aspect_ratio - 2.0).abs() < 0.0001);
}

fn model_handle(label: &str) -> ResourceHandle<ModelMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn material_handle(label: &str) -> ResourceHandle<MaterialMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}
