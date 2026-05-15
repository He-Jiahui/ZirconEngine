use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

pub(super) fn resize_damage_frame(presentation: &HostWindowPresentationData) -> Option<FrameRect> {
    let frame = presentation.host_layout.center_band_frame.clone();
    visible_frame(&frame).then_some(frame)
}

fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}
