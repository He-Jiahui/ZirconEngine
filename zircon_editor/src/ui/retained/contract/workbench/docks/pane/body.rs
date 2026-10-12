use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::draw_rect;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::viewport_toolbar;

// pane 内容的绘制起点由种类决定；视口工具栏占用内容上缘后，返回剩余区域供模板与原生内容共用。
pub(super) fn draw_pane_shell_and_body(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    content: &FrameRect,
) -> FrameRect {
    let palette = current_host_palette();
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_background");
        draw_rect(
            frame,
            content.clone(),
            pane_background_color(pane.kind.as_str(), palette),
        );
    }
    if let Some(toolbar) =
        super::super::super::super::viewport_chrome_geometry::viewport_toolbar_frame(pane, content)
    {
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

fn pane_background_color(kind: &str, palette: HostMaterialPalette) -> [u8; 4] {
    match kind {
        "Scene" | "Game" => palette.shell_background,
        _ => palette.surface_inset,
    }
}

#[cfg(test)]
#[path = "tests/body.rs"]
mod tests;

fn body_after_toolbar(content: &FrameRect, toolbar: &FrameRect) -> FrameRect {
    FrameRect {
        x: content.x,
        y: content.y + toolbar.height,
        width: content.width,
        height: (content.height - toolbar.height).max(0.0),
    }
}
