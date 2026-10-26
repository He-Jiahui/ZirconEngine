use std::{collections::BTreeMap, fmt};

use zircon_editor::ui::graph::{GraphEdgeView, GraphNodeBounds, GraphNodeView};
use zircon_plugin_rendering_vfx_graph_runtime::{VfxGraphAsset, VfxGraphNode};

/// Measured graph-local geometry for one immutable source and its actual host revision.
/// The host owns measurement/currentness; this does not generate a revision or layout store.
#[derive(Debug)]
pub struct VfxOverviewLayout<'a, Revision: ?Sized> {
    source: &'a VfxGraphAsset,
    revision: &'a Revision,
    bounds: &'a BTreeMap<usize, GraphNodeBounds>,
}

impl<'a, Revision: ?Sized> VfxOverviewLayout<'a, Revision> {
    pub fn from_measured_bounds(
        source: &'a VfxGraphAsset,
        revision: &'a Revision,
        bounds: &'a BTreeMap<usize, GraphNodeBounds>,
    ) -> Self {
        Self {
            source,
            revision,
            bounds,
        }
    }
}

/// An index locator retained only with its borrowed source/revision, never an authoring key.
#[derive(Debug)]
pub struct VfxOverviewNodeLocator<'a, Revision: ?Sized> {
    source: &'a VfxGraphAsset,
    revision: &'a Revision,
    node_index: usize,
}

impl<Revision: ?Sized> VfxOverviewNodeLocator<'_, Revision> {
    pub fn node_index(&self) -> usize {
        self.node_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfxOverviewConnectionEligibility {
    UnavailableNoSourcePortsOrEdges,
}

impl VfxOverviewConnectionEligibility {
    pub const fn reason(self) -> &'static str {
        "此资源未定义端口或连线"
    }
}

/// Shared presenter data backed by the real typed source, not a GraphModel or compile receipt.
/// Raw node IDs are indices local to this source/revision. Hosts must retire old input/selection
/// on a context change and use a checked locator when resolving a retained index.
#[derive(Debug)]
pub struct VfxOverviewProjection<'a, Revision: ?Sized> {
    source: &'a VfxGraphAsset,
    revision: &'a Revision,
    nodes: Vec<GraphNodeView<usize>>,
    edges: Vec<GraphEdgeView<usize>>,
}

impl<'a, Revision: Eq + ?Sized> VfxOverviewProjection<'a, Revision> {
    pub fn source(&self) -> &'a VfxGraphAsset {
        self.source
    }

    pub fn revision(&self) -> &'a Revision {
        self.revision
    }

    pub fn nodes(&self) -> &[GraphNodeView<usize>] {
        &self.nodes
    }

    pub fn edges(&self) -> &[GraphEdgeView<usize>] {
        &self.edges
    }

    pub fn connection_eligibility(&self) -> VfxOverviewConnectionEligibility {
        // This source declares no ports; an empty edge list alone is not a graph-wide veto.
        VfxOverviewConnectionEligibility::UnavailableNoSourcePortsOrEdges
    }

    pub fn locate(
        &self,
        node_index: usize,
    ) -> Result<VfxOverviewNodeLocator<'a, Revision>, VfxOverviewError> {
        self.source
            .nodes
            .get(node_index)
            .ok_or(VfxOverviewError::UnknownNode { node_index })?;
        Ok(VfxOverviewNodeLocator {
            source: self.source,
            revision: self.revision,
            node_index,
        })
    }

    /// Return the original payload without converting values, sanitizing it or compiling it.
    pub fn resolve(
        &self,
        locator: &VfxOverviewNodeLocator<'_, Revision>,
    ) -> Result<&'a VfxGraphNode, VfxOverviewError> {
        check_context(self.source, self.revision, locator.source, locator.revision)?;
        self.source
            .nodes
            .get(locator.node_index)
            .ok_or(VfxOverviewError::UnknownNode {
                node_index: locator.node_index,
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfxOverviewError {
    StaleSource,
    StaleRevision,
    MissingLayout { node_index: usize },
    UnexpectedLayoutNode { node_index: usize },
    InvalidBounds { node_index: usize },
    UnknownNode { node_index: usize },
}

impl fmt::Display for VfxOverviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleSource => write!(f, "VFX overview belongs to another source"),
            Self::StaleRevision => write!(f, "VFX overview belongs to another source revision"),
            Self::MissingLayout { node_index } => {
                write!(f, "missing measured VFX bounds for node {node_index}")
            }
            Self::UnexpectedLayoutNode { node_index } => {
                write!(f, "VFX layout references absent source node {node_index}")
            }
            Self::InvalidBounds { node_index } => {
                write!(f, "invalid measured VFX bounds for node {node_index}")
            }
            Self::UnknownNode { node_index } => write!(f, "unknown VFX source node {node_index}"),
        }
    }
}

