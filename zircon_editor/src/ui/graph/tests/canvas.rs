use super::GraphCanvasState;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct NodeIdWithoutDefault(u64);

#[test]
fn default_canvas_does_not_require_default_node_ids() {
    let canvas = GraphCanvasState::<NodeIdWithoutDefault>::default();

    assert!(canvas.selection().node_ids().is_empty());
}
