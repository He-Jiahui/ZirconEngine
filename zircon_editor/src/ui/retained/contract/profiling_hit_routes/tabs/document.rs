use super::super::super::data::HostWindowSceneData;
use super::super::geometry::translated;
use super::shared::tab_route_hit;

pub(in crate::ui::retained_host::host_contract) fn document_tab_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    x: f32,
    y: f32,
) -> bool {
    scene.document_surfaces().iter().any(|document| {
        tab_route_hit(
            &document.tab_frames,
            id,
            x,
            y,
            Some(&translated(
                &document.header_frame,
                document.region_frame.x,
                document.region_frame.y,
            )),
        )
    })
}
