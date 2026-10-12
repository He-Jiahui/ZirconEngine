use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::union::{union_optional_frames, union_visible_frame};

pub(in crate::ui::retained_host::host_contract) fn pane_pointer_press_damage_frame(
    presentation: &HostWindowPresentationData,
    pane_frame: &FrameRect,
    extra_damage: Option<FrameRect>,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let mut damage = None;

    // Pane button callbacks can update the active pane, sibling pane chrome,
    // or status text. Keep menu/title chrome outside the repaint.
    damage = union_visible_frame(damage, pane_frame.clone());
    damage = union_visible_frame(damage, presentation.host_layout.center_band_frame.clone());
    damage = union_visible_frame(damage, scene.layout.center_band_frame.clone());
    damage = union_visible_frame(damage, presentation.host_layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.layout.status_bar_frame.clone());
    damage = union_visible_frame(damage, scene.status_bar.status_bar_frame.clone());

    union_optional_frames(damage, extra_damage)
}
