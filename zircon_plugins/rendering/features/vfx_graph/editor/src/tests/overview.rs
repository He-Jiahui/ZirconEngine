use std::collections::BTreeMap;

use zircon_editor::ui::graph::{
    default_connection_verdict, required_input_diagnostics, ConnectVerdict, GraphConnectRejection,
    GraphEdgeView, GraphNodeBounds, GraphNodePresentation, GraphNodeView, GraphPinView, GraphPoint,
    GraphPortRef, GraphSelection, StructureConstraint,
};
use zircon_plugin_rendering_vfx_graph_runtime::{VfxGraphAsset, VfxGraphNode};

use super::{
    project_vfx_overview, VfxOverviewConnectionEligibility, VfxOverviewError, VfxOverviewLayout,
};

fn five_kind_source() -> VfxGraphAsset {
    VfxGraphAsset {
        name: "火焰／a long VFX graph name".to_owned(),
        max_particles: 4096,
        nodes: vec![
            VfxGraphNode::SpawnRate {
                particles_per_second: 32.5,
            },
            VfxGraphNode::Lifetime { seconds: 1.25 },
            VfxGraphNode::Velocity {
                value: [-2.0, 1.5, 0.25],
            },
            VfxGraphNode::ColorOverLife {
                start: [4.0, 0.5, -0.25, 1.0],
                end: [0.1, 2.0, 0.0, 0.0],
            },
            VfxGraphNode::ShaderGraphMaterial {
                shader_graph: "res://效果/火焰.shadergraph".to_owned(),
            },
        ],
    }
}

// Explicit regression geometry only; production measurements must come from the real host.
fn measured_bounds(source: &VfxGraphAsset) -> BTreeMap<usize, GraphNodeBounds> {
    (0..source.nodes.len())
        .map(|index| {
            (
                index,
                GraphNodeBounds::new(
                    GraphPoint::new(index as f32 * 280.0 - 120.0, -30.0),
                    GraphPoint::new(240.0, 90.0),
                ),
            )
        })
        .collect()
}

#[test]
fn vfx_overview_preserves_five_real_payloads_and_shared_presenter_contract() {
    let source = five_kind_source();
    let original = source.clone();
    let revision = 7_u64;
    let bounds = measured_bounds(&source);
    let layout = VfxOverviewLayout::from_measured_bounds(&source, &revision, &bounds);
    let graph = project_vfx_overview(&source, &revision, &layout).unwrap();
    assert!(std::ptr::eq(graph.source(), &source));
    assert!(std::ptr::eq(graph.revision(), &revision));
    assert_eq!(graph.source().name, original.name);
    assert_eq!(graph.source().max_particles, 4096);
    assert_eq!(graph.nodes().len(), 5);
    assert_eq!(
        graph
            .nodes()
            .iter()
            .map(|node| node.display_name.as_str())
            .collect::<Vec<_>>(),
        [
            "SpawnRate",
            "Lifetime",
            "Velocity",
            "ColorOverLife",
            "ShaderGraphMaterial"
        ]
    );
    for (index, node) in graph.nodes().iter().enumerate() {
        let locator = graph.locate(node.id).unwrap();
        assert_eq!(locator.node_index(), index);
        assert!(std::ptr::eq(
            graph.resolve(&locator).unwrap(),
            &source.nodes[index]
        ));
        assert_eq!(graph.resolve(&locator).unwrap(), &original.nodes[index]);
        assert_eq!(node.bounds, bounds[&index]);
        let presenter = GraphNodePresentation::from_view(node, &GraphSelection::default());
        assert_eq!((presenter.input_count, presenter.output_count), (0, 0));
        assert!(presenter.attachment_labels.is_empty());
    }
    assert!(graph.edges().is_empty());
    assert!(required_input_diagnostics(graph.nodes(), graph.edges()).is_empty());
    assert_eq!(
        graph.connection_eligibility(),
        VfxOverviewConnectionEligibility::UnavailableNoSourcePortsOrEdges
    );
    assert!(!graph.connection_eligibility().reason().is_empty());
    let forged = GraphEdgeView::new(
        GraphPortRef::output(0, "absent"),
        GraphPortRef::input(1, "absent"),
    );
    assert_eq!(
        default_connection_verdict(
            graph.nodes(),
            graph.edges(),
            &forged,
            StructureConstraint::FreeGraph,
        ),
        ConnectVerdict::Rejected(GraphConnectRejection::MissingPin)
    );
    // Empty VFX ports do not disable real required-input diagnostics in another graph.
    let required = GraphNodeView {
        id: 0_usize,
        display_name: "required input control fixture".to_owned(),
        bounds: bounds[&0],
        inputs: vec![GraphPinView::input("actual-required", "float", true)],
        outputs: Vec::new(),
        attachments: Vec::new(),
    };
    assert_eq!(
        required_input_diagnostics(std::slice::from_ref(&required), &[]).len(),
        1
    );
    let output = GraphNodeView {
        id: 1_usize,
        display_name: "legitimate output control fixture".to_owned(),
        bounds: bounds[&1],
        inputs: Vec::new(),
        outputs: vec![GraphPinView::output("actual-output", "float")],
        attachments: Vec::new(),
    };
    let candidate = GraphEdgeView::new(
        GraphPortRef::output(1, "actual-output"),
        GraphPortRef::input(0, "actual-required"),
    );
    assert_eq!(
        default_connection_verdict(
            &[output, required],
            &[],
            &candidate,
            StructureConstraint::FreeGraph
        ),
        ConnectVerdict::Allowed
    );
    assert_eq!(source, original);
}