impl std::error::Error for VfxOverviewError {}

/// Preserve empty sources, duplicate kinds, order and typed values. Parameter diagnostics and
/// lowering stay with the real compiler; successful projection says nothing about frame readiness.
pub fn project_vfx_overview<'a, Revision: Eq + ?Sized>(
    source: &'a VfxGraphAsset,
    revision: &'a Revision,
    layout: &VfxOverviewLayout<'_, Revision>,
) -> Result<VfxOverviewProjection<'a, Revision>, VfxOverviewError> {
    check_context(source, revision, layout.source, layout.revision)?;
    if let Some(node_index) = layout
        .bounds
        .keys()
        .copied()
        .find(|&index| index >= source.nodes.len())
    {
        return Err(VfxOverviewError::UnexpectedLayoutNode { node_index });
    }
    let mut nodes = Vec::with_capacity(source.nodes.len());
    for (node_index, payload) in source.nodes.iter().enumerate() {
        let bounds = *layout
            .bounds
            .get(&node_index)
            .ok_or(VfxOverviewError::MissingLayout { node_index })?;
        validate_bounds(bounds, node_index)?;
        nodes.push(GraphNodeView {
            id: node_index,
            display_name: caption(payload).to_owned(),
            bounds,
            inputs: Vec::new(),
            outputs: Vec::new(),
            attachments: Vec::new(),
        });
    }
    Ok(VfxOverviewProjection {
        source,
        revision,
        nodes,
        edges: Vec::new(),
    })
}

fn check_context<Revision: Eq + ?Sized>(
    source: &VfxGraphAsset,
    revision: &Revision,
    original_source: &VfxGraphAsset,
    original_revision: &Revision,
) -> Result<(), VfxOverviewError> {
    if !std::ptr::eq(source, original_source) {
        return Err(VfxOverviewError::StaleSource);
    }
    if revision != original_revision {
        return Err(VfxOverviewError::StaleRevision);
    }
    Ok(())
}

fn validate_bounds(bounds: GraphNodeBounds, node_index: usize) -> Result<(), VfxOverviewError> {
    if !bounds.origin.x.is_finite()
        || !bounds.origin.y.is_finite()
        || !bounds.size.x.is_finite()
        || !bounds.size.y.is_finite()
        || bounds.size.x <= 0.0
        || bounds.size.y <= 0.0
        || !bounds.right().is_finite()
        || !bounds.bottom().is_finite()
        || bounds.right() <= bounds.origin.x
        || bounds.bottom() <= bounds.origin.y
    {
        return Err(VfxOverviewError::InvalidBounds { node_index });
    }
    Ok(())
}

fn caption(node: &VfxGraphNode) -> &'static str {
    match node {
        VfxGraphNode::SpawnRate { .. } => "SpawnRate",
        VfxGraphNode::Lifetime { .. } => "Lifetime",
        VfxGraphNode::Velocity { .. } => "Velocity",
        VfxGraphNode::ColorOverLife { .. } => "ColorOverLife",
        VfxGraphNode::ShaderGraphMaterial { .. } => "ShaderGraphMaterial",
    }
}
