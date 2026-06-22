use super::super::super::data::HostWindowSceneData;
use super::super::geometry::translated;
use super::shared::tab_route_hit;

pub(in crate::ui::retained_host::host_contract) fn document_tab_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    x: f32,
    y: f32,
) -> bool {
    tab_route_hit(
        &scene.document_dock.tab_frames,
        id,
        x,
        y,
        Some(&translated(
            &scene.document_dock.header_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        )),
    )
}
