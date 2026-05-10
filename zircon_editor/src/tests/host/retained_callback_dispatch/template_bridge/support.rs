use super::super::support::*;

pub(super) fn surface_control_frame(
    surface: &zircon_runtime::ui::surface::UiSurface,
    control_id: &str,
) -> Option<UiFrame> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.layout_cache.frame)
    })
}
