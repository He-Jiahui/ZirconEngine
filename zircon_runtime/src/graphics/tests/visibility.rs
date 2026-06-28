use crate::core::framework::render::{
    RenderLayerSet, RenderVirtualGeometryCluster, RenderVirtualGeometryPage,
};
use crate::core::math::{Transform, Vec3};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::scene::components::Mobility;
use crate::scene::world::World;

use crate::graphics::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBvhUpdateStrategy, VisibilityContext,
    VisibilityDrawCommand,
};

#[test]
fn visibility_context_partitions_static_and_dynamic_meshes() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let static_mesh = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.zmaterial"),
    );
    let dynamic_mesh = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.zmaterial"),
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
    world
        .set_render_layer_mask(world.active_camera(), 0x0000_0006)
        .expect("camera layer assignment should include test meshes");

    let context = VisibilityContext::from(&world.to_render_frame_extract());

    assert_eq!(context.renderable_entities, vec![static_mesh, dynamic_mesh]);
    assert_eq!(context.static_entities, vec![static_mesh]);
    assert_eq!(context.dynamic_entities, vec![dynamic_mesh]);
}

#[test]
fn visibility_context_builds_deterministic_batches_and_instancing_candidates() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let crate_a = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.zmaterial"),
    );
    let statue = world.spawn_mesh_node(
        model_handle("res://models/statue.obj"),
        material_handle("res://materials/statue.zmaterial"),
    );
    let crate_b = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.zmaterial"),
    );
    let tree = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.zmaterial"),
    );
    world
        .set_render_layer_mask(crate_a, 0x0000_0001)
        .expect("render layer assignment should succeed");
    world
        .set_render_layer_mask(crate_b, 0x0000_0001)
        .expect("render layer assignment should succeed");
    world
        .set_render_layer_mask(statue, 0x0000_0008)
        .expect("render layer assignment should succeed");
    world
        .set_render_layer_mask(tree, 0x0000_0008)
        .expect("render layer assignment should succeed");
    world
        .set_mobility(tree, Mobility::Static)
        .expect("static mobility assignment should succeed");
    world
        .set_render_layer_mask(world.active_camera(), 0x0000_0009)
        .expect("camera layer assignment should include all test batches");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.meshes.reverse();
    extract.visibility.renderable_entities.reverse();
    extract.visibility.static_entities.reverse();
    extract.visibility.dynamic_entities.reverse();
    extract.visibility.renderables.reverse();

    let context = VisibilityContext::from(&extract);
    let mut expected_batches = vec![
        crate_batch(vec![crate_a, crate_b]),
        VisibilityBatch {
            key: VisibilityBatchKey {
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(0x0000_0008),
                material_id: ResourceId::from_stable_label("res://materials/statue.zmaterial"),
                model_id: ResourceId::from_stable_label("res://models/statue.obj"),
                mobility: Mobility::Dynamic,
            },
            entities: vec![statue],
        },
        VisibilityBatch {
            key: VisibilityBatchKey {
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(0x0000_0008),
                material_id: ResourceId::from_stable_label("res://materials/tree.zmaterial"),
                model_id: ResourceId::from_stable_label("res://models/tree.obj"),
                mobility: Mobility::Static,
            },
            entities: vec![tree],
        },
    ];
    expected_batches.sort_by(|left, right| left.key.cmp(&right.key));

    assert_eq!(context.batches, expected_batches);
    let expected_visible_instances = expected_batches
        .iter()
        .flat_map(|batch| batch.entities.iter().copied())
        .collect::<Vec<_>>();
    let expected_draw_commands = draw_commands_for_batches(&expected_batches);

    assert_eq!(context.visible_instances, expected_visible_instances);
    assert_eq!(context.draw_commands, expected_draw_commands);
    assert_eq!(
        context.gpu_instancing_candidates,
        vec![crate_batch(vec![crate_a, crate_b])]
    );
}

#[test]
fn visibility_context_filters_visible_batches_through_camera_frustum() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let visible = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.zmaterial"),
    );
    let culled = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.zmaterial"),
    );
    world
        .update_transform(visible, Transform::from_translation(Vec3::ZERO))
        .expect("visible mesh transform should update");
    world
        .update_transform(
            culled,
            Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        )
        .expect("culled mesh transform should update");

    let context = VisibilityContext::from(&world.to_render_frame_extract());

    assert_eq!(context.main_view_visible_entities(), vec![visible]);
    assert_eq!(context.main_view_culled_entities(), vec![culled]);
    assert_eq!(
        context.main_view_visible_batches(),
        vec![crate_batch(vec![visible])]
    );
    assert_eq!(context.visible_instances, vec![visible]);
    assert_eq!(
        context.draw_commands,
        vec![draw_command(crate_batch_key(), 0, 1)]
    );
    assert!(
        context.gpu_instancing_candidates.is_empty(),
        "a half-culled batch should not remain instancing-eligible for the visible pass"
    );
}

#[test]
fn visibility_context_without_history_marks_bvh_full_rebuild() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let crate_entity = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.zmaterial"),
    );
    let tree_entity = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.zmaterial"),
    );
    world
        .set_mobility(tree_entity, Mobility::Static)
        .expect("static mobility assignment should succeed");

    let context = VisibilityContext::from(&world.to_render_frame_extract());

    assert_eq!(
        context.bvh_update_plan.strategy,
        VisibilityBvhUpdateStrategy::FullRebuild
    );
    assert_eq!(
        context.bvh_update_plan.inserted_entities,
        vec![crate_entity, tree_entity]
    );
    assert!(context.bvh_update_plan.updated_entities.is_empty());
    assert!(context.bvh_update_plan.removed_entities.is_empty());
    assert_eq!(context.bvh_instances.len(), 2);
    assert_eq!(context.history_snapshot.instances.len(), 2);
    assert_eq!(
        context.instance_upload_plan.static_instance_entities,
        vec![tree_entity]
    );
    assert_eq!(
        context.instance_upload_plan.dynamic_instance_entities,
        vec![crate_entity]
    );
    assert_eq!(
        context.instance_upload_plan.dirty_dynamic_entities,
        vec![crate_entity]
    );
}

