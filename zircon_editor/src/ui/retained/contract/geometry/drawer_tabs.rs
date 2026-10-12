use super::super::super::componentized_workbench_regions::owns_ordinary_panes;
use super::super::super::data::{HostWindowPresentationData, HostWindowSceneData};
use super::super::UiProfileTabFrame;
use super::tabs::{collect_bottom_dock_tabs, collect_floating_window_tabs, collect_side_dock_tabs};

pub(in crate::ui::retained_host::host_contract) fn collect_drawer_tabs(
    presentation: &HostWindowPresentationData,
) -> Vec<UiProfileTabFrame> {
    let scene = &presentation.host_scene_data;
    let mut drawer_tabs = Vec::with_capacity(drawer_tab_capacity(scene));
    if !owns_ordinary_panes(presentation) {
        collect_side_dock_tabs("left", &scene.left_dock, &mut drawer_tabs);
        collect_side_dock_tabs("right", &scene.right_dock, &mut drawer_tabs);
    }
    collect_bottom_dock_tabs("bottom", &scene.bottom_dock, &mut drawer_tabs);
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        if let Some(window) = scene.floating_layer.floating_windows.row_data(row) {
            collect_floating_window_tabs(&window, &mut drawer_tabs);
        }
    }
    drawer_tabs
}

fn drawer_tab_capacity(scene: &HostWindowSceneData) -> usize {
    drawer_tab_capacity_from_rows(
        scene.left_dock.tab_frames.row_count(),
        scene.right_dock.tab_frames.row_count(),
        scene.bottom_dock.tab_frames.row_count(),
        scene
            .floating_layer
            .floating_windows
            .iter()
            .map(|window| window.tab_frames.row_count()),
    )
}

fn drawer_tab_capacity_from_rows<I>(left: usize, right: usize, bottom: usize, floating: I) -> usize
where
    I: IntoIterator<Item = usize>,
{
    floating.into_iter().fold(
        drawer_dock_tab_capacity_from_rows(left, right, bottom),
        usize::saturating_add,
    )
}

const fn drawer_dock_tab_capacity_from_rows(left: usize, right: usize, bottom: usize) -> usize {
    left.saturating_add(right).saturating_add(bottom)
}

#[cfg(test)]
#[path = "drawer_tabs/tests/capacity.rs"]
mod capacity_tests;
