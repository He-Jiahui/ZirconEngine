use super::super::super::data::HostWindowSceneData;
use super::super::UiProfileTabFrame;
use super::tabs::{collect_bottom_dock_tabs, collect_floating_window_tabs, collect_side_dock_tabs};

pub(in crate::ui::retained_host::host_contract) fn collect_drawer_tabs(
    scene: &HostWindowSceneData,
) -> Vec<UiProfileTabFrame> {
    let mut drawer_tabs = Vec::new();
    collect_side_dock_tabs("left", &scene.left_dock, &mut drawer_tabs);
    collect_side_dock_tabs("right", &scene.right_dock, &mut drawer_tabs);
    collect_bottom_dock_tabs("bottom", &scene.bottom_dock, &mut drawer_tabs);
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        if let Some(window) = scene.floating_layer.floating_windows.row_data(row) {
            collect_floating_window_tabs(&window, &mut drawer_tabs);
        }
    }
    drawer_tabs
}
