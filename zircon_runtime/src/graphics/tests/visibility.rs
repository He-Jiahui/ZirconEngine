use crate::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
};
use crate::core::math::{Transform, Vec3};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::scene::components::Mobility;
use crate::scene::world::World;

use crate::graphics::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBvhUpdateStrategy, VisibilityContext,
    VisibilityDrawCommand, VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryDrawSegment,
    VisibilityVirtualGeometryFeedback, VisibilityVirtualGeometryPageUploadPlan,
};

#[test]
fn visibility_context_partitions_static_and_dynamic_meshes() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

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
        material_handle("res://materials/crate.material.toml"),
    );
    let statue = world.spawn_mesh_node(
        model_handle("res://models/statue.obj"),
        material_handle("res://materials/statue.material.toml"),
    );
    let crate_b = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.material.toml"),
    );
    let tree = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.material.toml"),
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
                render_layer_mask: 0x0000_0008,
                material_id: ResourceId::from_stable_label("res://materials/statue.material.toml"),
                model_id: ResourceId::from_stable_label("res://models/statue.obj"),
                mobility: Mobility::Dynamic,
            },
            entities: vec![statue],
        },
        VisibilityBatch {
            key: VisibilityBatchKey {
                render_layer_mask: 0x0000_0008,
                material_id: ResourceId::from_stable_label("res://materials/tree.material.toml"),
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
        material_handle("res://materials/crate.material.toml"),
    );
    let culled = world.spawn_mesh_node(
        model_handle("res://models/crate.obj"),
        material_handle("res://materials/crate.material.toml"),
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

    assert_eq!(context.visible_entities, vec![visible]);
    assert_eq!(context.culled_entities, vec![culled]);
    assert_eq!(context.visible_batches, vec![crate_batch(vec![visible])]);
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
        material_handle("res://materials/crate.material.toml"),
    );
    let tree_entity = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.material.toml"),
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
        material_handle("res://materials/crate.material.toml"),
    );
    let removed = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.material.toml"),
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
        material_handle("res://materials/statue.material.toml"),
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
        material_handle("res://materials/crate.material.toml"),
    );
    let emitter_b = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.material.toml"),
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
        material_handle("res://materials/crate.material.toml"),
    );
    let removed_emitter = world.spawn_mesh_node(
        model_handle("res://models/tree.obj"),
        material_handle("res://materials/tree.material.toml"),
    );
    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.particles.emitters = vec![emitter_a, removed_emitter];
    let previous_context = VisibilityContext::from(&previous_extract);

    let inserted_emitter = world.spawn_mesh_node(
        model_handle("res://models/statue.obj"),
        material_handle("res://materials/statue.material.toml"),
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

#[test]
fn visibility_context_builds_virtual_geometry_visibility_feedback_and_page_plan() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![
            virtual_cluster(mesh, 15, 150, 1, None, Vec3::new(100.0, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 0, None, Vec3::new(0.0, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 20, 200, 1, None, Vec3::new(0.1, 0.0, 0.0), 5.0),
            virtual_cluster(mesh, 10, 100, 2, None, Vec3::new(0.2, 0.0, 0.0), 2.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, false),
            virtual_page(150, false),
            virtual_page(200, true),
            virtual_page(300, false),
            virtual_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.virtual_geometry_visible_clusters,
        vec![
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 30,
                page_id: 300,
                lod_level: 0,
                cluster_ordinal: 3,
                cluster_count: 4,
                resident: false,
            },
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 20,
                page_id: 200,
                lod_level: 1,
                cluster_ordinal: 2,
                cluster_count: 4,
                resident: true,
            },
        ]
    );
    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: vec![500],
        }
    );
    assert_eq!(
        context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![30, 20],
            requested_pages: vec![300],
            evictable_pages: vec![500],
            hot_resident_pages: Vec::new(),
        }
    );
    assert_eq!(
        context.history_snapshot.virtual_geometry_requested_pages,
        vec![300]
    );
}

