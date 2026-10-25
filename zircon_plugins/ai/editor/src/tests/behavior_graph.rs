use std::collections::BTreeMap;

use zircon_editor::ui::graph::{
    GraphConnectRejection, GraphEdgeView, GraphNodeBounds, GraphPoint, GraphPortRef,
    GraphRouteStyle,
};

use crate::{
    AiBehaviorTreeConnectionRequest as Connection, AiBehaviorTreeEdgeRouteRequest as EdgeRoute,
    AiBehaviorTreeGraphAdapter as Adapter, AiBehaviorTreeGraphError as GraphError,
    AiBehaviorTreeGraphGeometry as Geometry,
};

use super::{asset, compiled_children, effect_tree, subtree_node, Asset, Edit, Error, Kind, Node};

fn geometry(source: &Asset) -> BTreeMap<String, GraphNodeBounds> {
    source
        .descriptor()
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            (
                node.id.clone(),
                GraphNodeBounds::new(
                    GraphPoint::new(index as f32 * 21.0 - 100.0, index as f32 * 33.0),
                    GraphPoint::new(120.0, 64.0),
                ),
            )
        })
        .collect()
}

fn edge_request<'a>(
    source: &'a Asset,
    edge: &'a GraphEdgeView<String>,
    revision: u64,
) -> EdgeRoute<'a> {
    EdgeRoute {
        tree_id: &source.descriptor().id,
        revision,
        edge,
        source_anchor: GraphPoint::new(20.0, 30.0),
        target_anchor: GraphPoint::new(50.0, 200.0),
        style: GraphRouteStyle::Bezier,
    }
}

#[test]
fn projection_uses_supplied_geometry_and_real_wrapper_and_leaf_kinds() {
    let source = asset()
        .edited(&Edit::WrapDecorator {
            child_id: "set_score".to_owned(),
            decorator: Node::new("guard", Kind::Decorator, "Invert")
                .with_implementation("inverter"),
        })
        .unwrap()
        .unwrap();
    let source = source
        .edited(&Edit::InsertChild {
            parent_id: "root".to_owned(),
            index: 3,
            node: Node::new("service", Kind::Service, "Distance")
                .with_implementation("update_blackboard_distance"),
        })
        .unwrap()
        .unwrap();
    let source = source
        .edited(&Edit::InsertChild {
            parent_id: "root".to_owned(),
            index: 4,
            node: subtree_node(),
        })
        .unwrap()
        .unwrap();
    let bounds = geometry(&source);
    let view = Adapter::new(&source, 12)
        .project(Geometry {
            tree_id: &source.descriptor().id,
            revision: 12,
            node_bounds: &bounds,
        })
        .unwrap();
    assert_eq!(view.tree_id, source.descriptor().id);
    assert_eq!(view.revision, 12);
    assert_eq!(view.nodes.len(), source.descriptor().nodes.len());
    assert_eq!(view.edges.len(), view.nodes.len() - 1);
    for node in &view.nodes {
        assert_eq!(node.bounds, bounds[&node.id]);
        assert!(node.attachments.is_empty());
        assert_eq!(node.inputs.is_empty(), node.id == "root");
        if [
            "set_score",
            "emit_guard_event",
            "wait",
            "service",
            "call_patrol",
        ]
        .contains(&node.id.as_str())
        {
            assert!(node.outputs.is_empty());
        }
    }
    let guard = view.nodes.iter().find(|node| node.id == "guard").unwrap();
    assert_eq!(guard.outputs[0].name, "child");
    let children: Vec<_> = view
        .edges
        .iter()
        .filter(|edge| edge.from.node_id == "root")
        .map(|edge| edge.to.node_id.as_str())
        .collect();
    assert_eq!(
        children,
        [
            "guard",
            "emit_guard_event",
            "wait",
            "service",
            "call_patrol"
        ]
    );
    assert!(view.edges.iter().any(|edge| {
        edge.from.node_id == "guard"
            && edge.from.pin_name == "child"
            && edge.to.node_id == "set_score"
    }));
}

