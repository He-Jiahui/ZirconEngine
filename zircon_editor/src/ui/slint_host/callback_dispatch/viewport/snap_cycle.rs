use crate::scene::viewport::{DisplayMode, GridMode};

pub(in crate::ui::slint_host::callback_dispatch::viewport) fn next_display_mode_name(
    mode: DisplayMode,
) -> &'static str {
    match mode {
        DisplayMode::Shaded => "WireOverlay",
        DisplayMode::WireOverlay => "WireOnly",
        DisplayMode::WireOnly => "Shaded",
    }
}

pub(in crate::ui::slint_host::callback_dispatch::viewport) fn next_grid_mode_name(
    mode: GridMode,
) -> &'static str {
    match mode {
        GridMode::Hidden => "VisibleNoSnap",
        GridMode::VisibleNoSnap => "VisibleAndSnap",
        GridMode::VisibleAndSnap => "Hidden",
    }
}

pub(in crate::ui::slint_host::callback_dispatch::viewport) fn next_translate_snap(
    step: f32,
) -> f32 {
    if step <= 0.1 {
        0.25
    } else if step <= 0.25 {
        0.5
    } else if step <= 0.5 {
        1.0
    } else if step <= 1.0 {
        2.0
    } else {
        0.1
    }
}

pub(in crate::ui::slint_host::callback_dispatch::viewport) fn next_rotate_snap_degrees(
    step: f32,
) -> f32 {
    if step <= 5.0 {
        15.0
    } else if step <= 15.0 {
        30.0
    } else if step <= 30.0 {
        45.0
    } else if step <= 45.0 {
        90.0
    } else {
        5.0
    }
}

pub(in crate::ui::slint_host::callback_dispatch::viewport) fn next_scale_snap(step: f32) -> f32 {
    if step <= 0.05 {
        0.1
    } else if step <= 0.1 {
        0.25
    } else if step <= 0.25 {
        0.5
    } else if step <= 0.5 {
        1.0
    } else {
        0.05
    }
}
