use std::collections::{BTreeMap, BTreeSet};

use super::{GraphNodeBounds, GraphNodeView, GraphPoint};

pub const GRAPH_MIN_ZOOM: f32 = 0.2;
pub const GRAPH_MAX_ZOOM: f32 = 4.0;
const ZOOM_STEP: f32 = 0.1;

/// Outcome of framing measured graph bounds. This changes only the existing view transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphBoundsFitResult {
    pub changed: bool,
    /// False when the zoom floor or coordinate precision prevents full containment.
    pub fully_visible: bool,
}

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

    /// Centers caller-measured graph bounds in a pane-local logical viewport in O(1).
    /// `padding` is a nonnegative inset in logical screen units on each side. The host must
    /// supply complete bounds and viewport geometry from the same document/layout/transform
    /// generation, resolve all/selection/source-focus intent, and apply DPI exactly once.
    /// This method does not choose nodes, change selection/history, or invalidate host geometry.
    ///
    /// Empty, invalid, or unrepresentable geometry returns `None` without changing state.
    /// Valid geometry uses the existing zoom limits; a fit constrained by the minimum zoom
    /// still centers the bounds and explicitly reports whether they are fully visible.
    pub fn fit_graph_bounds(
        &mut self,
        bounds: GraphNodeBounds,
        viewport_size: GraphPoint,
        padding: f32,
    ) -> Option<GraphBoundsFitResult> {
        let values = [
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.x,
            bounds.size.y,
            viewport_size.x,
            viewport_size.y,
            padding,
        ];
        if values.iter().any(|value| !value.is_finite())
            || bounds.size.x <= 0.0
            || bounds.size.y <= 0.0
            || viewport_size.x <= 0.0
            || viewport_size.y <= 0.0
            || padding < 0.0
        {
            return None;
        }
        let end = GraphPoint::new(bounds.right(), bounds.bottom());
        if !end.x.is_finite()
            || !end.y.is_finite()
            || end.x <= bounds.origin.x
            || end.y <= bounds.origin.y
        {
            return None;
        }
        let inset = f64::from(padding);
        let width = f64::from(viewport_size.x) - 2.0 * inset;
        let height = f64::from(viewport_size.y) - 2.0 * inset;
        if width <= 0.0 || height <= 0.0 {
            return None;
        }
        let desired_zoom =
            (width / f64::from(bounds.size.x)).min(height / f64::from(bounds.size.y));
        let mut zoom =
            desired_zoom.clamp(f64::from(GRAPH_MIN_ZOOM), f64::from(GRAPH_MAX_ZOOM)) as f32;
        // Rounding a fitting scale upward can cut off the far edge. Positive zooms are
        // ordered by their bit representation; stay within the existing minimum.
        if f64::from(zoom) > desired_zoom && zoom > GRAPH_MIN_ZOOM {
            zoom = f32::from_bits(zoom.to_bits() - 1);
        }
        let pan = GraphPoint::new(
            (f64::from(viewport_size.x) * 0.5
                - (f64::from(bounds.origin.x) + f64::from(bounds.size.x) * 0.5) * f64::from(zoom))
                as f32,
            (f64::from(viewport_size.y) * 0.5
                - (f64::from(bounds.origin.y) + f64::from(bounds.size.y) * 0.5) * f64::from(zoom))
                as f32,
        );
        let start_on_screen = GraphPoint::new(
            bounds.origin.x.mul_add(zoom, pan.x),
            bounds.origin.y.mul_add(zoom, pan.y),
        );
        let end_on_screen = GraphPoint::new(end.x.mul_add(zoom, pan.x), end.y.mul_add(zoom, pan.y));
        if [
            pan.x,
            pan.y,
            start_on_screen.x,
            start_on_screen.y,
            end_on_screen.x,
            end_on_screen.y,
        ]
        .iter()
        .any(|value| !value.is_finite())
            || end_on_screen.x <= start_on_screen.x
            || end_on_screen.y <= start_on_screen.y
        {
            return None;
        }
        let result = GraphBoundsFitResult {
            changed: self.pan != pan || self.zoom != zoom,
            fully_visible: start_on_screen.x >= padding
                && start_on_screen.y >= padding
                && end_on_screen.x <= viewport_size.x - padding
                && end_on_screen.y <= viewport_size.y - padding,
        };
        self.pan = pan;
        self.zoom = zoom;
        Some(result)
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
#[path = "tests/canvas.rs"]
mod tests;