#[test]
fn vfx_overview_preserves_empty_sources_repeated_kinds_and_order() {
    let empty = VfxGraphAsset {
        name: String::new(),
        max_particles: 0,
        nodes: Vec::new(),
    };
    let revision = 0_u64;
    let empty_bounds = BTreeMap::new();
    let layout = VfxOverviewLayout::from_measured_bounds(&empty, &revision, &empty_bounds);
    let graph = project_vfx_overview(&empty, &revision, &layout).unwrap();
    assert!(graph.nodes().is_empty());
    assert!(graph.edges().is_empty());
    assert!(matches!(
        graph.locate(0),
        Err(VfxOverviewError::UnknownNode { node_index: 0 })
    ));

    let mut repeated = five_kind_source();
    repeated.nodes.insert(
        1,
        VfxGraphNode::SpawnRate {
            particles_per_second: 64.25,
        },
    );
    repeated.nodes.push(VfxGraphNode::Lifetime { seconds: 3.5 });
    let original = repeated.clone();
    let bounds = measured_bounds(&repeated);
    let layout = VfxOverviewLayout::from_measured_bounds(&repeated, &revision, &bounds);
    let graph = project_vfx_overview(&repeated, &revision, &layout).unwrap();
    assert_eq!(graph.nodes().len(), 7);
    assert_eq!(graph.nodes()[0].display_name, graph.nodes()[1].display_name);
    for (index, expected) in original.nodes.iter().enumerate() {
        assert_eq!(graph.nodes()[index].id, index);
        assert_eq!(
            graph.resolve(&graph.locate(index).unwrap()).unwrap(),
            expected
        );
    }
    assert_eq!(repeated, original);
}

#[test]
fn vfx_overview_rejects_old_layout_and_locators_after_source_revision_or_order_changes() {
    let source = five_kind_source();
    let revision = 1_u64;
    let next_revision = 2_u64;
    let bounds = measured_bounds(&source);
    let old_layout = VfxOverviewLayout::from_measured_bounds(&source, &revision, &bounds);
    let old = project_vfx_overview(&source, &revision, &old_layout).unwrap();
    let locator = old.locate(0).unwrap();
    assert_eq!(
        project_vfx_overview(&source, &next_revision, &old_layout).unwrap_err(),
        VfxOverviewError::StaleRevision
    );
    let current_layout = VfxOverviewLayout::from_measured_bounds(&source, &next_revision, &bounds);
    let current = project_vfx_overview(&source, &next_revision, &current_layout).unwrap();
    assert_eq!(
        current.resolve(&locator),
        Err(VfxOverviewError::StaleRevision)
    );

    let equal_other_source = source.clone();
    assert_eq!(
        project_vfx_overview(&equal_other_source, &revision, &old_layout).unwrap_err(),
        VfxOverviewError::StaleSource
    );
    let mut reordered = source.clone();
    reordered.nodes.swap(0, 1);
    assert_eq!(
        project_vfx_overview(&reordered, &next_revision, &old_layout).unwrap_err(),
        VfxOverviewError::StaleSource
    );
    let reordered_bounds = measured_bounds(&reordered);
    let reordered_layout =
        VfxOverviewLayout::from_measured_bounds(&reordered, &next_revision, &reordered_bounds);
    let current = project_vfx_overview(&reordered, &next_revision, &reordered_layout).unwrap();
    assert_eq!(
        current.resolve(&locator),
        Err(VfxOverviewError::StaleSource)
    );
    let current_locator = current.locate(0).unwrap();
    assert_eq!(locator.node_index(), current_locator.node_index());
    assert!(
        matches!(current.resolve(&current_locator), Ok(VfxGraphNode::Lifetime { seconds }) if *seconds == 1.25)
    );
    assert!(matches!(
        current.locate(5),
        Err(VfxOverviewError::UnknownNode { node_index: 5 })
    ));
}

