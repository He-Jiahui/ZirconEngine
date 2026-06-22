use super::super::super::data::HostWindowSceneData;
use super::super::UiProfileNamedFrame;
use super::pane_frames::collect_activity_rail_buttons;

pub(in crate::ui::retained_host::host_contract) fn collect_activity_rail_profile_buttons(
    scene: &HostWindowSceneData,
) -> Vec<UiProfileNamedFrame> {
    let mut buttons = Vec::new();
    collect_activity_rail_buttons("left", &scene.left_dock, &mut buttons);
    collect_activity_rail_buttons("right", &scene.right_dock, &mut buttons);
    buttons
}
