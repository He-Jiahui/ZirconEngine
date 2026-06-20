use super::super::data::HostWindowPresentationData;
use super::{chrome, tabs, template, viewport_toolbar};

pub(in crate::ui::retained_host::host_contract) fn route_contains_profile_frame(
    presentation: &HostWindowPresentationData,
    kind: &str,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
    let scene = &presentation.host_scene_data;
    match kind {
        "resize_splitter" => chrome::resize_splitter_route_hit(scene, surface, x, y),
        "document_tab" => tabs::document_tab_route_hit(scene, id, x, y),
        "drawer_tab" => tabs::drawer_tab_route_hit(scene, id, surface, x, y),
        "floating_tab" => tabs::floating_tab_route_hit(scene, id, surface, x, y),
        "host_page_tab" => tabs::host_page_tab_route_hit(scene, id, x, y),
        "activity_rail_button" => chrome::activity_rail_route_hit(scene, id, surface, x, y),
        "viewport_toolbar_control" => viewport_toolbar::viewport_toolbar_route_hit(scene, id, x, y),
        "template_control" => template::template_route_hit(scene, id, x, y),
        _ => false,
    }
}
