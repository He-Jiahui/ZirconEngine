use std::collections::BTreeMap;

use thiserror::Error;
use zircon_editor::ui::graph::{
    route_connection_with_directions, GraphConnectRejection, GraphConnectionRoute, GraphEdgeView,
    GraphNodeBounds, GraphNodeView, GraphPinDirection, GraphPinView, GraphPoint, GraphPortRef,
    GraphRouteDirection, GraphRouteStyle,
};
use zircon_runtime::core::framework::ai::{AiBehaviorNodeDescriptor, AiBehaviorNodeKind};

use super::{AiBehaviorTreeAsset, AiBehaviorTreeEdit, AiBehaviorTreeEditError};

const PARENT: &str = "parent";
const FLOW: &str = "ai.behavior.control";

/// Geometry from the owning retained surface, already in graph-local logical units.
/// The adapter neither invents node placement nor applies pan, zoom or DPI again.
pub struct AiBehaviorTreeGraphGeometry<'a> {
    pub tree_id: &'a str,
    pub revision: u64,
    pub node_bounds: &'a BTreeMap<String, GraphNodeBounds>,
}

/// A disposable shared-surface projection, not another mutable behavior-tree model.
pub struct AiBehaviorTreeGraphProjection {
    pub tree_id: String,
    pub revision: u64,
    pub nodes: Vec<GraphNodeView<String>>,
    /// Edges for each parent retain its actual child order; pins are stable across reorders.
    pub edges: Vec<GraphEdgeView<String>>,
}

pub struct AiBehaviorTreeConnectionRequest<'a> {
    pub tree_id: &'a str,
    pub revision: u64,
    pub from: &'a GraphPortRef<String>,
    pub to: &'a GraphPortRef<String>,
    /// Destination index after removing this child's existing incoming edge.
    pub index_after_removal: usize,
}

/// Anchors from the same retained measurement in graph-local logical units. The host must
/// reject obsolete measurement/transform generations; document revision alone cannot do that.
pub struct AiBehaviorTreeEdgeRouteRequest<'a> {
    pub tree_id: &'a str,
    pub revision: u64,
    pub edge: &'a GraphEdgeView<String>,
    pub source_anchor: GraphPoint,
    pub target_anchor: GraphPoint,
    pub style: GraphRouteStyle,
}

/// One whole-tree candidate. This value has no budget permit, undo or publication authority.
pub struct AiBehaviorTreeConnectionCandidate {
    tree_id: String,
    revision: u64,
    edit: AiBehaviorTreeEdit,
    candidate: Option<AiBehaviorTreeAsset>,
}

impl AiBehaviorTreeConnectionCandidate {
    pub fn tree_id(&self) -> &str {
        &self.tree_id
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn edit(&self) -> &AiBehaviorTreeEdit {
        &self.edit
    }

    pub fn candidate(&self) -> Option<&AiBehaviorTreeAsset> {
        self.candidate.as_ref()
    }

    /// The document must still check its live revision and publish through its transaction.
    pub fn into_candidate(self) -> Option<AiBehaviorTreeAsset> {
        self.candidate
    }
}

#[derive(Debug, Error)]
pub enum AiBehaviorTreeGraphError {
    #[error("graph belongs to tree `{actual}`, expected `{expected}`")]
    TreeIdentity { expected: String, actual: String },
    #[error("graph revision {actual} differs from source revision {expected}")]
    Revision { expected: u64, actual: u64 },
    #[error("geometry has {actual} nodes, expected {expected}")]
    GeometryCount { expected: usize, actual: usize },
    #[error("real graph-local geometry is missing for `{node_id}`")]
    MissingGeometry { node_id: String },
    #[error("graph-local geometry is non-finite, overflowing or non-positive for `{node_id}`")]
    InvalidGeometry { node_id: String },
    #[error("the requested endpoints are not an edge of the current behavior tree")]
    MissingEdge,
    #[error("measured pin anchors cannot produce finite directed geometry")]
    InvalidRouteGeometry,
    #[error("behavior-tree connection rejected: {0:?}")]
    Connection(GraphConnectRejection),
    #[error(transparent)]
    Candidate(#[from] AiBehaviorTreeEditError),
}

/// Borrows the actual immutable source and the revision supplied by its document authority.
/// It does not implement GraphModel's retained-owner protocol: that integration requires
/// a proven whole-candidate bound and the host's one permit before materialization.
pub struct AiBehaviorTreeGraphAdapter<'a> {
    asset: &'a AiBehaviorTreeAsset,
    revision: u64,
}

impl<'a> AiBehaviorTreeGraphAdapter<'a> {
    pub fn new(asset: &'a AiBehaviorTreeAsset, revision: u64) -> Self {
        Self { asset, revision }
    }

