use super::super::super::data::{FrameRect, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::{first_non_empty, ACCENT, SEPARATOR};
use super::layout::{constrain_welcome_content, welcome_node_frame, WELCOME_CONTENT_MAX_WIDTH};
use super::style::{
    WELCOME_ACTION_DISABLED_SURFACE, WELCOME_ACTION_DISABLED_TEXT, WELCOME_MUTED_TEXT,
    WELCOME_PRIMARY_ACTION, WELCOME_SUCCESS, WELCOME_SURFACE, WELCOME_SURFACE_HOVERED,
    WELCOME_SURFACE_INSET, WELCOME_TEXT, WELCOME_WARNING,
};

pub(super) fn draw_welcome_main_column(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    main_panel: &FrameRect,
    clip: &FrameRect,
) {
    let content_x = main_panel.x + 28.0;
    let content_width = (main_panel.width - 56.0)
        .max(0.0)
        .min(WELCOME_CONTENT_MAX_WIDTH);
    let hero = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeHeroPanel").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: main_panel.y + 28.0,
            width: content_width,
            height: 84.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_hero(frame, pane, &hero, clip);

    let status = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeStatusPanel").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: hero.y + hero.height + 12.0,
            width: content_width,
            height: 30.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_status(frame, pane, &status, clip);

    let header = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeNewProjectHeaderPanel").unwrap_or_else(|| {
            FrameRect {
                x: content_x,
                y: status.y + status.height + 22.0,
                width: content_width,
                height: 46.0,
            }
        }),
        content_x,
        content_width,
    );
    draw_text_bars_clipped(
        frame,
        header.x,
        header.y + 2.0,
        "New Project",
        Some(clip),
        WELCOME_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        header.x,
        header.y + 24.0,
        pane.welcome.form.template_label.as_str(),
        Some(clip),
        WELCOME_MUTED_TEXT,
    );

    let name = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeProjectNameField").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: header.y + header.height + 16.0,
            width: content_width,
            height: 56.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_field(
        frame,
        &name,
        "Project name",
        pane.welcome.form.project_name.as_str(),
        clip,
    );

    let location = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeLocationField").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: name.y + name.height + 12.0,
            width: content_width,
            height: 56.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_field(
        frame,
        &location,
        "Location",
        pane.welcome.form.location.as_str(),
        clip,
    );

    let preview = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomePreviewPanel").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: location.y + location.height + 14.0,
            width: content_width,
            height: 72.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_preview(frame, pane, &preview, clip);

    let validation = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeValidationPanel").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: preview.y + preview.height + 10.0,
            width: content_width,
            height: 36.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_validation(frame, pane, &validation, clip);

    let actions = constrain_welcome_content(
        welcome_node_frame(pane, body, "WelcomeActionsRow").unwrap_or_else(|| FrameRect {
            x: content_x,
            y: validation.y + validation.height + 12.0,
            width: content_width,
            height: 32.0,
        }),
        content_x,
        content_width,
    );
    draw_welcome_actions(frame, pane, &actions, clip);
}

