use super::super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::super::paint_diagnostics::{
    debug_refresh_overlay_frame, presentation_top_bar_frame, union_diagnostic_frames,
};

pub(in crate::ui::retained_host::host_contract) fn damage_with_debug_overlay(
    damage: Option<FrameRect>,
    last_debug_overlay_text: Option<&str>,
    debug_overlay_text: &str,
    size: (u32, u32),
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let damage = damage?;
    if last_debug_overlay_text == Some(debug_overlay_text) {
        return Some(damage);
    }
    let overlay = debug_refresh_overlay_frame(
        &presentation_top_bar_frame(size.0, size.1, presentation),
        debug_overlay_text,
    )?;
    Some(union_diagnostic_frames(&damage, &overlay))
}
