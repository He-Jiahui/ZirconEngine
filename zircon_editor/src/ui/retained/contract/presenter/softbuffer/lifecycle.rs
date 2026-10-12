use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::window::Window;

use super::super::super::diagnostics::HostRefreshDiagnostics;
use super::backbuffer::capture_native_resize_snapshot;
use super::surface_io::{clamp_size, current_window_size, resize_surface};
use super::SoftbufferHostPresenter;

pub(in crate::ui::retained_host::host_contract) fn new_presenter(
    window: Arc<dyn Window>,
) -> Result<SoftbufferHostPresenter, softbuffer::SoftBufferError> {
    let context = Context::new(window.clone())?;
    let mut surface = Surface::new(&context, window.clone())?;
    let size = current_window_size(window.as_ref());
    resize_surface(&mut surface, size)?;
    Ok(SoftbufferHostPresenter {
        context,
        surface,
        size,
        backbuffer: None,
        native_resize_snapshot: None,
        diagnostics: HostRefreshDiagnostics::default(),
        last_debug_overlay_text: None,
        last_logged_presentation: None,
        last_logged_size: None,
    })
}

pub(in crate::ui::retained_host::host_contract) fn resize_presenter(
    presenter: &mut SoftbufferHostPresenter,
    size: (u32, u32),
) -> Result<(), softbuffer::SoftBufferError> {
    let size = clamp_size(size);
    resize_surface(&mut presenter.surface, size)?;
    if capture_native_resize_snapshot(
        &mut presenter.native_resize_snapshot,
        &mut presenter.backbuffer,
    ) {
        zircon_runtime::profile_counter!(
            "editor",
            "ui.window_resize.softbuffer_snapshot_capture_count",
            1_u8
        );
    }
    presenter.size = size;
    presenter.last_debug_overlay_text = None;
    Ok(())
}
