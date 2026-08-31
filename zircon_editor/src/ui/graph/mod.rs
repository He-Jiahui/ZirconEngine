//! Shared graph-authoring foundation for editor domain toolkits.

mod canvas;
mod commands;
mod model;
mod node_widget;
mod routing;

pub use canvas::{GraphCanvasState, GraphNodeDrag, GraphNodeMove, GraphSelection};
pub use commands::{
    GraphAlignment, GraphClipboardModel, GraphDeltaCommand, GraphEditContext, aligned_node_moves,
};
pub use model::{
    ConnectVerdict, GraphAttachmentView, GraphConnectRejection, GraphDiagnostic, GraphEdgeView,
    GraphModel, GraphMutationEffect, GraphNodeBounds, GraphNodeView, GraphPinDirection,
    GraphPinView, GraphPoint, GraphPortRef, StructureConstraint, default_connection_verdict,
    required_input_diagnostics,
};
pub use node_widget::GraphNodePresentation;
pub use routing::{GraphConnectionRoute, GraphRouteStyle, route_connection};

#[cfg(test)]
mod tests;