#[test]
fn vfx_overview_rejects_missing_extra_and_invalid_measured_bounds() {
    let source = five_kind_source();
    let revision = 1_u64;
    let bounds = measured_bounds(&source);
    let mut missing = bounds.clone();
    missing.remove(&4);
    let layout = VfxOverviewLayout::from_measured_bounds(&source, &revision, &missing);
    assert_eq!(
        project_vfx_overview(&source, &revision, &layout).unwrap_err(),
        VfxOverviewError::MissingLayout { node_index: 4 }
    );
    let mut extra = bounds.clone();
    extra.insert(5, bounds[&0]);
    let layout = VfxOverviewLayout::from_measured_bounds(&source, &revision, &extra);
    assert_eq!(
        project_vfx_overview(&source, &revision, &layout).unwrap_err(),
        VfxOverviewError::UnexpectedLayoutNode { node_index: 5 }
    );
    let empty = VfxGraphAsset {
        name: String::new(),
        max_particles: 0,
        nodes: Vec::new(),
    };
    let layout = VfxOverviewLayout::from_measured_bounds(&empty, &revision, &bounds);
    assert_eq!(
        project_vfx_overview(&empty, &revision, &layout).unwrap_err(),
        VfxOverviewError::UnexpectedLayoutNode { node_index: 0 }
    );

    let valid = bounds[&0];
    let mut invalid = Vec::new();
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        invalid.push(GraphNodeBounds::new(
            GraphPoint::new(value, 0.0),
            valid.size,
        ));
        invalid.push(GraphNodeBounds::new(
            GraphPoint::new(0.0, value),
            valid.size,
        ));
    }
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
        invalid.push(GraphNodeBounds::new(
            valid.origin,
            GraphPoint::new(value, 90.0),
        ));
        invalid.push(GraphNodeBounds::new(
            valid.origin,
            GraphPoint::new(240.0, value),
        ));
    }
    for value in [1.0, f32::MAX] {
        invalid.push(GraphNodeBounds::new(
            GraphPoint::new(f32::MAX, 0.0),
            GraphPoint::new(value, 90.0),
        ));
        invalid.push(GraphNodeBounds::new(
            GraphPoint::new(0.0, f32::MAX),
            GraphPoint::new(240.0, value),
        ));
    }
    for invalid_bounds in invalid {
        let mut measured = bounds.clone();
        measured.insert(0, invalid_bounds);
        let layout = VfxOverviewLayout::from_measured_bounds(&source, &revision, &measured);
        assert_eq!(
            project_vfx_overview(&source, &revision, &layout).unwrap_err(),
            VfxOverviewError::InvalidBounds { node_index: 0 }
        );
    }
    let layout = VfxOverviewLayout::from_measured_bounds(&source, &revision, &bounds);
    assert_eq!(
        project_vfx_overview(&source, &revision, &layout)
            .unwrap()
            .nodes()[0]
            .bounds,
        valid
    );
}

#[test]
fn vfx_overview_keeps_typed_invalid_values_visible_for_real_compiler_diagnostics() {
    let source = VfxGraphAsset {
        name: "diagnostic source".to_owned(),
        max_particles: 0,
        nodes: vec![
            VfxGraphNode::SpawnRate {
                particles_per_second: f32::NAN,
            },
            VfxGraphNode::Lifetime { seconds: -1.0 },
            VfxGraphNode::Velocity {
                value: [f32::INFINITY, f32::NEG_INFINITY, -0.0],
            },
            VfxGraphNode::ColorOverLife {
                start: [8.0, -2.0, f32::NAN, 1.0],
                end: [2.0, 0.5, 0.0, -1.0],
            },
            VfxGraphNode::ShaderGraphMaterial {
                shader_graph: String::new(),
            },
        ],
    };
    let revision = 1_u64;
    let bounds = measured_bounds(&source);
    let layout = VfxOverviewLayout::from_measured_bounds(&source, &revision, &bounds);
    let graph = project_vfx_overview(&source, &revision, &layout).unwrap();
    for index in 0..5 {
        assert!(std::ptr::eq(
            graph.resolve(&graph.locate(index).unwrap()).unwrap(),
            &source.nodes[index]
        ));
    }
    let VfxGraphNode::SpawnRate {
        particles_per_second,
    } = graph.resolve(&graph.locate(0).unwrap()).unwrap()
    else {
        panic!("expected actual spawn payload")
    };
    assert_eq!(particles_per_second.to_bits(), f32::NAN.to_bits());
    let VfxGraphNode::ColorOverLife { start, end } =
        graph.resolve(&graph.locate(3).unwrap()).unwrap()
    else {
        panic!("expected actual color payload")
    };
    assert_eq!(start[0], 8.0);
    assert_eq!(start[1], -2.0);
    assert!(start[2].is_nan());
    assert_eq!(end[3], -1.0);
    assert_eq!(graph.source().max_particles, 0);
}
