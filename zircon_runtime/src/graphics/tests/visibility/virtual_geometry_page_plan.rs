use crate::core::framework::render::RenderVirtualGeometryExtract;
use crate::core::math::{Transform, Vec3};
use crate::scene::world::World;

use crate::graphics::{
    VisibilityContext, VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};

use super::{material_handle, model_handle, remove_default_meshes, virtual_cluster, virtual_page};
#[test]
fn visibility_context_builds_virtual_geometry_visibility_feedback_and_page_plan() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world
        .spawn_mesh_node(
            model_handle("res://models/virtual_geometry.obj"),
            material_handle("res://materials/virtual_geometry.zmaterial"),
        )
        .expect("test mesh spawn should succeed");
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
                stable_instance_key: mesh << 16,
                cluster_id: 30,
                page_id: 300,
                lod_level: 0,
                cluster_ordinal: 3,
                cluster_count: 4,
                resident: false,
            },
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                stable_instance_key: mesh << 16,
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

    let mesh = world
        .spawn_mesh_node(
            model_handle("res://models/virtual_geometry.obj"),
            material_handle("res://materials/virtual_geometry.zmaterial"),
        )
        .expect("test mesh spawn should succeed");
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
fn visibility_context_refines_virtual_geometry_parent_cluster_into_visible_children_when_budget_allows()
 {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world
        .spawn_mesh_node(
            model_handle("res://models/virtual_geometry.obj"),
            material_handle("res://materials/virtual_geometry.zmaterial"),
        )
        .expect("test mesh spawn should succeed");
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
                stable_instance_key: mesh << 16,
                cluster_id: 20,
                page_id: 200,
                lod_level: 1,
                cluster_ordinal: 1,
                cluster_count: 3,
                resident: true,
            },
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                stable_instance_key: mesh << 16,
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
fn visibility_context_keeps_parent_virtual_geometry_cluster_visible_while_requesting_nonresident_children()
 {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world
        .spawn_mesh_node(
            model_handle("res://models/virtual_geometry.obj"),
            material_handle("res://materials/virtual_geometry.zmaterial"),
        )
        .expect("test mesh spawn should succeed");
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
            stable_instance_key: mesh << 16,
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
fn visibility_context_keeps_resident_virtual_geometry_children_visible_while_requesting_nonresident_grandchildren()
 {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world
        .spawn_mesh_node(
            model_handle("res://models/virtual_geometry.obj"),
            material_handle("res://materials/virtual_geometry.zmaterial"),
        )
        .expect("test mesh spawn should succeed");
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
                stable_instance_key: mesh << 16,
                cluster_id: 20,
                page_id: 200,
                lod_level: 1,
                cluster_ordinal: 1,
                cluster_count: 5,
                resident: true,
            },
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                stable_instance_key: mesh << 16,
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

    let mesh = world
        .spawn_mesh_node(
            model_handle("res://models/virtual_geometry.obj"),
            material_handle("res://materials/virtual_geometry.zmaterial"),
        )
        .expect("test mesh spawn should succeed");
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
            stable_instance_key: mesh << 16,
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
                stable_instance_key: mesh << 16,
                cluster_id: 20,
                page_id: 200,
                lod_level: 1,
                cluster_ordinal: 1,
                cluster_count: 3,
                resident: true,
            },
            VisibilityVirtualGeometryCluster {
                entity: mesh,
                stable_instance_key: mesh << 16,
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
