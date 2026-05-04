use std::num::NonZeroU32;
use std::sync::Arc;

use slint::Model;
use softbuffer::{Context, Surface};
use winit::window::Window;
use zircon_runtime::diagnostic_log::write_diagnostic_log;

use super::data::HostWindowPresentationData;
use super::painter::paint_host_frame;

pub(super) struct SoftbufferHostPresenter {
    #[allow(dead_code)]
    context: Context<Arc<dyn Window>>,
    surface: Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: (u32, u32),
    present_count: u64,
    last_logged_presentation: Option<String>,
    last_logged_size: Option<(u32, u32)>,
}

impl SoftbufferHostPresenter {
    pub(super) fn new(window: Arc<dyn Window>) -> Result<Self, softbuffer::SoftBufferError> {
        let context = Context::new(window.clone())?;
        let mut surface = Surface::new(&context, window.clone())?;
        let size = current_window_size(window.as_ref());
        resize_surface(&mut surface, size)?;
        Ok(Self {
            context,
            surface,
            size,
            present_count: 0,
            last_logged_presentation: None,
            last_logged_size: None,
        })
    }

    pub(super) fn resize(&mut self, size: (u32, u32)) -> Result<(), softbuffer::SoftBufferError> {
        let size = clamp_size(size);
        resize_surface(&mut self.surface, size)?;
        self.size = size;
        Ok(())
    }

    pub(super) fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
    ) -> Result<(), softbuffer::SoftBufferError> {
        let size = current_window_size(self.surface.window().as_ref());
        if self.size != size {
            self.resize(size)?;
        }

        self.present_count += 1;
        let summary = presentation_summary(presentation);
        if self.present_count <= 8
            || self.last_logged_size != Some(size)
            || self.last_logged_presentation.as_deref() != Some(summary.as_str())
        {
            write_diagnostic_log(
                "editor_host_presenter",
                format!(
                    "present frame={} frame_size={}x{} {}",
                    self.present_count, size.0, size.1, summary
                ),
            );
            self.last_logged_size = Some(size);
            self.last_logged_presentation = Some(summary);
        }
        let frame = paint_host_frame(size.0, size.1, presentation);
        let window = self.surface.window().clone();
        let mut buffer = self.surface.buffer_mut()?;
        buffer.fill(0);
        let pixel_count = (size.0 as usize) * (size.1 as usize);
        for (pixel, rgba) in buffer
            .iter_mut()
            .take(pixel_count)
            .zip(frame.as_bytes().chunks_exact(4))
        {
            let red = rgba[0] as u32;
            let green = rgba[1] as u32;
            let blue = rgba[2] as u32;
            *pixel = (red << 16) | (green << 8) | blue;
        }

        window.pre_present_notify();
        buffer.present()
    }
}

fn presentation_summary(presentation: &HostWindowPresentationData) -> String {
    let layout = &presentation.host_layout;
    let scene = &presentation.host_scene_data;
    format!(
        "project_path={} viewport_label={} status={} center={} document={} viewport={} left={} right={} bottom={} page_tabs={} document_tabs={} left_tabs={} right_tabs={} bottom_tabs={} floating_windows={} document_pane_kind={} left_pane_kind={} right_pane_kind={} bottom_pane_kind={}",
        presentation.host_shell.project_path,
        presentation.host_shell.viewport_label,
        presentation.host_shell.status_secondary,
        frame_summary(&layout.center_band_frame),
        frame_summary(&layout.document_region_frame),
        frame_summary(&layout.viewport_content_frame),
        frame_summary(&layout.left_region_frame),
        frame_summary(&layout.right_region_frame),
        frame_summary(&layout.bottom_region_frame),
        scene.page_chrome.tabs.row_count(),
        scene.document_dock.tabs.row_count(),
        scene.left_dock.tabs.row_count(),
        scene.right_dock.tabs.row_count(),
        scene.bottom_dock.tabs.row_count(),
        scene.floating_layer.floating_windows.row_count(),
        scene.document_dock.pane.kind,
        scene.left_dock.pane.kind,
        scene.right_dock.pane.kind,
        scene.bottom_dock.pane.kind,
    )
}

fn frame_summary(frame: &super::data::FrameRect) -> String {
    format!(
        "{:.1},{:.1},{:.1},{:.1}",
        frame.x, frame.y, frame.width, frame.height
    )
}

fn current_window_size(window: &dyn Window) -> (u32, u32) {
    let size = window.surface_size();
    clamp_size((size.width, size.height))
}

fn resize_surface(
    surface: &mut Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: (u32, u32),
) -> Result<(), softbuffer::SoftBufferError> {
    surface.resize(non_zero(size.0), non_zero(size.1))
}

fn clamp_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value is clamped to non-zero")
}