#[test]
fn projection_rejects_foreign_stale_incomplete_and_invalid_geometry() {
    let source = asset();
    let adapter = Adapter::new(&source, 7);
    let mut bounds = geometry(&source);
    assert!(matches!(
        adapter.project(Geometry {
            tree_id: "other",
            revision: 7,
            node_bounds: &bounds
        }),
        Err(GraphError::TreeIdentity { .. })
    ));
    assert!(matches!(
        adapter.project(Geometry {
            tree_id: &source.descriptor().id,
            revision: 8,
            node_bounds: &bounds
        }),
        Err(GraphError::Revision { .. })
    ));
    let removed = bounds.remove("wait").unwrap();
    assert!(matches!(
        adapter.project(Geometry {
            tree_id: &source.descriptor().id,
            revision: 7,
            node_bounds: &bounds
        }),
        Err(GraphError::GeometryCount { .. })
    ));
    bounds.insert("unrelated".to_owned(), removed);
    assert!(matches!(
        adapter.project(Geometry {
            tree_id: &source.descriptor().id,
            revision: 7,
            node_bounds: &bounds
        }),
        Err(GraphError::MissingGeometry { .. })
    ));
    bounds.remove("unrelated");
    for bad in [
        GraphNodeBounds::new(GraphPoint::new(f32::NAN, 0.0), GraphPoint::new(1.0, 1.0)),
        GraphNodeBounds::new(GraphPoint::ZERO, GraphPoint::new(0.0, 1.0)),
        GraphNodeBounds::new(GraphPoint::ZERO, GraphPoint::new(1.0, -1.0)),
        GraphNodeBounds::new(
            GraphPoint::new(f32::MAX, 0.0),
            GraphPoint::new(f32::MAX, 1.0),
        ),
    ] {
        bounds.insert("wait".to_owned(), bad);
        assert!(matches!(
            adapter.project(Geometry {
                tree_id: &source.descriptor().id,
                revision: 7,
                node_bounds: &bounds
            }),
            Err(GraphError::InvalidGeometry { .. })
        ));
    }
}

#[test]
fn replace_connection_uses_one_validated_reparent_and_post_removal_index() {
    let source = asset();
    let before = source.to_toml().to_owned();
    let from = GraphPortRef::output("root".to_owned(), "children");
    let to = GraphPortRef::input("set_score".to_owned(), "parent");
    let adapter = Adapter::new(&source, 41);
    let candidate = adapter
        .candidate_for_connection(Connection {
            tree_id: &source.descriptor().id,
            revision: 41,
            from: &from,
            to: &to,
            index_after_removal: 2,
        })
        .unwrap();
    assert_eq!(candidate.tree_id(), source.descriptor().id);
    assert_eq!(candidate.revision(), 41);
    assert!(
        matches!(candidate.edit(), Edit::Reparent { node_id, parent_id, index: 2 }
        if node_id == "set_score" && parent_id == "root")
    );
    assert_eq!(
        compiled_children(candidate.candidate().unwrap(), "root"),
        ["emit_guard_event", "wait", "set_score"]
    );
    let changed = candidate.into_candidate().unwrap();
    assert_eq!(
        Asset::from_toml(changed.to_toml()).unwrap().descriptor(),
        changed.descriptor()
    );
    let noop = adapter
        .candidate_for_connection(Connection {
            tree_id: &source.descriptor().id,
            revision: 41,
            from: &from,
            to: &to,
            index_after_removal: 0,
        })
        .unwrap();
    assert!(noop.candidate().is_none());
    assert!(adapter
        .candidate_for_connection(Connection {
            tree_id: &source.descriptor().id,
            revision: 41,
            from: &from,
            to: &to,
            index_after_removal: 3,
        })
        .is_err());
    assert_eq!(source.to_toml(), before);
    assert_eq!(
        compiled_children(&source, "root"),
        ["set_score", "emit_guard_event", "wait"]
    );
}

