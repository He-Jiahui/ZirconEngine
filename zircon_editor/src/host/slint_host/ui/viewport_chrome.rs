use super::*;

pub(super) fn blank_viewport_chrome() -> SceneViewportChromeData {
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

pub(super) fn scene_viewport_chrome(
    settings: &zircon_scene::SceneViewportSettings,
) -> SceneViewportChromeData {
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

fn scene_tool_label(tool: zircon_scene::SceneViewportTool) -> &'static str {
    match tool {
        zircon_scene::SceneViewportTool::Drag => "Drag",
        zircon_scene::SceneViewportTool::Move => "Move",
        zircon_scene::SceneViewportTool::Rotate => "Rotate",
        zircon_scene::SceneViewportTool::Scale => "Scale",
    }
}

fn transform_space_label(space: zircon_scene::TransformSpace) -> &'static str {
    match space {
        zircon_scene::TransformSpace::Local => "Local",
        zircon_scene::TransformSpace::Global => "Global",
    }
}

fn projection_mode_label(mode: zircon_scene::ProjectionMode) -> &'static str {
    match mode {
        zircon_scene::ProjectionMode::Perspective => "Perspective",
        zircon_scene::ProjectionMode::Orthographic => "Orthographic",
    }
}

fn view_orientation_label(orientation: zircon_scene::ViewOrientation) -> &'static str {
    match orientation {
        zircon_scene::ViewOrientation::User => "User",
        zircon_scene::ViewOrientation::PosX => "PosX",
        zircon_scene::ViewOrientation::NegX => "NegX",
        zircon_scene::ViewOrientation::PosY => "PosY",
        zircon_scene::ViewOrientation::NegY => "NegY",
        zircon_scene::ViewOrientation::PosZ => "PosZ",
        zircon_scene::ViewOrientation::NegZ => "NegZ",
    }
}

fn display_mode_label(mode: zircon_scene::DisplayMode) -> &'static str {
    match mode {
        zircon_scene::DisplayMode::Shaded => "Shaded",
        zircon_scene::DisplayMode::WireOverlay => "WireOverlay",
        zircon_scene::DisplayMode::WireOnly => "WireOnly",
    }
}

fn grid_mode_label(mode: zircon_scene::GridMode) -> &'static str {
    match mode {
        zircon_scene::GridMode::Hidden => "Hidden",
        zircon_scene::GridMode::VisibleNoSnap => "VisibleNoSnap",
        zircon_scene::GridMode::VisibleAndSnap => "VisibleAndSnap",
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