#[test]
fn visibility_context_with_history_tracks_virtual_geometry_requested_pages() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 30, 300, 0, None, Vec3::new(0.0, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 20, 200, 1, None, Vec3::new(0.1, 0.0, 0.0), 5.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(200, true),
            virtual_page(300, false),
            virtual_page(700, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    let mut current_extract = world.to_render_frame_extract();
    current_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 60, 600, 0, None, Vec3::new(0.0, 0.0, 0.0), 10.0),
            virtual_cluster(mesh, 30, 300, 1, None, Vec3::new(0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 20, 200, 2, None, Vec3::new(0.2, 0.0, 0.0), 4.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(200, true),
            virtual_page(300, false),
            virtual_page(600, false),
            virtual_page(700, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 700],
            requested_pages: vec![600, 300],
            dirty_requested_pages: vec![600],
            evictable_pages: vec![700],
        }
    );
    assert_eq!(
        context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![60, 30, 20],
            requested_pages: vec![600, 300],
            evictable_pages: vec![700],
            hot_resident_pages: Vec::new(),
        }
    );
    assert_eq!(
        context.history_snapshot.virtual_geometry_requested_pages,
        vec![600, 300]
    );
}

#[test]
fn visibility_context_refines_virtual_geometry_parent_cluster_into_visible_children_when_budget_allows(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.virtual_geometry_visible_clusters,
        vec![
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 20,
                page_id: 200,
                lod_level: 1,
                cluster_ordinal: 1,
                cluster_count: 3,
                resident: true,
            },
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 30,
                page_id: 300,
                lod_level: 1,
                cluster_ordinal: 2,
                cluster_count: 3,
                resident: true,
            },
        ]
    );
    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 300],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: vec![100],
        }
    );
    assert_eq!(
        context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![20, 30],
            requested_pages: Vec::new(),
            evictable_pages: vec![100],
            hot_resident_pages: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_keeps_parent_virtual_geometry_cluster_visible_while_requesting_nonresident_children(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, false),
            virtual_page(300, false),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.virtual_geometry_visible_clusters,
        vec![VisibilityVirtualGeometryCluster {
            entity: mesh,
            cluster_id: 10,
            page_id: 100,
            lod_level: 0,
            cluster_ordinal: 0,
            cluster_count: 3,
            resident: true,
        }]
    );
    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100],
            requested_pages: vec![200, 300],
            dirty_requested_pages: vec![200, 300],
            evictable_pages: Vec::new(),
        }
    );
    assert_eq!(
        context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10],
            requested_pages: vec![200, 300],
            evictable_pages: Vec::new(),
            hot_resident_pages: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_keeps_resident_virtual_geometry_children_visible_while_requesting_nonresident_grandchildren(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 40, 400, 2, Some(20), Vec3::new(0.16, 0.0, 0.0), 6.5),
            virtual_cluster(mesh, 50, 500, 2, Some(30), Vec3::new(-0.16, 0.0, 0.0), 5.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
            virtual_page(400, false),
            virtual_page(500, false),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.virtual_geometry_visible_clusters,
        vec![
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 20,
                page_id: 200,
                lod_level: 1,
                cluster_ordinal: 1,
                cluster_count: 5,
                resident: true,
            },
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 30,
                page_id: 300,
                lod_level: 1,
                cluster_ordinal: 2,
                cluster_count: 5,
                resident: true,
            },
        ]
    );
    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 300],
            requested_pages: vec![400, 500],
            dirty_requested_pages: vec![400, 500],
            evictable_pages: vec![100],
        }
    );
    assert_eq!(
        context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![20, 30],
            requested_pages: vec![400, 500],
            evictable_pages: vec![100],
            hot_resident_pages: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_holds_resident_parent_one_frame_after_requested_children_become_resident() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, false),
            virtual_page(300, false),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    let mut current_extract = world.to_render_frame_extract();
    current_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let held_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        held_context.virtual_geometry_visible_clusters,
        vec![VisibilityVirtualGeometryCluster {
            entity: mesh,
            cluster_id: 10,
            page_id: 100,
            lod_level: 0,
            cluster_ordinal: 0,
            cluster_count: 3,
            resident: true,
        }]
    );
    assert_eq!(
        held_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 300],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        }
    );
    assert_eq!(
        held_context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10],
            requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
            hot_resident_pages: vec![200, 300],
        }
    );

    let settled_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&held_context.history_snapshot),
    );

    assert_eq!(
        settled_context.virtual_geometry_visible_clusters,
        vec![
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 20,
                page_id: 200,
                lod_level: 1,
                cluster_ordinal: 1,
                cluster_count: 3,
                resident: true,
            },
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 30,
                page_id: 300,
                lod_level: 1,
                cluster_ordinal: 2,
                cluster_count: 3,
                resident: true,
            },
        ]
    );
    assert_eq!(
        settled_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 300],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        }
    );
    assert_eq!(
        settled_context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![20, 30],
            requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
            hot_resident_pages: vec![100],
        }
    );

    let merge_ready_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&settled_context.history_snapshot),
    );

    assert_eq!(
        merge_ready_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 300],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: vec![100],
        }
    );
    assert_eq!(
        merge_ready_context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![20, 30],
            requested_pages: Vec::new(),
            evictable_pages: vec![100],
            hot_resident_pages: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_holds_resident_child_page_one_frame_when_frontier_merges_back_to_parent() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    assert_eq!(
        previous_context.virtual_geometry_feedback.visible_cluster_ids,
        vec![20, 30],
        "expected the fully resident frame to settle onto the child frontier before testing merge-back hysteresis"
    );

    let mut current_extract = world.to_render_frame_extract();
    current_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, false),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let held_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        held_context.virtual_geometry_visible_clusters,
        vec![VisibilityVirtualGeometryCluster {
            entity: mesh,
            cluster_id: 10,
            page_id: 100,
            lod_level: 0,
            cluster_ordinal: 0,
            cluster_count: 3,
            resident: true,
        }]
    );
    assert_eq!(
        held_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: Vec::new(),
        }
    );
    assert_eq!(
        held_context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10],
            requested_pages: vec![300],
            evictable_pages: Vec::new(),
            hot_resident_pages: vec![200],
        }
    );

    let settled_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&held_context.history_snapshot),
    );

    assert_eq!(
        settled_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200],
            requested_pages: vec![300],
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        }
    );
    assert_eq!(
        settled_context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10],
            requested_pages: vec![300],
            evictable_pages: Vec::new(),
            hot_resident_pages: vec![200],
        }
    );
}

