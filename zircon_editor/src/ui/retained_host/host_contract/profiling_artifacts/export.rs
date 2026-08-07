use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::mpsc::{self, SyncSender};

use super::super::data::HostWindowPresentationData;
use super::super::presenter::{HostPresenterBackend, paint_host_presentation_snapshot};
use super::UiProfileGeometry;
use super::environment::{
    is_forced_softbuffer_screenshot_run, profile_capture_enabled, profile_export_dir,
    profile_screenshot_capture_enabled,
};
use crate::ui::retained_host::primitives::PhysicalSize;

const GEOMETRY_FILE: &str = "ui_profile_geometry.json";
const REFERENCE_SCREENSHOT_FILE: &str = "screenshot_reference.png";
const ARTIFACT_QUEUE_CAPACITY: usize = 1;

struct PresentArtifactExport {
    export_dir: PathBuf,
    geometry: UiProfileGeometry,
    screenshot: Option<ProfileScreenshot>,
}

struct ProfileScreenshot {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

static ARTIFACT_EXPORT_SENDER: OnceLock<Option<SyncSender<PresentArtifactExport>>> =
    OnceLock::new();

pub(in crate::ui::retained_host::host_contract) fn queue_present_artifacts(
    presentation: &HostWindowPresentationData,
    size: &PhysicalSize,
    backend: HostPresenterBackend,
) -> bool {
    if !profile_capture_enabled() {
        return false;
    }
    let Some(export_dir) = profile_export_dir() else {
        return false;
    };
    if is_forced_softbuffer_screenshot_run() {
        return false;
    }

    let geometry = UiProfileGeometry::from_presentation(presentation, size, backend);
    let screenshot = profile_screenshot_capture_enabled().then(|| {
        let frame = paint_host_presentation_snapshot(size.width, size.height, presentation);
        ProfileScreenshot {
            width: frame.width(),
            height: frame.height(),
            rgba: frame.into_bytes(),
        }
    });
    let export = PresentArtifactExport {
        export_dir,
        geometry,
        screenshot,
    };
    artifact_export_sender().is_some_and(|sender| sender.try_send(export).is_ok())
}

fn artifact_export_sender() -> Option<&'static SyncSender<PresentArtifactExport>> {
    ARTIFACT_EXPORT_SENDER
        .get_or_init(|| {
            let (sender, receiver) = mpsc::sync_channel(ARTIFACT_QUEUE_CAPACITY);
            std::thread::Builder::new()
                .name("zircon-ui-profile-export".into())
                .spawn(move || {
                    while let Ok(export) = receiver.recv() {
                        write_present_artifacts(export);
                    }
                })
                .ok()
                .map(|_| sender)
        })
        .as_ref()
}

fn write_present_artifacts(export: PresentArtifactExport) {
    if fs::create_dir_all(&export.export_dir).is_err() {
        return;
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(&export.geometry) {
        let _ = fs::write(export.export_dir.join(GEOMETRY_FILE), bytes);
    }
    if let Some(screenshot) = export.screenshot {
        let _ = image::save_buffer_with_format(
            export.export_dir.join(REFERENCE_SCREENSHOT_FILE),
            &screenshot.rgba,
            screenshot.width,
            screenshot.height,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        );
    }
}
