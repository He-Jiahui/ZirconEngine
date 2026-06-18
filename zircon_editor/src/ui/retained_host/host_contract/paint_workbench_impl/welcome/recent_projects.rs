use super::super::super::data::{FrameRect, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;
use super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::{first_non_empty, SEPARATOR};
use super::layout::{welcome_node_frame, WELCOME_ROW_GAP, WELCOME_ROW_HEIGHT};
use super::style::{
    WELCOME_MUTED_TEXT, WELCOME_SURFACE, WELCOME_SURFACE_INSET, WELCOME_TEXT, WELCOME_WARNING,
};

pub(super) fn draw_welcome_recent_projects(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    recent_panel: &FrameRect,
    clip: &FrameRect,
) {
    let header =
        welcome_node_frame(pane, body, "WelcomeRecentHeaderPanel").unwrap_or_else(|| FrameRect {
            x: recent_panel.x,
            y: recent_panel.y + 26.0,
            width: recent_panel.width,
            height: 54.0,
        });
    draw_text_bars_clipped(
        frame,
        header.x + 18.0,
        header.y + 6.0,
        "Recent Projects",
        Some(clip),
        WELCOME_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        header.x + 18.0,
        header.y + 30.0,
        "Pinned startup workspace",
        Some(clip),
        WELCOME_MUTED_TEXT,
    );

    let list =
        welcome_node_frame(pane, body, "WelcomeRecentListPanel").unwrap_or_else(|| FrameRect {
            x: recent_panel.x + 12.0,
            y: header.y + header.height + 14.0,
            width: (recent_panel.width - 24.0).max(0.0),
            height: (recent_panel.height - header.height - 40.0).max(0.0),
        });
    draw_rect_clipped(frame, list.clone(), Some(clip), WELCOME_SURFACE_INSET);
    draw_border_clipped(frame, list.clone(), Some(clip), SEPARATOR);

    let row_count = pane.welcome.recent_projects.row_count();
    if row_count == 0 {
        draw_text_bars_clipped(
            frame,
            list.x + 14.0,
            list.y + 16.0,
            "No recent projects",
            Some(clip),
            WELCOME_MUTED_TEXT,
        );
        draw_text_bars_clipped(
            frame,
            list.x + 14.0,
            list.y + 40.0,
            "Create a new project to start",
            Some(clip),
            WELCOME_MUTED_TEXT,
        );
        return;
    }

    let visible_rows = row_count.min(7);
    for index in 0..visible_rows {
        let Some(recent) = pane.welcome.recent_projects.row_data(index) else {
            continue;
        };
        let row = FrameRect {
            x: list.x + 8.0,
            y: list.y + 8.0 + index as f32 * (WELCOME_ROW_HEIGHT + WELCOME_ROW_GAP),
            width: (list.width - 16.0).max(0.0),
            height: WELCOME_ROW_HEIGHT,
        };
        if intersect(&row, clip).is_none() {
            continue;
        }
        draw_rect_clipped(frame, row.clone(), Some(clip), WELCOME_SURFACE);
        draw_border_clipped(
            frame,
            row.clone(),
            Some(clip),
            if recent.invalid {
                WELCOME_WARNING
            } else {
                SEPARATOR
            },
        );
        draw_text_bars_clipped(
            frame,
            row.x + 12.0,
            row.y + 8.0,
            recent.display_name.as_str(),
            Some(clip),
            WELCOME_TEXT,
        );
        draw_text_bars_clipped(
            frame,
            row.x + 12.0,
            row.y + 28.0,
            recent.path.as_str(),
            Some(clip),
            WELCOME_MUTED_TEXT,
        );
        let status = first_non_empty(&[
            recent.status_label.as_str(),
            recent.last_opened_label.as_str(),
        ]);
        if !status.is_empty() {
            draw_text_bars_clipped(
                frame,
                row.x + row.width - 108.0_f32.min(row.width * 0.38),
                row.y + 8.0,
                status,
                Some(clip),
                if recent.invalid {
                    WELCOME_WARNING
                } else {
                    WELCOME_MUTED_TEXT
                },
            );
        }
    }
}
