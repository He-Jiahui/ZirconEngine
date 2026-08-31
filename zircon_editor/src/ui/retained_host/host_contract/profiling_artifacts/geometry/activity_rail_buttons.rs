use super::super::super::data::HostWindowSceneData;
use super::super::UiProfileNamedFrame;
use super::pane_frames::collect_activity_rail_buttons;

#[cfg(test)]
#[path = "activity_rail_buttons/capacity_tests.rs"]
mod capacity_tests;

pub(in crate::ui::retained_host::host_contract) fn collect_activity_rail_profile_buttons(
    scene: &HostWindowSceneData,
) -> Vec<UiProfileNamedFrame> {
    let button_capacity = scene
        .left_dock
        .rail_button_frames
        .row_count()
        .saturating_add(scene.right_dock.rail_button_frames.row_count());
    let mut buttons = Vec::with_capacity(button_capacity);
    collect_activity_rail_buttons("left", &scene.left_dock, &mut buttons);
    collect_activity_rail_buttons("right", &scene.right_dock, &mut buttons);
    buttons
}
