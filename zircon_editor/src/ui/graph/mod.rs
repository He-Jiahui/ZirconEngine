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

pub use canvas::{GraphCanvasState, GraphNodeDrag, GraphNodeMove, GraphSelection};
pub use commands::{
    aligned_node_moves, GraphAlignment, GraphClipboardModel, GraphDeltaCommand, GraphEditContext,
};
pub use model::{
    default_connection_verdict, required_input_diagnostics, ConnectVerdict, GraphAttachmentView,
    GraphConnectRejection, GraphDiagnostic, GraphEdgeView, GraphModel, GraphMutationEffect,
    GraphNodeBounds, GraphNodeView, GraphPinDirection, GraphPinView, GraphPoint, GraphPortRef,
    StructureConstraint,
};
pub use node_widget::GraphNodePresentation;
pub use routing::{route_connection, GraphConnectionRoute, GraphRouteStyle};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
