mod entry;
mod status;
mod target;
mod union;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

pub(in crate::ui::retained_host::host_contract) use self::entry::viewport_toolbar_press_damage_frame;

pub(in crate::ui::retained_host::host_contract) fn viewport_chrome_damage_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    status::center_band_status_damage_frame(presentation)
}

pub(in crate::ui::retained_host::host_contract) fn native_viewport_chrome_damage_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let surface = &presentation.native_floating_surface_data;
    let bounds = &surface.native_window_bounds;
    let header_height = surface.header_height_px.clamp(0.0, bounds.height.max(0.0));
    let damage = FrameRect {
        x: 0.0,
        y: header_height,
        width: bounds.width,
        height: bounds.height - header_height,
    };
    (damage.x.is_finite()
        && damage.y.is_finite()
        && damage.width.is_finite()
        && damage.height.is_finite()
        && damage.width > 0.0
        && damage.height > 0.0)
        .then_some(damage)
}
