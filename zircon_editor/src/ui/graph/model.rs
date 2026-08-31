use std::collections::{BTreeMap, BTreeSet};

use crate::core::editor_authoring_extension::GraphNodePaletteDescriptor;

/// A point in graph-local coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GraphPoint {
    pub x: f32,
    pub y: f32,
}

impl GraphPoint {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Axis-aligned node or marquee bounds in graph-local coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GraphNodeBounds {
    pub origin: GraphPoint,
    pub size: GraphPoint,
}

impl GraphNodeBounds {
    pub const fn new(origin: GraphPoint, size: GraphPoint) -> Self {
        Self { origin, size }
    }

    pub fn from_corners(first: GraphPoint, second: GraphPoint) -> Self {
        let left = first.x.min(second.x);
        let top = first.y.min(second.y);
        Self::new(
            GraphPoint::new(left, top),
            GraphPoint::new((first.x - second.x).abs(), (first.y - second.y).abs()),
        )
    }

    pub fn right(self) -> f32 {
        self.origin.x + self.size.x
    }

    pub fn bottom(self) -> f32 {
        self.origin.y + self.size.y
    }

    pub fn intersects(self, other: Self) -> bool {
        self.origin.x <= other.right()
            && self.right() >= other.origin.x
            && self.origin.y <= other.bottom()
            && self.bottom() >= other.origin.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GraphPinDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPinView {
    pub name: String,
    pub value_type: String,
    pub required: bool,
    pub direction: GraphPinDirection,
}

impl GraphPinView {
    pub fn input(name: impl Into<String>, value_type: impl Into<String>, required: bool) -> Self {
        Self {
            name: name.into(),
            value_type: value_type.into(),
            required,
            direction: GraphPinDirection::Input,
        }
    }

    pub fn output(name: impl Into<String>, value_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value_type: value_type.into(),
            required: false,
            direction: GraphPinDirection::Output,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphAttachmentView {
    pub id: String,
    pub display_name: String,
    pub category: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeView<NodeId> {
    pub id: NodeId,
    pub display_name: String,
    pub bounds: GraphNodeBounds,
    pub inputs: Vec<GraphPinView>,
    pub outputs: Vec<GraphPinView>,
    /// Domain-owned items displayed inside a node, such as BT decorators and services.
    pub attachments: Vec<GraphAttachmentView>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphPortRef<NodeId> {
    pub node_id: NodeId,
    pub direction: GraphPinDirection,
    pub pin_name: String,
}

impl<NodeId> GraphPortRef<NodeId> {
    pub fn input(node_id: NodeId, pin_name: impl Into<String>) -> Self {
        Self {
            node_id,
            direction: GraphPinDirection::Input,
            pin_name: pin_name.into(),
        }
    }

    pub fn output(node_id: NodeId, pin_name: impl Into<String>) -> Self {
        Self {
            node_id,
            direction: GraphPinDirection::Output,
            pin_name: pin_name.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphEdgeView<NodeId> {
    pub from: GraphPortRef<NodeId>,
    pub to: GraphPortRef<NodeId>,
}

impl<NodeId> GraphEdgeView<NodeId> {
    pub fn new(from: GraphPortRef<NodeId>, to: GraphPortRef<NodeId>) -> Self {
        Self { from, to }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureConstraint {
    FreeGraph,
    Tree,
    Dag,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphConnectRejection {
    SameNode,
    InvalidPinDirection,
    MissingNode,
    MissingPin,
    TypeMismatch { from: String, to: String },
    DuplicateEdge,
    TreeInputAlreadyConnected,
    Cycle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectVerdict {
    Allowed,
    Rejected(GraphConnectRejection),
}

impl ConnectVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    pub fn is_rejected(&self) -> bool {
        !self.is_allowed()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphDiagnostic<NodeId> {
    pub node_id: NodeId,
    pub port: GraphPortRef<NodeId>,
    pub message: &'static str,
}

/// Describes whether a domain graph mutation took effect. Errors must leave the domain model
/// unchanged; a mutation that took effect returns the inverse delta required by undo/redo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphMutationEffect<Delta> {
    Applied { inverse: Delta },
    Unchanged,
}

/// Domain-owned graph mutation protocol. The concrete delta stays with the asset authority so the
/// graph foundation never becomes a second animation, state-machine, or behavior-tree model.
pub trait GraphModel: Send {
    type NodeId: Clone + Eq + Ord;
    type Delta: Clone;
    type Error;

    fn nodes(&self) -> Vec<GraphNodeView<Self::NodeId>>;
    fn edges(&self) -> Vec<GraphEdgeView<Self::NodeId>>;
    fn palette(&self) -> &GraphNodePaletteDescriptor;
    fn can_connect(
        &self,
        from: GraphPortRef<Self::NodeId>,
        to: GraphPortRef<Self::NodeId>,
    ) -> ConnectVerdict;
    fn structure_constraint(&self) -> StructureConstraint;
    fn apply(
        &mut self,
        delta: Self::Delta,
    ) -> Result<GraphMutationEffect<Self::Delta>, Self::Error>;
}

/// Default schema behavior for domains that do not override connection compatibility.
///
/// This mirrors Unreal's division of responsibility: the canvas asks a domain schema for a
/// verdict, while this shared default only enforces pin direction, value-type compatibility, and
/// declared structure constraints.
pub fn default_connection_verdict<NodeId>(
    nodes: &[GraphNodeView<NodeId>],
    edges: &[GraphEdgeView<NodeId>],
    candidate: &GraphEdgeView<NodeId>,
    structure: StructureConstraint,
) -> ConnectVerdict
where
    NodeId: Clone + Eq + Ord,
{
    if candidate.from.node_id == candidate.to.node_id {
        return ConnectVerdict::Rejected(GraphConnectRejection::SameNode);
    }
    if candidate.from.direction != GraphPinDirection::Output
        || candidate.to.direction != GraphPinDirection::Input
    {
        return ConnectVerdict::Rejected(GraphConnectRejection::InvalidPinDirection);
    }

    let Some(from_node) = nodes.iter().find(|node| node.id == candidate.from.node_id) else {
        return ConnectVerdict::Rejected(GraphConnectRejection::MissingNode);
    };
    let Some(to_node) = nodes.iter().find(|node| node.id == candidate.to.node_id) else {
        return ConnectVerdict::Rejected(GraphConnectRejection::MissingNode);
    };
    let Some(from_pin) = from_node
        .outputs
        .iter()
        .find(|pin| pin.name == candidate.from.pin_name)
    else {
        return ConnectVerdict::Rejected(GraphConnectRejection::MissingPin);
    };
    let Some(to_pin) = to_node
        .inputs
        .iter()
        .find(|pin| pin.name == candidate.to.pin_name)
    else {
        return ConnectVerdict::Rejected(GraphConnectRejection::MissingPin);
    };
    if from_pin.direction != GraphPinDirection::Output
        || to_pin.direction != GraphPinDirection::Input
    {
        return ConnectVerdict::Rejected(GraphConnectRejection::InvalidPinDirection);
    }
    if from_pin.value_type != to_pin.value_type {
        return ConnectVerdict::Rejected(GraphConnectRejection::TypeMismatch {
            from: from_pin.value_type.clone(),
            to: to_pin.value_type.clone(),
        });
    }
    if edges.iter().any(|edge| edge == candidate) {
        return ConnectVerdict::Rejected(GraphConnectRejection::DuplicateEdge);
    }
    if structure == StructureConstraint::Tree && edges.iter().any(|edge| edge.to == candidate.to) {
        return ConnectVerdict::Rejected(GraphConnectRejection::TreeInputAlreadyConnected);
    }
    if structure != StructureConstraint::FreeGraph && introduces_cycle(edges, candidate) {
        return ConnectVerdict::Rejected(GraphConnectRejection::Cycle);
    }
    ConnectVerdict::Allowed
}

pub fn required_input_diagnostics<NodeId>(
    nodes: &[GraphNodeView<NodeId>],
    edges: &[GraphEdgeView<NodeId>],
) -> Vec<GraphDiagnostic<NodeId>>
where
    NodeId: Clone + Eq + Ord,
{
    let mut diagnostics = Vec::new();
    for node in nodes {
        for pin in node.inputs.iter().filter(|pin| pin.required) {
            let port = GraphPortRef::input(node.id.clone(), pin.name.clone());
            if !edges.iter().any(|edge| edge.to == port) {
                diagnostics.push(GraphDiagnostic {
                    node_id: node.id.clone(),
                    port,
                    message: "required input is not connected",
                });
            }
        }
    }
    diagnostics
}

fn introduces_cycle<NodeId>(
    edges: &[GraphEdgeView<NodeId>],
    candidate: &GraphEdgeView<NodeId>,
) -> bool
where
    NodeId: Clone + Eq + Ord,
{
    let mut adjacency = BTreeMap::<NodeId, Vec<NodeId>>::new();
    for edge in edges {
        adjacency
            .entry(edge.from.node_id.clone())
            .or_default()
            .push(edge.to.node_id.clone());
    }

    let target = candidate.from.node_id.clone();
    let mut pending = vec![candidate.to.node_id.clone()];
    let mut visited = BTreeSet::new();
    while let Some(node_id) = pending.pop() {
        if node_id == target {
            return true;
        }
        if !visited.insert(node_id.clone()) {
            continue;
        }
        if let Some(next) = adjacency.get(&node_id) {
            pending.extend(next.iter().cloned());
        }
    }
    false
}