    /// Allocates view values only after caller admission; never use as an unmetered paint cache.
    pub fn project(
        &self,
        geometry: AiBehaviorTreeGraphGeometry<'_>,
    ) -> Result<AiBehaviorTreeGraphProjection, AiBehaviorTreeGraphError> {
        self.check_scope(geometry.tree_id, geometry.revision)?;
        let tree = self.asset.descriptor();
        if geometry.node_bounds.len() != tree.nodes.len() {
            return Err(AiBehaviorTreeGraphError::GeometryCount {
                expected: tree.nodes.len(),
                actual: geometry.node_bounds.len(),
            });
        }
        // Complete geometry is validated before constructing any partial projection.
        for node in &tree.nodes {
            let bounds = geometry.node_bounds.get(&node.id).ok_or_else(|| {
                AiBehaviorTreeGraphError::MissingGeometry {
                    node_id: node.id.clone(),
                }
            })?;
            if !valid_bounds(*bounds) {
                return Err(AiBehaviorTreeGraphError::InvalidGeometry {
                    node_id: node.id.clone(),
                });
            }
        }
        let mut nodes = Vec::with_capacity(tree.nodes.len());
        let mut edges = Vec::with_capacity(tree.nodes.len().saturating_sub(1));
        for node in &tree.nodes {
            let inputs = if node.id == tree.root_node {
                Vec::new()
            } else {
                vec![GraphPinView::input(PARENT, FLOW, true)]
            };
            let outputs = child_pin(node.kind)
                .map(|pin| vec![GraphPinView::output(pin, FLOW)])
                .unwrap_or_default();
            nodes.push(GraphNodeView {
                id: node.id.clone(),
                display_name: node.display_name.clone(),
                bounds: geometry.node_bounds[&node.id],
                inputs,
                outputs,
                // Decorators remain real wrapper nodes; no fabricated attachment identities.
                attachments: Vec::new(),
            });
            for child in &node.children {
                let pin = child_pin(node.kind).ok_or(AiBehaviorTreeGraphError::Connection(
                    GraphConnectRejection::MissingPin,
                ))?;
                edges.push(GraphEdgeView::new(
                    GraphPortRef::output(node.id.clone(), pin),
                    GraphPortRef::input(child.clone(), PARENT),
                ));
            }
        }
        Ok(AiBehaviorTreeGraphProjection {
            tree_id: tree.id.clone(),
            revision: self.revision,
            nodes,
            edges,
        })
    }

    /// Routes an existing authored edge only, after host allocation admission. It does not
    /// invent pin positions, preview a mutation, apply a camera/DPI transform or imply activity.
    pub fn route_edge(
        &self,
        request: AiBehaviorTreeEdgeRouteRequest<'_>,
    ) -> Result<GraphConnectionRoute, AiBehaviorTreeGraphError> {
        self.check_scope(request.tree_id, request.revision)?;
        let (parent, child) = self.check_ports(&request.edge.from, &request.edge.to)?;
        if !parent.children.contains(&child.id) {
            return Err(AiBehaviorTreeGraphError::MissingEdge);
        }
        route_connection_with_directions(
            request.source_anchor,
            request.target_anchor,
            GraphRouteDirection::Down,
            GraphRouteDirection::Up,
            request.style,
        )
        .ok_or(AiBehaviorTreeGraphError::InvalidRouteGeometry)
    }