#[test]
fn connection_candidates_check_tree_revision_direction_and_exact_ports() {
    let source = asset();
    let adapter = Adapter::new(&source, 9);
    let from = GraphPortRef::output("root".to_owned(), "children");
    let to = GraphPortRef::input("wait".to_owned(), "parent");
    let attempt =
        |tree_id: &str, revision, from: &GraphPortRef<String>, to: &GraphPortRef<String>| {
            adapter.candidate_for_connection(Connection {
                tree_id,
                revision,
                from,
                to,
                index_after_removal: 0,
            })
        };
    assert!(matches!(
        attempt("foreign", 9, &from, &to),
        Err(GraphError::TreeIdentity { .. })
    ));
    assert!(matches!(
        attempt(&source.descriptor().id, 8, &from, &to),
        Err(GraphError::Revision { .. })
    ));
    assert!(matches!(
        attempt(&source.descriptor().id, 9, &to, &from),
        Err(GraphError::Connection(
            GraphConnectRejection::InvalidPinDirection
        ))
    ));
    for invalid in [
        GraphPortRef::output("set_score".to_owned(), "children"),
        GraphPortRef::output("root".to_owned(), "child"),
    ] {
        assert!(matches!(
            attempt(&source.descriptor().id, 9, &invalid, &to),
            Err(GraphError::Connection(GraphConnectRejection::MissingPin))
        ));
    }
    assert!(matches!(
        attempt(
            &source.descriptor().id,
            9,
            &GraphPortRef::output("missing".to_owned(), "children"),
            &to
        ),
        Err(GraphError::Connection(GraphConnectRejection::MissingNode))
    ));
    assert!(matches!(
        attempt(
            &source.descriptor().id,
            9,
            &from,
            &GraphPortRef::input("root".to_owned(), "parent")
        ),
        Err(GraphError::Connection(GraphConnectRejection::SameNode))
    ));
}

#[test]
fn whole_connection_candidate_validation_rejects_cycles_and_empty_wrappers() {
    let source = Asset::from_descriptor(
        super::Tree::new("cycle", "Cycle", "root")
            .with_node(
                Node::new("root", Kind::Sequence, "Root")
                    .with_child("branch")
                    .with_child("other"),
            )
            .with_node(Node::new("branch", Kind::Sequence, "Branch").with_child("nested"))
            .with_node(Node::new("nested", Kind::Sequence, "Nested").with_child("leaf"))
            .with_node(Node::new("other", Kind::Task, "Other"))
            .with_node(Node::new("leaf", Kind::Task, "Leaf")),
    )
    .unwrap();
    let from = GraphPortRef::output("nested".to_owned(), "children");
    let to = GraphPortRef::input("branch".to_owned(), "parent");
    assert!(matches!(
        Adapter::new(&source, 3).candidate_for_connection(Connection {
            tree_id: "cycle",
            revision: 3,
            from: &from,
            to: &GraphPortRef::input("root".to_owned(), "parent"),
            index_after_removal: 1,
        }),
        Err(GraphError::Connection(GraphConnectRejection::MissingPin))
    ));
    assert!(matches!(
        Adapter::new(&source, 3).candidate_for_connection(Connection {
            tree_id: "cycle",
            revision: 3,
            from: &from,
            to: &to,
            index_after_removal: 1,
        }),
        Err(GraphError::Candidate(_))
    ));
    assert_eq!(compiled_children(&source, "root"), ["branch", "other"]);
    let wrapped = asset()
        .edited(&Edit::WrapDecorator {
            child_id: "set_score".to_owned(),
            decorator: Node::new("guard", Kind::Decorator, "Invert")
                .with_implementation("inverter"),
        })
        .unwrap()
        .unwrap();
    assert!(matches!(
        Adapter::new(&wrapped, 4).candidate_for_connection(Connection {
            tree_id: &wrapped.descriptor().id,
            revision: 4,
            from: &GraphPortRef::output("root".to_owned(), "children"),
            to: &GraphPortRef::input("set_score".to_owned(), "parent"),
            index_after_removal: 3,
        }),
        Err(GraphError::Candidate(_))
    ));
    assert_eq!(compiled_children(&wrapped, "guard"), ["set_score"]);
}

