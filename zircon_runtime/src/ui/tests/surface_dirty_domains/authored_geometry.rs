use std::collections::BTreeSet;

use super::*;
use crate::ui::surface::{
    hit_test_surface_frame, UiAuthoredGeometryFallbackReason, UiAuthoredGeometryPublication,
};

#[test]
fn exact_authored_geometry_updates_frame_and_instance_hit_authority() {
    let mut surface = authored_geometry_surface(false);
    let topology_generation = surface.tree.layout_order_generation();
    let before = surface.surface_frame();
    let before_generations = before.domain_generations;

    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .layout_cache
        .frame = UiFrame::new(72.0, 10.0, 40.0, 20.0);

    let publication = surface.publish_authored_geometry(
        root_size(),
        &BTreeSet::from([button_id()]),
        topology_generation,
    );
    let UiAuthoredGeometryPublication::Local(report) = publication else {
        panic!("stable authored geometry should patch locally: {publication:?}");
    };

    assert_eq!(report.arranged_outer_node_visit_count, 1);
    assert_eq!(report.hit_grid_outer_node_visit_count, 1);
    assert_eq!(report.render_outer_node_visit_count, 1);
    assert_eq!(surface.hit_test(UiPoint::new(12.0, 14.0)).top_hit, None);
    assert_eq!(
        surface.hit_test(UiPoint::new(76.0, 14.0)).top_hit,
        Some(button_id())
    );

    let after = surface.surface_frame();
    assert_eq!(
        before.arranged_tree.get(button_id()).unwrap().frame,
        UiFrame::new(8.0, 10.0, 40.0, 20.0),
        "a retained frame must keep the pre-patch arranged segment"
    );
    assert_eq!(
        hit_test_surface_frame(&before, UiPoint::new(12.0, 14.0)).top_hit,
        Some(button_id()),
        "a retained frame must keep the pre-patch hit entry and cell membership"
    );
    assert_eq!(
        hit_test_surface_frame(&before, UiPoint::new(76.0, 14.0)).top_hit,
        None
    );
    assert!(after.domain_generations.layout > before_generations.layout);
    assert!(after.domain_generations.hit_test > before_generations.hit_test);
    assert!(after.domain_generations.render > before_generations.render);
    for point in [UiPoint::new(12.0, 14.0), UiPoint::new(76.0, 14.0)] {
        assert_eq!(
            hit_test_surface_frame(&after, point),
            surface.hit_test(point),
            "published frame and instance must share one hit authority"
        );
    }
}

#[test]
fn empty_authored_geometry_delta_reuses_the_published_frame() {
    let mut surface = authored_geometry_surface(false);
    let topology_generation = surface.tree.layout_order_generation();
    let before = surface.surface_frame();

    let publication =
        surface.publish_authored_geometry(root_size(), &BTreeSet::new(), topology_generation);

    assert_eq!(publication, UiAuthoredGeometryPublication::Unchanged);
    assert!(std::sync::Arc::ptr_eq(&before, &surface.surface_frame()));
}

#[test]
fn stale_topology_generation_selects_typed_full_fallback() {
    let mut surface = authored_geometry_surface(false);
    let stale_generation = surface.tree.layout_order_generation();
    surface
        .tree
        .insert_child(
            root_id(),
            UiTreeNode::new(sibling_id(), UiNodePath::new("root/sibling"))
                .with_frame(UiFrame::new(50.0, 10.0, 20.0, 20.0)),
        )
        .unwrap();

    let publication = surface.publish_authored_geometry(
        root_size(),
        &BTreeSet::from([button_id()]),
        stale_generation,
    );

    assert!(matches!(
        publication,
        UiAuthoredGeometryPublication::FullFallback {
            reason: UiAuthoredGeometryFallbackReason::TopologyGenerationChanged,
            ..
        }
    ));
    assert_eq!(
        surface.arranged_tree.nodes.len(),
        surface.tree.nodes.len(),
        "fallback must publish the complete new topology"
    );
}

