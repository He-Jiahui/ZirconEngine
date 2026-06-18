use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use super::data::{FrameRect, HostWindowPresentationData};
use super::presenter::{paint_host_presentation_snapshot, HostPresenterBackend};
use crate::ui::retained_host::primitives::PhysicalSize;

mod geometry;

#[cfg(test)]
use super::data::{HostChromeTabData, TemplatePaneNodeData};
#[cfg(test)]
use crate::ui::retained_host::primitives::ModelRc;
#[cfg(test)]
use geometry::collect_surface_frame_controls;

const GEOMETRY_FILE: &str = "ui_profile_geometry.json";
const REFERENCE_SCREENSHOT_FILE: &str = "screenshot_reference.png";

pub(super) fn export_present_artifacts(
    presentation: &HostWindowPresentationData,
    size: &PhysicalSize,
    backend: HostPresenterBackend,
) {
    if !profile_capture_enabled() {
        return;
    }
    let Some(export_dir) = profile_export_dir() else {
        return;
    };
    if fs::create_dir_all(&export_dir).is_err() || is_forced_softbuffer_screenshot_run() {
        return;
    }

    let geometry = UiProfileGeometry::from_presentation(presentation, size, backend);
    if let Ok(bytes) = serde_json::to_vec_pretty(&geometry) {
        let _ = fs::write(export_dir.join(GEOMETRY_FILE), bytes);
    }

    if profile_screenshot_capture_enabled() {
        let frame = paint_host_presentation_snapshot(size.width, size.height, presentation);
        let _ = image::save_buffer_with_format(
            export_dir.join(REFERENCE_SCREENSHOT_FILE),
            frame.as_bytes(),
            frame.width(),
            frame.height(),
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        );
    }
}

#[derive(Serialize)]
struct UiProfileGeometry {
    schema_version: u32,
    presenter_backend: &'static str,
    window_client_size: UiProfileSize,
    layout: UiProfileLayout,
    resize_splitters: Vec<UiProfileNamedFrame>,
    document_tabs: Vec<UiProfileTabFrame>,
    drawer_tabs: Vec<UiProfileTabFrame>,
    host_page_tabs: Vec<UiProfileTabFrame>,
    activity_rail_buttons: Vec<UiProfileNamedFrame>,
    viewport_frame: Option<UiProfileFrame>,
    viewport_toolbar_controls: Vec<UiProfileNamedFrame>,
    template_controls: Vec<UiProfileNamedFrame>,
    clickable_frames: Vec<UiProfileNamedFrame>,
    hit_samples: Vec<UiProfileHitSample>,
}

#[derive(Clone, Serialize)]
struct UiProfileNamedFrame {
    id: String,
    kind: String,
    surface: String,
    frame: UiProfileFrame,
    #[serde(skip_serializing_if = "Option::is_none")]
    clip: Option<UiProfileFrame>,
}

impl UiProfileNamedFrame {
    fn from_tab(tab: &UiProfileTabFrame) -> Self {
        Self {
            id: tab.id.clone(),
            kind: tab.kind.clone(),
            surface: tab.surface.clone(),
            frame: tab.frame.clone(),
            clip: None,
        }
    }
}

#[derive(Clone, Serialize)]
struct UiProfileTabFrame {
    id: String,
    title: String,
    kind: String,
    surface: String,
    frame: UiProfileFrame,
    close_frame: UiProfileFrame,
    active: bool,
}

#[derive(Clone, Serialize)]
struct UiProfileHitSample {
    id: String,
    kind: String,
    surface: String,
    sample: String,
    point: UiProfilePoint,
    expected_hit: bool,
    route_hit: bool,
}

#[derive(Clone, Serialize)]
struct UiProfileFrame {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl From<FrameRect> for UiProfileFrame {
    fn from(frame: FrameRect) -> Self {
        Self {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

impl From<&FrameRect> for UiProfileFrame {
    fn from(frame: &FrameRect) -> Self {
        Self {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

#[derive(Clone, Serialize)]
struct UiProfilePoint {
    x: f32,
    y: f32,
}

#[derive(Serialize)]
struct UiProfileSize {
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct UiProfileLayout {
    center_band: UiProfileFrame,
    document_region: UiProfileFrame,
    left_region: UiProfileFrame,
    right_region: UiProfileFrame,
    bottom_region: UiProfileFrame,
    status_bar: UiProfileFrame,
}

fn profile_capture_enabled() -> bool {
    env_truthy("ZIRCON_PROFILE_CAPTURE")
}

fn profile_screenshot_capture_enabled() -> bool {
    env_truthy("ZIRCON_PROFILE_CAPTURE_SCREENSHOTS")
}

fn is_forced_softbuffer_screenshot_run() -> bool {
    env_truthy("ZIRCON_PROFILE_FORCE_SOFTBUFFER") && !profile_capture_enabled()
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.as_str(),
                "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
            )
        })
        .unwrap_or(false)
}

fn profile_export_dir() -> Option<PathBuf> {
    let output_root = std::env::var("ZIRCON_PROFILE_OUTPUT_ROOT").ok()?;
    let session_id = std::env::var("ZIRCON_PROFILE_SESSION").unwrap_or_else(|_| "local".into());
    Some(PathBuf::from(output_root).join(sanitize_session_id(&session_id)))
}

fn sanitize_session_id(session_id: &str) -> String {
    session_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
