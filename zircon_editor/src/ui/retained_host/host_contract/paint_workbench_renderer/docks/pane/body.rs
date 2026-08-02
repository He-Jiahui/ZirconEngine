use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::draw_rect;
use super::super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use super::super::viewport_toolbar;

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

fn pane_background_color(kind: &str, palette: HostMaterialPalette) -> [u8; 4] {
    match kind {
        "Scene" | "Game" => palette.shell_background,
        _ => palette.surface_inset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn pane_backgrounds_project_viewport_and_empty_roles_from_the_current_theme() {
        let mut palette = PALETTE;
        palette.shell_background = [3, 5, 7, 255];
        palette.surface_inset = [11, 13, 17, 255];

        assert_eq!(pane_background_color("Scene", palette), [3, 5, 7, 255]);
        assert_eq!(pane_background_color("Game", palette), [3, 5, 7, 255]);
        assert_eq!(
            pane_background_color("Hierarchy", palette),
            [11, 13, 17, 255]
        );
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
