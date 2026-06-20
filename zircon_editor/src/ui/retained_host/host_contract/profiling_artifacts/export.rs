use std::fs;

use super::super::data::HostWindowPresentationData;
use super::super::presenter::{paint_host_presentation_snapshot, HostPresenterBackend};
use super::environment::{
    is_forced_softbuffer_screenshot_run, profile_capture_enabled, profile_export_dir,
    profile_screenshot_capture_enabled,
};
use super::UiProfileGeometry;
use crate::ui::retained_host::primitives::PhysicalSize;

const GEOMETRY_FILE: &str = "ui_profile_geometry.json";
const REFERENCE_SCREENSHOT_FILE: &str = "screenshot_reference.png";

pub(in crate::ui::retained_host::host_contract) fn export_present_artifacts(
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
