use crate::scene::viewport::{DisplayMode, ProjectionMode, SceneViewportTool};
use zircon_runtime::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime::ui::layout::UiFrame;
use zircon_runtime::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap,
};

use super::SceneViewportController;

const VIEWPORT_HUD_TREE_ID: &str = "zircon.editor.viewport.hud";
const VIEWPORT_HUD_NODE_ID: u64 = 1;
const VIEWPORT_HUD_MARGIN: f32 = 12.0;
const VIEWPORT_HUD_WIDTH: f32 = 196.0;
const VIEWPORT_HUD_HEIGHT: f32 = 26.0;

impl SceneViewportController {
    pub(crate) fn build_runtime_overlay_ui(&self) -> Option<UiRenderExtract> {
        let viewport = self.state.viewport.size;
        if viewport.x < 160 || viewport.y < 64 {
            return None;
        }

        let max_width = viewport.x.saturating_sub((VIEWPORT_HUD_MARGIN as u32) * 2) as f32;
        let width = VIEWPORT_HUD_WIDTH.min(max_width).max(120.0);
        let frame = UiFrame::new(
            viewport.x as f32 - width - VIEWPORT_HUD_MARGIN,
            VIEWPORT_HUD_MARGIN,
            width,
            VIEWPORT_HUD_HEIGHT,
        );

        Some(UiRenderExtract {
            tree_id: UiTreeId::new(VIEWPORT_HUD_TREE_ID),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(VIEWPORT_HUD_NODE_ID),
                    kind: UiRenderCommandKind::Quad,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#0f1723cc".to_string()),
                        foreground_color: Some("#eef4ff".to_string()),
                        border_color: Some("#6fb7ff88".to_string()),
                        border_width: 1.0,
                        font: Some("res://fonts/default.font.toml".to_string()),
                        font_size: 13.0,
                        line_height: 16.0,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::None,
                        text_render_mode: UiTextRenderMode::Auto,
                        ..UiResolvedStyle::default()
                    },
                    text: Some(viewport_hud_text(
                        self.state.settings.tool,
                        self.state.settings.projection_mode,
                        self.state.settings.display_mode,
                    )),
                    image: None,
                    opacity: 1.0,
                }],
            },
        })
    }
}

fn viewport_hud_text(
    tool: SceneViewportTool,
    projection: ProjectionMode,
    display_mode: DisplayMode,
) -> String {
    format!(
        "{} | {} | {}",
        tool_label(tool),
        projection_label(projection),
        display_mode_label(display_mode)
    )
}

fn tool_label(tool: SceneViewportTool) -> &'static str {
    match tool {
        SceneViewportTool::Drag => "Drag",
        SceneViewportTool::Move => "Move",
        SceneViewportTool::Rotate => "Rotate",
        SceneViewportTool::Scale => "Scale",
    }
}

fn projection_label(projection: ProjectionMode) -> &'static str {
    match projection {
        ProjectionMode::Perspective => "Persp",
        ProjectionMode::Orthographic => "Ortho",
    }
}

fn display_mode_label(display_mode: DisplayMode) -> &'static str {
    match display_mode {
        DisplayMode::Shaded => "Shaded",
        DisplayMode::WireOverlay => "Wire+",
        DisplayMode::WireOnly => "Wire",
    }
}