#[test]
fn connection_candidates_preserve_the_weighted_topology_guard() {
    let mut tree = effect_tree();
    tree.nodes[0].implementation = "random_selector".to_owned();
    tree.nodes[0]
        .parameters
        .push(super::Parameter::new("weight.set_score", 3.0_f32));
    let source = Asset::from_descriptor(tree).unwrap();
    assert!(matches!(
        Adapter::new(&source, 5).candidate_for_connection(Connection {
            tree_id: &source.descriptor().id,
            revision: 5,
            from: &GraphPortRef::output("root".to_owned(), "children"),
            to: &GraphPortRef::input("set_score".to_owned(), "parent"),
            index_after_removal: 2,
        }),
        Err(GraphError::Candidate(Error::WeightedTopology { .. }))
    ));
    assert_eq!(
        compiled_children(&source, "root"),
        ["set_score", "emit_guard_event", "wait"]
    );
}

#[test]
fn existing_edge_routes_use_supplied_anchors_and_vertical_tangents() {
    let source = asset();
    let adapter = Adapter::new(&source, 23);
    let edge = GraphEdgeView::new(
        GraphPortRef::output("root".to_owned(), "children"),
        GraphPortRef::input("set_score".to_owned(), "parent"),
    );
    // Unit geometry fixtures exercise the external shared API, not a measured engine frame.
    for (start, end) in [
        (GraphPoint::new(75.0, 40.0), GraphPoint::new(150.0, 240.0)),
        (GraphPoint::new(-30.0, 240.0), GraphPoint::new(80.0, -20.0)),
    ] {
        for style in [GraphRouteStyle::Bezier, GraphRouteStyle::Orthogonal] {
            let route = adapter
                .route_edge(EdgeRoute {
                    tree_id: &source.descriptor().id,
                    revision: 23,
                    edge: &edge,
                    source_anchor: start,
                    target_anchor: end,
                    style,
                })
                .unwrap();
            assert_eq!(route.style(), style);
            assert_eq!(route.points().first(), Some(&start));
            assert_eq!(route.points().last(), Some(&end));
            assert_eq!(
                route.endpoint_tangents(),
                Some((GraphPoint::new(0.0, 1.0), GraphPoint::new(0.0, 1.0)))
            );
            let arrow = route.target_arrow().unwrap();
            assert_eq!(arrow.position, end);
            assert_eq!(arrow.direction, GraphPoint::new(0.0, 1.0));
        }
    }
}

