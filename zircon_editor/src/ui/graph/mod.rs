//! Shared graph-authoring foundation for editor domain toolkits.

mod canvas;
mod commands;
mod model;
mod node_widget;
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
