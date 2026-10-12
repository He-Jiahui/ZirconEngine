use super::super::super::data::HostWindowSceneData;
use super::shared::tab_route_hit;

pub(in crate::ui::retained_host::host_contract) fn host_page_tab_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    x: f32,
    y: f32,
) -> bool {
    tab_route_hit(&scene.page_chrome.tab_frames, id, x, y, None)
}
