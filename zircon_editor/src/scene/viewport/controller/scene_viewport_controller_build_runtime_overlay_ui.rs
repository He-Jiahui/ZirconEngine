use crate::scene::viewport::{DisplayMode, GridMode, ProjectionMode, SceneViewportTool};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiFrame,
    surface::{
        UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
        UiTextAlign, UiTextRenderMode, UiTextWrap,
    },
};

use super::SceneViewportController;

const VIEWPORT_HUD_TREE_ID: &str = "zircon.editor.viewport.hud";
const VIEWPORT_HUD_NODE_ID: u64 = 1;
const VIEWPORT_HUD_Z_INDEX: i32 = 10;
const VIEWPORT_HUD_MARGIN_X: f32 = 16.0;
const VIEWPORT_HUD_MARGIN_Y: f32 = 16.0;
const VIEWPORT_HUD_WIDTH: f32 = 280.0;
const VIEWPORT_HUD_MIN_WIDTH: f32 = 48.0;
const VIEWPORT_HUD_HEIGHT: f32 = 28.0;
const VIEWPORT_HUD_FONT_SIZE: f32 = 13.0;
const VIEWPORT_HUD_LINE_HEIGHT: f32 = 16.0;
const VIEWPORT_HUD_BACKGROUND: &str = "#16202ccc";
const VIEWPORT_HUD_FOREGROUND: &str = "#eef3ff";
const VIEWPORT_HUD_FONT: &str = "res://fonts/default.font.toml";
const VIEWPORT_HUD_OPACITY: f32 = 1.0;

impl SceneViewportController {
    pub(crate) fn build_runtime_overlay_ui(&self) -> Option<UiRenderExtract> {
        let max_width =
            (self.state.viewport.size.x as f32 - VIEWPORT_HUD_MARGIN_X - VIEWPORT_HUD_MARGIN_X)
                .max(VIEWPORT_HUD_MIN_WIDTH);
        let frame = UiFrame::new(
            VIEWPORT_HUD_MARGIN_X,
            VIEWPORT_HUD_MARGIN_Y,
            VIEWPORT_HUD_WIDTH.min(max_width),
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
                    z_index: VIEWPORT_HUD_Z_INDEX,
                    style: UiResolvedStyle {
                        background_color: Some(VIEWPORT_HUD_BACKGROUND.to_string()),
                        foreground_color: Some(VIEWPORT_HUD_FOREGROUND.to_string()),
                        font: Some(VIEWPORT_HUD_FONT.to_string()),
                        font_size: VIEWPORT_HUD_FONT_SIZE,
                        line_height: VIEWPORT_HUD_LINE_HEIGHT,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::None,
                        text_render_mode: UiTextRenderMode::Auto,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some(self.runtime_hud_text()),
                    image: None,
                    opacity: VIEWPORT_HUD_OPACITY,
                }],
            },
        })
    }

    fn runtime_hud_text(&self) -> String {
        format!(
            "{} | {} | {} | {}",
            tool_label(self.state.settings.tool),
            projection_label(self.state.settings.projection_mode),
            display_label(self.state.settings.display_mode),
            grid_label(self.state.settings.grid_mode)
        )
    }
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

fn display_label(display: DisplayMode) -> &'static str {
    match display {
        DisplayMode::Shaded => "Shaded",
        DisplayMode::WireOverlay => "Wire+Shaded",
        DisplayMode::WireOnly => "Wire",
    }
}

fn grid_label(grid: GridMode) -> &'static str {
    match grid {
        GridMode::Hidden => "Grid Off",
        GridMode::VisibleNoSnap => "Grid",
        GridMode::VisibleAndSnap => "Snap Grid",
    }
}
