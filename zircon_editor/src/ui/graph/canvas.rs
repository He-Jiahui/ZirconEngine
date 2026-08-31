use std::collections::{BTreeMap, BTreeSet};

use super::{GraphNodeBounds, GraphNodeView, GraphPoint};

pub const GRAPH_MIN_ZOOM: f32 = 0.2;
pub const GRAPH_MAX_ZOOM: f32 = 4.0;
const ZOOM_STEP: f32 = 0.1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphSelection<NodeId> {
    node_ids: BTreeSet<NodeId>,
}

impl<NodeId> Default for GraphSelection<NodeId> {
    fn default() -> Self {
        Self {
            node_ids: BTreeSet::new(),
        }
    }
}

impl<NodeId> GraphSelection<NodeId>
where
    NodeId: Ord,
{
    pub fn node_ids(&self) -> &BTreeSet<NodeId> {
        &self.node_ids
    }

    pub fn contains(&self, node_id: &NodeId) -> bool {
        self.node_ids.contains(node_id)
    }

    pub fn replace<I>(&mut self, node_ids: I) -> bool
    where
        I: IntoIterator<Item = NodeId>,
    {
        let next = node_ids.into_iter().collect::<BTreeSet<_>>();
        if self.node_ids == next {
            return false;
        }
        self.node_ids = next;
        true
    }
}

/// A domain-applicable node position change produced by a completed canvas drag.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeMove<NodeId> {
    pub node_id: NodeId,
    pub position: GraphPoint,
}

/// Immutable gesture state. The canvas never writes positions into a domain asset; a toolkit
/// converts the returned moves into its own reversible transaction.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeDrag<NodeId> {
    anchor: GraphPoint,
    initial_positions: BTreeMap<NodeId, GraphPoint>,
}

impl<NodeId> GraphNodeDrag<NodeId>
where
    NodeId: Clone + Ord,
{
    pub fn moved_nodes(&self, graph_pointer: GraphPoint) -> Vec<GraphNodeMove<NodeId>> {
        let delta = GraphPoint::new(
            graph_pointer.x - self.anchor.x,
            graph_pointer.y - self.anchor.y,
        );
        self.initial_positions
            .iter()
            .map(|(node_id, position)| GraphNodeMove {
                node_id: node_id.clone(),
                position: GraphPoint::new(position.x + delta.x, position.y + delta.y),
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasState<NodeId = String> {
    pan: GraphPoint,
    zoom: f32,
    selection: GraphSelection<NodeId>,
}

impl<NodeId> Default for GraphCanvasState<NodeId> {
    fn default() -> Self {
        Self {
            pan: GraphPoint::ZERO,
            zoom: 1.0,
            selection: GraphSelection::default(),
        }
    }
}

impl<NodeId> GraphCanvasState<NodeId>
where
    NodeId: Clone + Ord,
{
    pub fn pan(&self) -> GraphPoint {
        self.pan
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn selection(&self) -> &GraphSelection<NodeId> {
        &self.selection
    }

    pub fn replace_selection<I>(&mut self, node_ids: I) -> bool
    where
        I: IntoIterator<Item = NodeId>,
    {
        self.selection.replace(node_ids)
    }

    pub fn graph_to_screen(&self, point: GraphPoint) -> GraphPoint {
        GraphPoint::new(
            point.x.mul_add(self.zoom, self.pan.x),
            point.y.mul_add(self.zoom, self.pan.y),
        )
    }

    pub fn screen_to_graph(&self, point: GraphPoint) -> GraphPoint {
        GraphPoint::new(
            (point.x - self.pan.x) / self.zoom,
            (point.y - self.pan.y) / self.zoom,
        )
    }

    pub fn pan_by(&mut self, screen_delta: GraphPoint) {
        self.pan.x += screen_delta.x;
        self.pan.y += screen_delta.y;
    }

    /// Adjusts zoom around a screen-space anchor without letting the graph coordinate under the
    /// pointer drift. This is the same interaction invariant used by mature graph canvases.
    pub fn zoom_about_screen_point(&mut self, screen_anchor: GraphPoint, wheel_steps: f32) -> bool {
        let graph_anchor = self.screen_to_graph(screen_anchor);
        let next_zoom = (self.zoom + wheel_steps * ZOOM_STEP).clamp(GRAPH_MIN_ZOOM, GRAPH_MAX_ZOOM);
        if next_zoom == self.zoom {
            return false;
        }
        self.zoom = next_zoom;
        self.pan = GraphPoint::new(
            screen_anchor.x - graph_anchor.x * self.zoom,
            screen_anchor.y - graph_anchor.y * self.zoom,
        );
        true
    }

    pub fn replace_selection_from_marquee(
        &mut self,
        nodes: &[GraphNodeView<NodeId>],
        marquee: GraphNodeBounds,
    ) -> BTreeSet<NodeId> {
        let selected = nodes
            .iter()
            .filter(|node| node.bounds.intersects(marquee))
            .map(|node| node.id.clone())
            .collect::<BTreeSet<_>>();
        self.selection.replace(selected);
        self.selection.node_ids.clone()
    }

    pub fn begin_node_drag(
        &self,
        nodes: &[GraphNodeView<NodeId>],
        graph_pointer: GraphPoint,
    ) -> Option<GraphNodeDrag<NodeId>> {
        let initial_positions = nodes
            .iter()
            .filter(|node| self.selection.contains(&node.id))
            .map(|node| (node.id.clone(), node.bounds.origin))
            .collect::<BTreeMap<_, _>>();
        (!initial_positions.is_empty()).then_some(GraphNodeDrag {
            anchor: graph_pointer,
            initial_positions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::GraphCanvasState;

    #[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    struct NodeIdWithoutDefault(u64);

    #[test]
    fn default_canvas_does_not_require_default_node_ids() {
        let canvas = GraphCanvasState::<NodeIdWithoutDefault>::default();

        assert!(canvas.selection().node_ids().is_empty());
    }
}
