use slint::SharedString;

use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportSettings, SceneViewportTool,
    TransformSpace, ViewOrientation,
};
use crate::ui::slint_host::SceneViewportChromeData;

pub(crate) fn blank_viewport_chrome() -> SceneViewportChromeData {
    SceneViewportChromeData {
        tool: SharedString::default(),
        transform_space: SharedString::default(),
        projection_mode: SharedString::default(),
        view_orientation: SharedString::default(),
        display_mode: SharedString::default(),
        grid_mode: SharedString::default(),
        gizmos_enabled: false,
        preview_lighting: false,
        preview_skybox: false,
        translate_snap: 0.0,
        rotate_snap_deg: 0.0,
        scale_snap: 0.0,
        translate_snap_label: SharedString::default(),
        rotate_snap_label: SharedString::default(),
        scale_snap_label: SharedString::default(),
    }
}

pub(crate) fn scene_viewport_chrome(settings: &SceneViewportSettings) -> SceneViewportChromeData {
    SceneViewportChromeData {
        tool: scene_tool_label(settings.tool).into(),
        transform_space: transform_space_label(settings.transform_space).into(),
        projection_mode: projection_mode_label(settings.projection_mode).into(),
        view_orientation: view_orientation_label(settings.view_orientation).into(),
        display_mode: display_mode_label(settings.display_mode).into(),
        grid_mode: grid_mode_label(settings.grid_mode).into(),
        gizmos_enabled: settings.gizmos_enabled,
        preview_lighting: settings.preview_lighting,
        preview_skybox: settings.preview_skybox,
        translate_snap: settings.translate_step,
        rotate_snap_deg: settings.rotate_step_deg,
        scale_snap: settings.scale_step,
        translate_snap_label: format!("T {}", format_step(settings.translate_step)).into(),
        rotate_snap_label: format!("R {}", format_step(settings.rotate_step_deg)).into(),
        scale_snap_label: format!("S {}", format_step(settings.scale_step)).into(),
    }
}

fn scene_tool_label(tool: SceneViewportTool) -> &'static str {
    match tool {
        SceneViewportTool::Drag => "Drag",
        SceneViewportTool::Move => "Move",
        SceneViewportTool::Rotate => "Rotate",
        SceneViewportTool::Scale => "Scale",
    }
}

fn transform_space_label(space: TransformSpace) -> &'static str {
    match space {
        TransformSpace::Local => "Local",
        TransformSpace::Global => "Global",
    }
}

fn projection_mode_label(mode: ProjectionMode) -> &'static str {
    match mode {
        ProjectionMode::Perspective => "Perspective",
        ProjectionMode::Orthographic => "Orthographic",
    }
}

fn view_orientation_label(orientation: ViewOrientation) -> &'static str {
    match orientation {
        ViewOrientation::User => "User",
        ViewOrientation::PosX => "PosX",
        ViewOrientation::NegX => "NegX",
        ViewOrientation::PosY => "PosY",
        ViewOrientation::NegY => "NegY",
        ViewOrientation::PosZ => "PosZ",
        ViewOrientation::NegZ => "NegZ",
    }
}

fn display_mode_label(mode: DisplayMode) -> &'static str {
    match mode {
        DisplayMode::Shaded => "Shaded",
        DisplayMode::WireOverlay => "WireOverlay",
        DisplayMode::WireOnly => "WireOnly",
    }
}

fn grid_mode_label(mode: GridMode) -> &'static str {
    match mode {
        GridMode::Hidden => "Hidden",
        GridMode::VisibleNoSnap => "VisibleNoSnap",
        GridMode::VisibleAndSnap => "VisibleAndSnap",
    }
}

fn format_step(value: f32) -> String {
    if value.fract().abs() <= f32::EPSILON {
        format!("{value:.0}")
    } else if (value * 10.0).fract().abs() <= f32::EPSILON {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
}