#[test]
fn visibility_context_keeps_resident_child_frontier_hot_across_repeated_budget_collapse_without_pending_requests(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    assert_eq!(
        previous_context.virtual_geometry_feedback.visible_cluster_ids,
        vec![20, 30],
        "expected the fully resident previous frame to settle onto the child frontier before testing repeated budget-collapse hysteresis"
    );

    let mut collapsed_extract = world.to_render_frame_extract();
    collapsed_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let first_collapsed_context = VisibilityContext::from_extract_with_history(
        &collapsed_extract,
        Some(&previous_context.history_snapshot),
    );
    assert_eq!(
        first_collapsed_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 300],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
        "expected the first collapsed frame to keep the previously active resident child frontier hot while the visible frontier merges back to the coarse parent"
    );

    let settled_collapsed_context = VisibilityContext::from_extract_with_history(
        &collapsed_extract,
        Some(&first_collapsed_context.history_snapshot),
    );

    assert_eq!(
        settled_collapsed_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 300],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
        "expected repeated budget-collapse frames to keep the last fully resident child frontier hot instead of dropping it into evictable_pages after only one collapsed frame when no pending request is left to protect it"
    );
    assert_eq!(
        settled_collapsed_context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10],
            requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
            hot_resident_pages: vec![200, 300],
        }
    );
}