#[test]
fn edge_routes_reject_foreign_stale_and_fabricated_edges() {
    let source = asset()
        .edited(&Edit::WrapDecorator {
            child_id: "set_score".to_owned(),
            decorator: Node::new("guard", Kind::Decorator, "Invert")
                .with_implementation("inverter"),
        })
        .unwrap()
        .unwrap();
    let before = source.clone();
    let adapter = Adapter::new(&source, 8);
    let edge = GraphEdgeView::new(
        GraphPortRef::output("root".to_owned(), "children"),
        GraphPortRef::input("guard".to_owned(), "parent"),
    );
    let request = || EdgeRoute {
        tree_id: &source.descriptor().id,
        revision: 8,
        edge: &edge,
        source_anchor: GraphPoint::new(20.0, 30.0),
        target_anchor: GraphPoint::new(50.0, 200.0),
        style: GraphRouteStyle::Bezier,
    };
    let mut foreign = request();
    foreign.tree_id = "other-tree";
    assert!(matches!(
        adapter.route_edge(foreign),
        Err(GraphError::TreeIdentity { .. })
    ));
    let mut stale = request();
    stale.revision = 7;
    assert!(matches!(
        adapter.route_edge(stale),
        Err(GraphError::Revision { .. })
    ));
    let fabricated = GraphEdgeView::new(
        GraphPortRef::output("root".to_owned(), "children"),
        GraphPortRef::input("set_score".to_owned(), "parent"),
    );
    let mut absent = request();
    absent.edge = &fabricated;
    assert!(matches!(
        adapter.route_edge(absent),
        Err(GraphError::MissingEdge)
    ));
    let reversed = GraphEdgeView::new(edge.to.clone(), edge.from.clone());
    let mut reversed_request = request();
    reversed_request.edge = &reversed;
    assert!(matches!(
        adapter.route_edge(reversed_request),
        Err(GraphError::Connection(
            GraphConnectRejection::InvalidPinDirection
        ))
    ));
    assert_eq!(source, before);
}

#[test]
fn edge_routes_keep_wrapper_identity_and_reject_leaf_outputs() {
    let mut source = asset()
        .edited(&Edit::WrapDecorator {
            child_id: "set_score".to_owned(),
            decorator: Node::new("guard", Kind::Decorator, "Invert")
                .with_implementation("inverter"),
        })
        .unwrap()
        .unwrap();
    for (index, node) in [
        (
            3,
            Node::new("service", Kind::Service, "Distance")
                .with_implementation("update_blackboard_distance"),
        ),
        (4, subtree_node()),
    ] {
        source = source
            .edited(&Edit::InsertChild {
                parent_id: "root".to_owned(),
                index,
                node,
            })
            .unwrap()
            .unwrap();
    }
    let adapter = Adapter::new(&source, 9);
    let wrapper_edge = GraphEdgeView::new(
        GraphPortRef::output("guard".to_owned(), "child"),
        GraphPortRef::input("set_score".to_owned(), "parent"),
    );
    assert!(adapter
        .route_edge(edge_request(&source, &wrapper_edge, 9))
        .is_ok());
    for leaf_id in ["set_score", "service", "call_patrol"] {
        let edge = GraphEdgeView::new(
            GraphPortRef::output(leaf_id.to_owned(), "children"),
            GraphPortRef::input("emit_guard_event".to_owned(), "parent"),
        );
        assert!(matches!(
            adapter.route_edge(edge_request(&source, &edge, 9)),
            Err(GraphError::Connection(GraphConnectRejection::MissingPin))
        ));
    }
    assert_eq!(compiled_children(&source, "guard"), ["set_score"]);
}

#[test]
fn edge_routes_reject_non_finite_and_overflowing_measurements() {
    let source = asset();
    let before = source.clone();
    let adapter = Adapter::new(&source, 10);
    let edge = GraphEdgeView::new(
        GraphPortRef::output("root".to_owned(), "children"),
        GraphPortRef::input("set_score".to_owned(), "parent"),
    );
    for (start, end) in [
        (GraphPoint::new(f32::NAN, 0.0), GraphPoint::new(10.0, 20.0)),
        (
            GraphPoint::new(0.0, 0.0),
            GraphPoint::new(f32::INFINITY, 20.0),
        ),
        (
            GraphPoint::new(f32::MAX, f32::MAX),
            GraphPoint::new(-f32::MAX, -f32::MAX),
        ),
    ] {
        assert!(matches!(
            adapter.route_edge(EdgeRoute {
                tree_id: &source.descriptor().id,
                revision: 10,
                edge: &edge,
                source_anchor: start,
                target_anchor: end,
                style: GraphRouteStyle::Bezier,
            }),
            Err(GraphError::InvalidRouteGeometry)
        ));
    }
    assert_eq!(source, before);
}
