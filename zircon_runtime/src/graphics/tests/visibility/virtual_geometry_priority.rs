use crate::core::framework::render::RenderVirtualGeometryExtract;
use crate::core::math::{Transform, Vec3};
use crate::scene::world::World;

use crate::graphics::{
    VisibilityContext, VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryDrawSegment,
    VisibilityVirtualGeometryFeedback, VisibilityVirtualGeometryPageUploadPlan,
};

use super::{material_handle, model_handle, remove_default_meshes, virtual_cluster, virtual_page};
#[test]
fn visibility_context_only_holds_requested_virtual_geometry_lineage_when_frontier_budget_collapses()
{
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
        material_handle("res://materials/virtual_geometry.zmaterial"),
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
        material_handle("res://materials/virtual_geometry.zmaterial"),
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
        material_handle("res://materials/virtual_geometry.zmaterial"),
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
        material_handle("res://materials/virtual_geometry.zmaterial"),
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
