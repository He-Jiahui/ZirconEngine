//! Scene viewport state, handle overlays, and editor-owned camera interaction.

mod controller;
#[cfg(test)]
mod edit_mode_projection;
mod handles;
mod interaction;
pub(crate) mod pointer;
mod projection;
mod render_packet;
mod settings;

pub(crate) use controller::SceneViewportController;
#[cfg(test)]
pub(crate) use edit_mode_projection::{SceneEditModeProjection, SceneInspectorFieldValue};
pub use interaction::{GizmoAxis, ViewportFeedback, ViewportInput, ViewportState};
pub use settings::{
    GridMode, SceneViewportSettings, SceneViewportTool, TransformSpace, ViewOrientation,
};
pub(crate) use zircon_runtime::core::framework::render::*;