#[test]
fn visibility_context_requests_nonresident_ancestor_page_and_holds_descendants_when_frontier_collapses_multiple_levels(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 40, 400, 2, Some(20), Vec3::new(0.16, 0.0, 0.0), 6.5),
            virtual_cluster(mesh, 50, 500, 2, Some(30), Vec3::new(-0.16, 0.0, 0.0), 5.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
            virtual_page(400, true),
            virtual_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    assert_eq!(
        previous_context.virtual_geometry_feedback.visible_cluster_ids,
        vec![40, 50],
        "expected the fully resident previous frame to refine all the way to the grandchild frontier before testing multi-level merge-back"
    );

    let mut current_extract = world.to_render_frame_extract();
    current_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 40, 400, 2, Some(20), Vec3::new(0.16, 0.0, 0.0), 6.5),
            virtual_cluster(mesh, 50, 500, 2, Some(30), Vec3::new(-0.16, 0.0, 0.0), 5.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, false),
            virtual_page(400, true),
            virtual_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        context.virtual_geometry_visible_clusters,
        vec![VisibilityVirtualGeometryCluster {
            entity: mesh,
            cluster_id: 10,
            page_id: 100,
            lod_level: 0,
            cluster_ordinal: 0,
            cluster_count: 5,
            resident: true,
        }]
    );
    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 400, 500],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: Vec::new(),
        },
        "expected multi-level frontier collapse to request the missing ancestor page and keep the full still-hot resident lineage out of the first evictable set"
    );
    assert_eq!(
        context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10],
            requested_pages: vec![300],
            evictable_pages: Vec::new(),
            hot_resident_pages: vec![200, 400, 500],
        }
    );
}

#[test]
fn visibility_context_keeps_resident_grandchild_pages_hot_while_multi_level_cascade_request_remains_pending(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 40, 400, 2, Some(20), Vec3::new(0.16, 0.0, 0.0), 6.5),
            virtual_cluster(mesh, 50, 500, 2, Some(30), Vec3::new(-0.16, 0.0, 0.0), 5.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
            virtual_page(400, true),
            virtual_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    let mut current_extract = world.to_render_frame_extract();
    current_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 40, 400, 2, Some(20), Vec3::new(0.16, 0.0, 0.0), 6.5),
            virtual_cluster(mesh, 50, 500, 2, Some(30), Vec3::new(-0.16, 0.0, 0.0), 5.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, false),
            virtual_page(400, true),
            virtual_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let held_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );
    let settled_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&held_context.history_snapshot),
    );

    assert_eq!(
        settled_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 400, 500],
            requested_pages: vec![300],
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
        "expected deeper split-merge hysteresis to keep the full resident lineage hot while the ancestor cascade request is still pending, instead of exposing intermediate pages to eviction on the second collapsed frame"
    );
    assert_eq!(
        settled_context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10],
            requested_pages: vec![300],
            evictable_pages: Vec::new(),
            hot_resident_pages: vec![200, 400, 500],
        }
    );
}