fn draw_welcome_hero(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    hero: &FrameRect,
    clip: &FrameRect,
) {
    draw_text_bars_clipped(
        frame,
        hero.x,
        hero.y + 4.0,
        first_non_empty(&[pane.welcome.title.as_str(), "Open or Create"]),
        Some(clip),
        WELCOME_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        hero.x,
        hero.y + 30.0,
        first_non_empty(&[
            pane.welcome.subtitle.as_str(),
            "Recent projects and a renderable empty-project template",
        ]),
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
    let accent = FrameRect {
        x: hero.x,
        y: hero.y + hero.height - 10.0,
        width: 96.0_f32.min(hero.width),
        height: 2.0,
    };
    draw_rect_clipped(frame, accent, Some(clip), ACCENT);
}

fn draw_welcome_status(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    status: &FrameRect,
    clip: &FrameRect,
) {
    draw_rect_clipped(frame, status.clone(), Some(clip), WELCOME_SURFACE_INSET);
    draw_border_clipped(frame, status.clone(), Some(clip), SEPARATOR);
    let marker = FrameRect {
        x: status.x + 10.0,
        y: status.y + 10.0,
        width: 8.0,
        height: 8.0,
    };
    draw_rect_clipped(frame, marker, Some(clip), WELCOME_SUCCESS);
    draw_text_bars_clipped(
        frame,
        status.x + 28.0,
        status.y + 7.0,
        first_non_empty(&[pane.welcome.status_message.as_str(), "Ready"]),
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
}

fn draw_welcome_field(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    label: &str,
    value: &str,
    clip: &FrameRect,
) {
    draw_rect_clipped(frame, rect.clone(), Some(clip), WELCOME_SURFACE);
    draw_border_clipped(frame, rect.clone(), Some(clip), SEPARATOR);
    draw_text_bars_clipped(
        frame,
        rect.x + 14.0,
        rect.y + 8.0,
        label,
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        rect.x + 14.0,
        rect.y + 30.0,
        value,
        Some(clip),
        WELCOME_TEXT,
    );
}

fn draw_welcome_preview(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    preview: &FrameRect,
    clip: &FrameRect,
) {
    draw_rect_clipped(frame, preview.clone(), Some(clip), WELCOME_SURFACE);
    draw_border_clipped(frame, preview.clone(), Some(clip), SEPARATOR);
    draw_text_bars_clipped(
        frame,
        preview.x + 14.0,
        preview.y + 10.0,
        "Project path",
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        preview.x + 14.0,
        preview.y + 36.0,
        first_non_empty(&[
            pane.welcome.form.project_path_preview.as_str(),
            "Project path will appear here",
        ]),
        Some(clip),
        if pane.welcome.form.project_path_preview.is_empty() {
            WELCOME_MUTED_TEXT
        } else {
            WELCOME_TEXT
        },
    );
}

fn draw_welcome_validation(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    validation: &FrameRect,
    clip: &FrameRect,
) {
    let message = if !pane.welcome.form.validation_message.trim().is_empty() {
        pane.welcome.form.validation_message.as_str()
    } else if pane.welcome.form.can_create {
        "Project settings are valid"
    } else {
        "Enter a project name and location"
    };
    let color = if pane.welcome.form.can_create {
        WELCOME_SUCCESS
    } else {
        WELCOME_WARNING
    };
    draw_text_bars_clipped(
        frame,
        validation.x,
        validation.y + 8.0,
        message,
        Some(clip),
        color,
    );
}

fn draw_welcome_actions(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    actions: &FrameRect,
    clip: &FrameRect,
) {
    let create_width = 154.0_f32.min(actions.width * 0.45);
    let open_width = 116.0_f32.min(actions.width * 0.34);
    let gap = 10.0_f32.min(actions.width * 0.04);
    let create = FrameRect {
        x: actions.x + actions.width - create_width,
        y: actions.y,
        width: create_width,
        height: actions.height,
    };
    let open = FrameRect {
        x: (create.x - gap - open_width).max(actions.x),
        y: actions.y,
        width: open_width,
        height: actions.height,
    };
    draw_welcome_button(
        frame,
        &open,
        "Open",
        false,
        pane.welcome.form.can_open_existing,
        clip,
    );
    draw_welcome_button(
        frame,
        &create,
        "Create Project",
        true,
        pane.welcome.form.can_create,
        clip,
    );
}

fn draw_welcome_button(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    label: &str,
    primary: bool,
    enabled: bool,
    clip: &FrameRect,
) {
    let color = if !enabled {
        WELCOME_ACTION_DISABLED_SURFACE
    } else if primary {
        WELCOME_PRIMARY_ACTION
    } else {
        WELCOME_SURFACE_HOVERED
    };
    let text = if enabled {
        WELCOME_TEXT
    } else {
        WELCOME_ACTION_DISABLED_TEXT
    };
    draw_rect_clipped(frame, rect.clone(), Some(clip), color);
    draw_border_clipped(frame, rect.clone(), Some(clip), SEPARATOR);
    draw_text_bars_clipped(frame, rect.x + 14.0, rect.y + 8.0, label, Some(clip), text);
}
