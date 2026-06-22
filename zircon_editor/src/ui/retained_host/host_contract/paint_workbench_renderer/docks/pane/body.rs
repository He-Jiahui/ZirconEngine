use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::draw_rect;
use super::super::super::{PANE_EMPTY, VIEWPORT_PANEL};
use super::super::viewport_toolbar;

pub(super) fn draw_pane_shell_and_body(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    content: &FrameRect,
) -> FrameRect {
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_background");
        draw_rect(frame, content.clone(), pane_background_color(pane));
    }
    if viewport_toolbar_is_visible(pane) {
        let toolbar = viewport_toolbar_frame(content);
        {
            zircon_runtime::profile_scope!(
                "editor",
                "host_painter",
                "painter_pane_viewport_toolbar"
            );
            viewport_toolbar::draw_viewport_toolbar(frame, pane, &toolbar, content);
        }
        body_after_toolbar(content, &toolbar)
    } else {
        content.clone()
    }
}

fn pane_background_color(pane: &PaneData) -> [u8; 4] {
    match pane.kind.as_str() {
        "Scene" | "Game" => VIEWPORT_PANEL,
        _ => PANE_EMPTY,
    }
}

fn viewport_toolbar_is_visible(pane: &PaneData) -> bool {
    matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar
}

fn viewport_toolbar_frame(content: &FrameRect) -> FrameRect {
    FrameRect {
        x: content.x,
        y: content.y,
        width: content.width,
        height: 28.0_f32.min(content.height),
    }
}

fn body_after_toolbar(content: &FrameRect, toolbar: &FrameRect) -> FrameRect {
    FrameRect {
        x: content.x,
        y: content.y + toolbar.height,
        width: content.width,
        height: (content.height - toolbar.height).max(0.0),
    }
}