    /// Materializes only after caller admission. Do not call from an unmetered hover probe.
    /// Replace removes the old parent only inside one Reparent candidate and validates the
    /// entire resulting tree through the existing TOML/manager/catalog compiler sequence.
    pub fn candidate_for_connection(
        &self,
        request: AiBehaviorTreeConnectionRequest<'_>,
    ) -> Result<AiBehaviorTreeConnectionCandidate, AiBehaviorTreeGraphError> {
        self.check_scope(request.tree_id, request.revision)?;
        let (parent, child) = self.check_ports(request.from, request.to)?;
        let edit = AiBehaviorTreeEdit::Reparent {
            node_id: child.id.clone(),
            parent_id: parent.id.clone(),
            index: request.index_after_removal,
        };
        let candidate = self.asset.edited(&edit)?;
        Ok(AiBehaviorTreeConnectionCandidate {
            tree_id: self.asset.descriptor().id.clone(),
            revision: self.revision,
            edit,
            candidate,
        })
    }

    fn check_ports(
        &self,
        from: &GraphPortRef<String>,
        to: &GraphPortRef<String>,
    ) -> Result<(&AiBehaviorNodeDescriptor, &AiBehaviorNodeDescriptor), AiBehaviorTreeGraphError>
    {
        if from.direction != GraphPinDirection::Output || to.direction != GraphPinDirection::Input {
            return Err(AiBehaviorTreeGraphError::Connection(
                GraphConnectRejection::InvalidPinDirection,
            ));
        }
        if from.node_id == to.node_id {
            return Err(AiBehaviorTreeGraphError::Connection(
                GraphConnectRejection::SameNode,
            ));
        }
        let tree = self.asset.descriptor();
        let parent = tree
            .nodes
            .iter()
            .find(|node| node.id == from.node_id)
            .ok_or(AiBehaviorTreeGraphError::Connection(
                GraphConnectRejection::MissingNode,
            ))?;
        let child = tree.nodes.iter().find(|node| node.id == to.node_id).ok_or(
            AiBehaviorTreeGraphError::Connection(GraphConnectRejection::MissingNode),
        )?;
        if child_pin(parent.kind) != Some(from.pin_name.as_str())
            || to.pin_name != PARENT
            || child.id == tree.root_node
        {
            return Err(AiBehaviorTreeGraphError::Connection(
                GraphConnectRejection::MissingPin,
            ));
        }
        Ok((parent, child))
    }

    fn check_scope(&self, tree_id: &str, revision: u64) -> Result<(), AiBehaviorTreeGraphError> {
        if tree_id != self.asset.descriptor().id {
            return Err(AiBehaviorTreeGraphError::TreeIdentity {
                expected: self.asset.descriptor().id.clone(),
                actual: tree_id.to_owned(),
            });
        }
        if revision != self.revision {
            return Err(AiBehaviorTreeGraphError::Revision {
                expected: self.revision,
                actual: revision,
            });
        }
        Ok(())
    }
}

fn child_pin(kind: AiBehaviorNodeKind) -> Option<&'static str> {
    match kind {
        AiBehaviorNodeKind::Selector
        | AiBehaviorNodeKind::Sequence
        | AiBehaviorNodeKind::Parallel => Some("children"),
        AiBehaviorNodeKind::Decorator => Some("child"),
        AiBehaviorNodeKind::Task | AiBehaviorNodeKind::Service | AiBehaviorNodeKind::Subtree => {
            None
        }
    }
}

fn valid_bounds(bounds: GraphNodeBounds) -> bool {
    bounds.origin.x.is_finite()
        && bounds.origin.y.is_finite()
        && bounds.size.x.is_finite()
        && bounds.size.y.is_finite()
        && bounds.size.x > 0.0
        && bounds.size.y > 0.0
        && bounds.right().is_finite()
        && bounds.bottom().is_finite()
}
