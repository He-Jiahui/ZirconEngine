use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn align_view_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    let orientation = match control_id {
        "align.pos_x" => "PosX",
        "align.neg_x" => "NegX",
        "align.pos_y" => "PosY",
        "align.neg_y" => "NegY",
        "align.pos_z" => "PosZ",
        "align.neg_z" => "NegZ",
        _ => return None,
    };
    Some(ViewportToolbarPointerRoute::AlignView {
        surface_key: surface_key.to_string(),
        orientation: orientation.to_string(),
    })
}