#[test]
fn visibility_context_keeps_intermediate_virtual_geometry_lineage_pages_hot_while_ancestor_request_remains_pending(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.08, 0.0, 0.0), 10.0),
            virtual_cluster(mesh, 30, 300, 2, Some(20), Vec3::new(0.12, 0.0, 0.0), 7.0),
            virtual_cluster(mesh, 40, 400, 3, Some(30), Vec3::new(0.16, 0.0, 0.0), 5.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
            virtual_page(400, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    assert_eq!(
        previous_context.virtual_geometry_feedback.visible_cluster_ids,
        vec![40],
        "expected the fully resident frame to refine onto the deepest resident lineage before testing wider cascade hysteresis"
    );

    let mut current_extract = world.to_render_frame_extract();
    current_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.08, 0.0, 0.0), 10.0),
            virtual_cluster(mesh, 30, 300, 2, Some(20), Vec3::new(0.12, 0.0, 0.0), 7.0),
            virtual_cluster(mesh, 40, 400, 3, Some(30), Vec3::new(0.16, 0.0, 0.0), 5.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, false),
            virtual_page(300, true),
            virtual_page(400, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        context.virtual_geometry_visible_clusters,
        vec![VisibilityVirtualGeometryCluster {
            entity: mesh,
            cluster_id: 10,
            page_id: 100,
            lod_level: 0,
            cluster_ordinal: 0,
            cluster_count: 4,
            resident: true,
        }]
    );
    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 300, 400],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: Vec::new(),
        },
        "expected wider cascade hysteresis to keep the intermediate resident lineage pages hot while the missing ancestor page request remains pending, instead of exposing the lineage to eviction before the hierarchy reconnects"
    );
    assert_eq!(
        context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10],
            requested_pages: vec![200],
            evictable_pages: Vec::new(),
            hot_resident_pages: vec![300, 400],
        }
    );
}

#[test]
fn visibility_context_only_holds_requested_virtual_geometry_lineage_when_frontier_budget_collapses()
{
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.10, 0.0, 0.0), 10.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.10, 0.0, 0.0), 9.5),
            virtual_cluster(mesh, 40, 400, 2, Some(20), Vec3::new(0.15, 0.0, 0.0), 7.0),
            virtual_cluster(mesh, 50, 500, 2, Some(30), Vec3::new(-0.15, 0.0, 0.0), 6.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
            virtual_page(400, true),
            virtual_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    let previous_context = VisibilityContext::from(&previous_extract);

    assert_eq!(
        previous_context.virtual_geometry_feedback.visible_cluster_ids,
        vec![40, 50],
        "expected the fully resident previous frame to refine onto both sibling lineages before testing frontier-budget collapse protection"
    );

    let mut current_extract = world.to_render_frame_extract();
    current_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 3,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.10, 0.0, 0.0), 10.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.10, 0.0, 0.0), 9.5),
            virtual_cluster(mesh, 40, 400, 2, Some(20), Vec3::new(0.15, 0.0, 0.0), 7.0),
            virtual_cluster(mesh, 50, 500, 2, Some(30), Vec3::new(-0.15, 0.0, 0.0), 6.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, false),
            virtual_page(300, true),
            virtual_page(400, true),
            virtual_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let held_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );
    let settled_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&held_context.history_snapshot),
    );

    assert_eq!(
        held_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 300, 400, 500],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![300],
        },
        "expected pending-cascade protection to stay confined to the requested lineage on the first collapsed frame so unrelated sibling lineage pages can re-enter eviction pressure immediately"
    );
    assert_eq!(
        settled_context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 300, 400, 500],
            requested_pages: vec![200],
            dirty_requested_pages: Vec::new(),
            evictable_pages: vec![300, 500],
        },
        "expected wider split-merge policy to keep only the requested lineage hot across repeated collapsed frames instead of pinning an unrelated sibling subtree behind the same visible frontier ancestor"
    );
}

#[test]
fn visibility_context_splits_virtual_geometry_draw_segments_across_parent_lineages_even_when_page_matches(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 40, 400, 2, Some(20), Vec3::new(0.16, 0.0, 0.0), 6.5),
            virtual_cluster(mesh, 50, 400, 2, Some(30), Vec3::new(-0.16, 0.0, 0.0), 5.5),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, true),
            virtual_page(200, true),
            virtual_page(300, true),
            virtual_page(400, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.virtual_geometry_visible_clusters,
        vec![
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 40,
                page_id: 400,
                lod_level: 2,
                cluster_ordinal: 3,
                cluster_count: 5,
                resident: true,
            },
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                cluster_id: 50,
                page_id: 400,
                lod_level: 2,
                cluster_ordinal: 4,
                cluster_count: 5,
                resident: true,
            },
        ]
    );
    assert_eq!(
        context.virtual_geometry_draw_segments,
        vec![
            VisibilityVirtualGeometryDrawSegment {
                entity: mesh,
                cluster_id: 40,
                page_id: 400,
                cluster_ordinal: 3,
                cluster_span_count: 1,
                cluster_count: 5,
                lineage_depth: 2,
                lod_level: 2,
            },
            VisibilityVirtualGeometryDrawSegment {
                entity: mesh,
                cluster_id: 50,
                page_id: 400,
                cluster_ordinal: 4,
                cluster_span_count: 1,
                cluster_count: 5,
                lineage_depth: 2,
                lod_level: 2,
            },
        ],
        "expected visibility-owned unified indirect boundaries to stay split across different parent lineages even when the refined clusters share one resident page"
    );
}

