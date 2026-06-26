use crate::core::framework::render::RenderVirtualGeometryExtract;
use crate::core::math::{Transform, Vec3};
use crate::scene::world::World;

use crate::graphics::{
    VisibilityContext, VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};

use super::{material_handle, model_handle, remove_default_meshes, virtual_cluster, virtual_page};
#[test]
fn visibility_context_holds_resident_child_page_one_frame_when_frontier_merges_back_to_parent() {
    let mut world = World::new();
    remove_default_meshes(&mut world);

    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.zmaterial"),
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
        material_handle("res://materials/virtual_geometry.zmaterial"),
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
        material_handle("res://materials/virtual_geometry.zmaterial"),
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
        material_handle("res://materials/virtual_geometry.zmaterial"),
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
        material_handle("res://materials/virtual_geometry.zmaterial"),
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
