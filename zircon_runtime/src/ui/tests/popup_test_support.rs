use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(super) fn assert_popup_node_open(surface: &UiSurface, node_id: UiNodeId, expected: bool) {
    let metadata = surface
        .tree
        .node(node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["popup_open"].as_bool(), Some(expected));
}