#[test]
fn visibility_context_with_history_tracks_bvh_dirty_entities() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let moving = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.zmaterial"),
    );
    let removed = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.zmaterial"),
    );
    let previous_context = VisibilityContext::from(&world.to_render_frame_extract());

    world
        .update_transform(
            moving,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .expect("moving mesh transform should update");
    assert!(world.remove_entity(removed));
    let inserted = world.spawn_mesh_node(
        model_handle("res://models/statue.obj"),
        material_handle("res://materials/statue.zmaterial"),
    );

    let context = VisibilityContext::from_extract_with_history(
        &world.to_render_frame_extract(),
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        context.bvh_update_plan.strategy,
        VisibilityBvhUpdateStrategy::Incremental
    );
    assert_eq!(context.bvh_update_plan.inserted_entities, vec![inserted]);
    assert_eq!(context.bvh_update_plan.updated_entities, vec![moving]);
    assert_eq!(context.bvh_update_plan.removed_entities, vec![removed]);
    assert_eq!(
        context.instance_upload_plan.static_instance_entities,
        Vec::<u64>::new()
    );
    assert_eq!(
        context.instance_upload_plan.dynamic_instance_entities,
        vec![moving, inserted]
    );
    assert_eq!(
        context.instance_upload_plan.dirty_dynamic_entities,
        vec![moving, inserted]
    );
}

#[test]
fn visibility_context_without_history_marks_particle_emitters_dirty() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let emitter_a = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.zmaterial"),
    );
    let emitter_b = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.zmaterial"),
    );
    let mut extract = world.to_render_frame_extract();
    extract.particles.emitters = vec![emitter_a, emitter_b];

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.particle_upload_plan.emitter_entities,
        vec![emitter_a, emitter_b]
    );
    assert_eq!(
        context.particle_upload_plan.dirty_emitters,
        vec![emitter_a, emitter_b]
    );
    assert!(context.particle_upload_plan.removed_emitters.is_empty());
}

#[test]
fn visibility_context_with_history_tracks_particle_upload_changes() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let emitter_a = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.zmaterial"),
    );
    let removed_emitter = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.zmaterial"),
    );
    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.particles.emitters = vec![emitter_a, removed_emitter];
    let previous_context = VisibilityContext::from(&previous_extract);

    let inserted_emitter = world.spawn_mesh_node(
        model_handle("res://models/statue.obj"),
        material_handle("res://materials/statue.zmaterial"),
    );
    let mut current_extract = world.to_render_frame_extract();
    current_extract.particles.emitters = vec![emitter_a, inserted_emitter];

    let context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        context.particle_upload_plan.emitter_entities,
        vec![emitter_a, inserted_emitter]
    );
    assert_eq!(
        context.particle_upload_plan.dirty_emitters,
        vec![inserted_emitter]
    );
    assert_eq!(
        context.particle_upload_plan.removed_emitters,
        vec![removed_emitter]
    );
}

mod virtual_geometry_frontier;
mod virtual_geometry_page_plan;
mod virtual_geometry_priority;

fn crate_batch(entities: Vec<u64>) -> VisibilityBatch {
    VisibilityBatch {
        key: crate_batch_key(),
        entities,
    }
}

fn crate_batch_key() -> VisibilityBatchKey {
    VisibilityBatchKey {
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(0x0000_0001),
        material_id: ResourceId::from_stable_label("res://materials/crate.zmaterial"),
        model_id: ResourceId::from_stable_label("res://models/crate.obj"),
        mobility: Mobility::Dynamic,
    }
}

fn draw_command(
    key: VisibilityBatchKey,
    visible_instance_offset: u32,
    visible_instance_count: u32,
) -> VisibilityDrawCommand {
    VisibilityDrawCommand {
        key,
        visible_instance_offset,
        visible_instance_count,
    }
}

fn draw_commands_for_batches(batches: &[VisibilityBatch]) -> Vec<VisibilityDrawCommand> {
    let mut offset = 0_u32;
    let mut commands = Vec::with_capacity(batches.len());
    for batch in batches {
        let count = u32::try_from(batch.entities.len()).expect("batch size should fit into u32");
        commands.push(draw_command(batch.key.clone(), offset, count));
        offset += count;
    }
    commands
}

fn remove_default_meshes(world: &mut World) {
    let mesh_entities = world
        .nodes()
        .iter()
        .filter(|node| node.mesh.is_some())
        .map(|node| node.id)
        .collect::<Vec<_>>();
    for entity in mesh_entities {
        assert!(world.remove_entity(entity));
    }
}

fn model_handle(label: &str) -> ResourceHandle<ModelMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn material_handle(label: &str) -> ResourceHandle<MaterialMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn virtual_cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    parent_cluster_id: Option<u32>,
    bounds_center: Vec3,
    screen_space_error: f32,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id: None,
        page_id,
        lod_level,
        parent_cluster_id,
        bounds_center,
        bounds_radius: 0.5,
        screen_space_error,
    }
}

fn virtual_page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}
