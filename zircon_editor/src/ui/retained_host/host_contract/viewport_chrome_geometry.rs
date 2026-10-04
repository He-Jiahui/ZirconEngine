//! One authored toolbar frame used by paint, input and profile consumers.
use super::data::{FrameRect, PaneData};

pub(super) fn viewport_toolbar_frame(pane: &PaneData, content: &FrameRect) -> Option<FrameRect> {
    if !matches!(pane.kind.as_str(), "Scene" | "Game") || !pane.show_toolbar {
        return None;
    }
    let authored = pane
        .viewport
        .toolbar_template_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "SceneViewportToolbarRoot");
    // Unprojected legacy/test panes retain their existing shell extent. Published
    // authored panes reserve the measured root extent. The surface origin stays
    // at content: node and hit frames already contain authored local offsets.
    Some(match authored {
        Some(root) => FrameRect {
            x: content.x,
            y: content.y,
            width: (root.frame.x + root.frame.width)
                .max(0.0)
                .min(content.width.max(0.0)),
            height: (root.frame.y + root.frame.height)
                .max(0.0)
                .min(content.height.max(0.0)),
        },
        None => FrameRect {
            x: content.x,
            y: content.y,
            width: content.width,
            height: 28.0_f32.min(content.height),
        },
    })
}

#[cfg(test)]
#[path = "tests/viewport_chrome_geometry.rs"]
mod tests;
