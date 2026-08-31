use super::super::data::{FrameRect, PaneData, WelcomePaneLayoutData};
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};
use super::SEPARATOR;

mod layout;
mod main_column;
mod recent_projects;
mod style;

use layout::{inset_frame, translated_welcome_frame, WELCOME_COLUMN_INSET};
use main_column::draw_welcome_main_column;
use recent_projects::draw_welcome_recent_projects;
use style::{WELCOME_BACKGROUND, WELCOME_SURFACE};

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_native_content(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) -> bool {
    if !pane.welcome.layout.has_nodes && pane.welcome.title.is_empty() {
        return false;
    }

    draw_rect_clipped(frame, body.clone(), Some(clip), WELCOME_BACKGROUND);

    let layout = &pane.welcome.layout;
    let (recent_panel, main_panel) = resolve_welcome_panel_frames(layout, body);
    if let Some(recent_panel) = recent_panel.as_ref() {
        draw_welcome_panel(frame, recent_panel, clip, WELCOME_SURFACE);
        draw_welcome_recent_projects(frame, pane, layout, body, recent_panel, clip);
    }
    if let Some(main_panel) = main_panel.as_ref() {
        draw_welcome_panel(frame, main_panel, clip, WELCOME_BACKGROUND);
        draw_welcome_main_column(frame, pane, layout, body, main_panel, clip);
    }
    true
}

fn resolve_welcome_panel_frames(
    layout: &WelcomePaneLayoutData,
    body: &FrameRect,
) -> (Option<FrameRect>, Option<FrameRect>) {
    let outer = translated_welcome_frame(layout.outer_panel.as_ref(), body)
        .unwrap_or_else(|| inset_frame(body, WELCOME_COLUMN_INSET, WELCOME_COLUMN_INSET));
    let fallback_recent = FrameRect {
        x: outer.x,
        y: outer.y,
        width: 320.0_f32.min(outer.width * 0.34).max(220.0),
        height: outer.height,
    };
    let recent_panel = translated_welcome_frame(layout.recent_panel.as_ref(), body)
        .or_else(|| (!layout.has_nodes).then_some(fallback_recent));
    let main_panel = translated_welcome_frame(layout.main_panel.as_ref(), body).or_else(|| {
        if layout.has_nodes {
            return None;
        }
        let recent_right = recent_panel
            .as_ref()
            .map_or(outer.x, |recent| recent.x + recent.width);
        Some(FrameRect {
            x: recent_right,
            y: outer.y,
            width: (outer.x + outer.width - recent_right).max(0.0),
            height: outer.height,
        })
    });
    (recent_panel, main_panel)
}

fn draw_welcome_panel(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: &FrameRect,
    color: [u8; 4],
) {
    draw_rect_clipped(frame, rect.clone(), Some(clip), color);
    draw_border_clipped(frame, rect.clone(), Some(clip), SEPARATOR);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authoritative_layout_keeps_a_collapsed_recent_panel_absent() {
        let body = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 640.0,
            height: 360.0,
        };
        let layout = WelcomePaneLayoutData {
            has_nodes: true,
            outer_panel: Some(FrameRect {
                x: 16.0,
                y: 12.0,
                width: 608.0,
                height: 336.0,
            }),
            main_panel: Some(FrameRect {
                x: 16.0,
                y: 12.0,
                width: 608.0,
                height: 336.0,
            }),
            ..WelcomePaneLayoutData::default()
        };

        let (recent, main) = resolve_welcome_panel_frames(&layout, &body);

        assert_eq!(recent, None);
        assert_eq!(main.expect("project task panel").width, 608.0);
    }

    #[test]
    fn legacy_layout_without_projected_nodes_retains_panel_fallbacks() {
        let body = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 900.0,
            height: 620.0,
        };

        let (recent, main) = resolve_welcome_panel_frames(&WelcomePaneLayoutData::default(), &body);

        let recent = recent.expect("legacy recent panel");
        let main = main.expect("legacy main panel");
        assert!(recent.width >= 220.0);
        assert_eq!(main.x, recent.x + recent.width);
        assert_eq!(main.x + main.width, body.width - WELCOME_COLUMN_INSET);
    }
}
