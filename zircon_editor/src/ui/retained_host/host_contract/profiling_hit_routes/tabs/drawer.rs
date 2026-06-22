use super::super::super::data::HostWindowSceneData;
use super::super::geometry::translated;
use super::shared::tab_route_hit;

pub(in crate::ui::retained_host::host_contract) fn drawer_tab_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
    (surface == "left"
        && tab_route_hit(
            &scene.left_dock.tab_frames,
            id,
            x,
            y,
            Some(&translated(
                &scene.left_dock.header_frame,
                scene.left_dock.region_frame.x,
                scene.left_dock.region_frame.y,
            )),
        ))
        || (surface == "right"
            && tab_route_hit(
                &scene.right_dock.tab_frames,
                id,
                x,
                y,
                Some(&translated(
                    &scene.right_dock.header_frame,
                    scene.right_dock.region_frame.x,
                    scene.right_dock.region_frame.y,
                )),
            ))
        || (surface == "bottom"
            && tab_route_hit(
                &scene.bottom_dock.tab_frames,
                id,
                x,
                y,
                Some(&translated(
                    &scene.bottom_dock.header_frame,
                    scene.bottom_dock.region_frame.x,
                    scene.bottom_dock.region_frame.y,
                )),
            ))
}