#[test]
fn visibility_context_keeps_parent_virtual_geometry_cluster_when_children_exceed_budget() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::ZERO, 12.0),
            virtual_cluster(mesh, 20, 200, 1, Some(10), Vec3::new(0.1, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 30, 300, 1, Some(10), Vec3::new(-0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 40, 400, 1, Some(10), Vec3::new(0.0, 0.1, 0.0), 7.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, false),
            virtual_page(200, false),
            virtual_page(300, false),
            virtual_page(400, false),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.virtual_geometry_visible_clusters,
        vec![VisibilityVirtualGeometryCluster {
            entity: mesh,
            cluster_id: 10,
            page_id: 100,
            lod_level: 0,
            cluster_ordinal: 0,
            cluster_count: 4,
            resident: false,
        }]
    );
    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: Vec::new(),
            requested_pages: vec![100],
            dirty_requested_pages: vec![100],
            evictable_pages: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_prioritizes_virtual_geometry_pages_backing_more_visible_clusters_when_page_budget_is_tight(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 1,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::new(0.0, 0.0, 0.0), 12.0),
            virtual_cluster(mesh, 20, 200, 1, None, Vec3::new(0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 30, 200, 1, None, Vec3::new(-0.1, 0.0, 0.0), 7.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, false),
            virtual_page(200, false),
            virtual_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![500],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![500],
        }
    );
    assert_eq!(
        context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10, 20, 30],
            requested_pages: vec![200],
            evictable_pages: vec![500],
            hot_resident_pages: Vec::new(),
        }
    );
}

#[test]
fn visibility_context_uses_aggregate_screen_space_error_to_break_virtual_geometry_page_priority_ties(
) {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 1,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, None, Vec3::new(0.0, 0.0, 0.0), 7.5),
            virtual_cluster(mesh, 11, 100, 0, None, Vec3::new(0.1, 0.0, 0.0), 6.5),
            virtual_cluster(mesh, 20, 200, 0, None, Vec3::new(-0.1, 0.0, 0.0), 6.0),
            virtual_cluster(mesh, 21, 200, 0, None, Vec3::new(-0.2, 0.0, 0.0), 5.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![virtual_page(100, false), virtual_page(200, false)],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context.virtual_geometry_page_upload_plan,
        VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: Vec::new(),
            requested_pages: vec![100],
            dirty_requested_pages: vec![100],
            evictable_pages: Vec::new(),
        }
    );
    assert_eq!(
        context.virtual_geometry_feedback,
        VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: vec![10, 11, 20, 21],
            requested_pages: vec![100],
            evictable_pages: Vec::new(),
            hot_resident_pages: Vec::new(),
        }
    );
}

fn crate_batch(entities: Vec<u64>) -> VisibilityBatch {
    VisibilityBatch {
        key: crate_batch_key(),
        entities,
    }
}

fn crate_batch_key() -> VisibilityBatchKey {
    VisibilityBatchKey {
        render_layer_mask: 0x0000_0001,
        material_id: ResourceId::from_stable_label("res://materials/crate.material.toml"),
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
        commands.push(draw_command(batch.key, offset, count));
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
