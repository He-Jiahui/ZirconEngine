//! Shared graph-authoring foundation for editor domain toolkits.

#[path = "canvas.rs"]
mod canvas;
#[path = "commands.rs"]
mod commands;
#[path = "model.rs"]
mod model;
#[path = "node_widget.rs"]
mod node_widget;
#[path = "routing.rs"]
mod routing;

pub use canvas::{
    GraphBoundsFitResult, GraphCanvasState, GraphNodeDrag, GraphNodeMove, GraphSelection,
};
pub use commands::{
    aligned_node_moves, GraphAlignment, GraphClipboardModel, GraphDeltaCommand, GraphEditContext,
};
pub use model::{
    default_connection_verdict, required_input_diagnostics, ConnectVerdict, GraphAttachmentView,
    GraphConnectRejection, GraphDeltaProofV1, GraphDiagnostic, GraphEdgeView, GraphModel,
    GraphMutationEffect, GraphNodeBounds, GraphNodeView, GraphPinDirection, GraphPinView,
    GraphPoint, GraphPortRef, PreparedGraphDeltaV1, StructureConstraint,
};
pub use node_widget::{GraphAttachmentPresentation, GraphNodePresentation};
pub use routing::{
    route_connection, route_connection_with_directions, GraphConnectionArrow, GraphConnectionRoute,
    GraphRouteDirection, GraphRouteStyle,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
