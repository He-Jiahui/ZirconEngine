use super::layout::{command_palette_metrics, command_palette_metrics_from_host, empty_text_rect};
use super::palette::command_palette_palette_from_host;
use super::panel::push_command_palette_empty_message;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

#[test]
fn command_palette_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.radius_control = 5.0;
    host.border_width = 1.5;
    host.font_body = 11.0;
    host.font_large = 15.0;
    host.line_height_ratio = 1.25;
    host.input_pad = [7.0, 8.0, 4.0, 5.0];
    host.gap_s = 5.0;
    host.gap_m = 9.0;
    host.gap_l = 13.0;
    host.row_height = 27.0;

    let metrics = command_palette_metrics_from_host(host);

    assert_eq!(metrics.panel_radius, 10.0);
    assert_eq!(metrics.search_radius, 2.0);
    assert_eq!(metrics.row_radius, 2.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.min_frame_extent, 1.5);
    assert_eq!(metrics.font_size, 13.0);
    assert_eq!(metrics.line_height, 16.25);
    assert_eq!(metrics.panel_padding_x, 13.0);
    assert_eq!(metrics.search_top, 12.0);
    assert_eq!(metrics.search_height, 35.0);
    assert_eq!(metrics.search_icon_size, 14.0);
    assert_eq!(metrics.search_icon_x, 10.0);
    assert_eq!(metrics.search_text_x, 29.0);
    assert_eq!(metrics.search_text_y, 9.0);
    assert_eq!(metrics.list_top, 56.0);
    assert_eq!(metrics.row_inset_x, 9.0);
    assert_eq!(metrics.row_height, 30.0);
    assert_eq!(metrics.row_text_x, 8.5);
    assert_eq!(metrics.row_text_y, 7.0);
    assert_eq!(metrics.row_detail_left_ratio, 0.72);
    assert_eq!(metrics.row_detail_width_ratio, 0.24);
    assert_eq!(metrics.match_indicator_left, 5.0);
    assert_eq!(metrics.match_indicator_width, 2.0);
    assert_eq!(metrics.match_indicator_height, 15.0);
    assert_eq!(metrics.empty_text_y, 68.0);
}

#[test]
fn command_palette_palette_projects_from_host_palette() {
    let mut host = PALETTE;
    host.popup = [10, 11, 12, 255];
    host.border = [20, 21, 22, 255];
    host.surface_inset = [30, 31, 32, 255];
    host.focus_ring = [40, 41, 42, 255];
    host.text = [50, 51, 52, 255];
    host.text_muted = [60, 61, 62, 255];
    host.accent = [70, 71, 72, 255];
    host.text_disabled = [80, 81, 82, 255];

    let palette = command_palette_palette_from_host(host);

    assert_eq!(palette.panel_surface, [10, 11, 12, 255]);
    assert_eq!(palette.panel_border, [20, 21, 22, 255]);
    assert_eq!(palette.search_surface, [30, 31, 32, 255]);
    assert_eq!(palette.search_idle_border, [20, 21, 22, 255]);
    assert_eq!(palette.search_focus_border, [40, 41, 42, 255]);
    assert_eq!(palette.search_icon, [60, 61, 62, 255]);
    assert_eq!(palette.text, [50, 51, 52, 255]);
    assert_eq!(palette.placeholder, [60, 61, 62, 255]);
    assert_eq!(palette.empty_text, [60, 61, 62, 255]);
    assert_eq!(palette.match_indicator, [70, 71, 72, 255]);
    assert_eq!(palette.match_indicator_disabled, [80, 81, 82, 255]);
}

#[test]
fn empty_message_uses_the_remaining_panel_content_band() {
    let metrics = command_palette_metrics();
    let panel = FrameRect {
        x: 20.0,
        y: 30.0,
        width: 240.0,
        height: metrics.empty_text_y + metrics.panel_padding_x + metrics.line_height * 3.0,
    };

    let text = empty_text_rect(&panel);

    assert_eq!(text.x, panel.x + metrics.panel_padding_x);
    assert_eq!(text.y, panel.y + metrics.empty_text_y);
    assert_eq!(text.height, metrics.line_height * 3.0);
    assert!(text.y + text.height <= panel.y + panel.height);
}

#[test]
fn empty_message_uses_runtime_word_wrap() {
    let panel = FrameRect {
        x: 20.0,
        y: 30.0,
        width: 240.0,
        height: 220.0,
    };
    let mut commands = Vec::new();

    push_command_palette_empty_message(&mut commands, &panel, &panel, 10, 1.0);

    assert_eq!(commands.len(), 1);
    assert_eq!(
        commands[0].text_layout_policy,
        HostTextLayoutPolicy::WordWrap
    );
    assert!(commands[0].frame.height > commands[0].line_height);
}