#[test]
fn root_resize_regrids_hit_authority_without_full_surface_fallback() {
    let mut surface = authored_geometry_surface(false);
    let topology_generation = surface.tree.layout_order_generation();
    let before = surface.surface_frame();
    surface
        .tree
        .node_mut(root_id())
        .expect("root node should exist")
        .layout_cache
        .frame = UiFrame::new(0.0, 0.0, 600.0, 60.0);
    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .layout_cache
        .frame = UiFrame::new(500.0, 10.0, 40.0, 20.0);

    let publication = surface.publish_authored_geometry(
        UiSize::new(600.0, root_size().height),
        &BTreeSet::from([root_id(), button_id()]),
        topology_generation,
    );
    let UiAuthoredGeometryPublication::Local(report) = publication else {
        panic!("root resize must stay inside the authored transaction: {publication:?}");
    };

    assert_eq!(report.arranged_outer_node_visit_count, 2);
    assert_eq!(report.render_outer_node_visit_count, 2);
    assert_eq!(report.hit_grid_outer_node_visit_count, 2);
    assert_eq!(surface.hit_test(UiPoint::new(12.0, 14.0)).top_hit, None);
    assert_eq!(
        surface.hit_test(UiPoint::new(504.0, 14.0)).top_hit,
        Some(button_id())
    );
    assert_eq!(
        surface
            .surface_frame()
            .arranged_tree
            .get(button_id())
            .unwrap()
            .frame,
        UiFrame::new(500.0, 10.0, 40.0, 20.0)
    );
    assert_eq!(
        hit_test_surface_frame(&before, UiPoint::new(12.0, 14.0)).top_hit,
        Some(button_id()),
        "the retained pre-resize frame must keep its old hit authority"
    );
    let after = surface.surface_frame();
    for point in [UiPoint::new(12.0, 14.0), UiPoint::new(504.0, 14.0)] {
        assert_eq!(
            hit_test_surface_frame(&after, point),
            surface.hit_test(point),
            "regridded frame and instance must share one hit authority"
        );
    }
}

#[test]
fn authored_geometry_missing_render_cache_selects_render_fallback() {
    let mut surface = authored_geometry_surface(false);
    assert!(!surface.render_extract.list.commands.is_empty());
    surface.render_cache = Default::default();
    let topology_generation = surface.tree.layout_order_generation();
    surface
        .tree
        .node_mut(button_id())
        .expect("button node should exist")
        .layout_cache
        .frame = UiFrame::new(12.0, 10.0, 40.0, 20.0);

    let publication = surface.publish_authored_geometry(
        root_size(),
        &BTreeSet::from([button_id()]),
        topology_generation,
    );

    assert!(matches!(
        publication,
        UiAuthoredGeometryPublication::FullFallback {
            reason: UiAuthoredGeometryFallbackReason::RenderCommandNotGeometryPatchable,
            ..
        }
    ));
    assert_eq!(
        surface
            .surface_frame()
            .arranged_tree
            .get(button_id())
            .unwrap()
            .frame,
        UiFrame::new(12.0, 10.0, 40.0, 20.0)
    );
}

#[test]
fn authored_clip_owner_expands_only_to_its_descendants() {
    let mut surface = authored_geometry_surface(true);
    let topology_generation = surface.tree.layout_order_generation();
    let sibling_before = surface.arranged_tree.get(sibling_id()).unwrap().clone();
    surface
        .tree
        .node_mut(root_id())
        .expect("root node should exist")
        .layout_cache
        .frame = UiFrame::new(4.0, 0.0, 120.0, 60.0);

    let publication = surface.publish_authored_geometry(
        root_size(),
        &BTreeSet::from([root_id()]),
        topology_generation,
    );
    let UiAuthoredGeometryPublication::Local(report) = publication else {
        panic!("clip owner translation should patch locally: {publication:?}");
    };

    assert_eq!(report.arranged_outer_node_visit_count, 2);
    assert_eq!(report.hit_grid_outer_node_visit_count, 2);
    assert_eq!(report.render_outer_node_visit_count, 2);
    assert_eq!(
        surface.arranged_tree.get(sibling_id()),
        Some(&sibling_before)
    );
}

fn authored_geometry_surface(include_sibling_root: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.authored_geometry"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id(), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 60.0))
            .with_layout_boundary(LayoutBoundary::ParentDirected),
    );
    surface
        .tree
        .node_mut(root_id())
        .expect("root node should exist")
        .clip_to_bounds = true;
    surface
        .tree
        .insert_child(
            root_id(),
            UiTreeNode::new(button_id(), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(8.0, 10.0, 40.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    if include_sibling_root {
        surface.tree.insert_root(
            UiTreeNode::new(sibling_id(), UiNodePath::new("sibling"))
                .with_frame(UiFrame::new(160.0, 0.0, 20.0, 20.0)),
        );
    }
    surface.rebuild_authored_frames(root_size());
    surface
}
