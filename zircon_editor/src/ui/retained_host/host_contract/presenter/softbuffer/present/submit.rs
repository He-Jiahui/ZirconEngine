use super::super::super::super::data::FrameRect;
use super::super::surface_io::{copy_rgba_to_softbuffer, softbuffer_damage_rect};
use super::super::SoftbufferHostPresenter;

pub(super) fn submit_presented_frame(
    presenter: &mut SoftbufferHostPresenter,
    damage: Option<&FrameRect>,
    size: (u32, u32),
) -> Result<(), softbuffer::SoftBufferError> {
    let frame = presenter
        .backbuffer
        .as_ref()
        .expect("presenter repaint path always creates a backbuffer");
    let window = presenter.surface.window().clone();
    let mut buffer = presenter.surface.buffer_mut()?;
    {
        zircon_runtime::profile_scope!("editor", "host_presenter", "copy_rgba_to_softbuffer");
        copy_rgba_to_softbuffer(frame, &mut *buffer, damage, size);
    }

    window.pre_present_notify();
    zircon_runtime::profile_scope!("editor", "host_presenter", "softbuffer_present");
    if let Some(damage) = softbuffer_damage_rect(damage, size) {
        buffer.present_with_damage(&[damage])
    } else {
        buffer.present()
    }
}
