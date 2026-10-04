use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::event_ui::UiNodeId;

/// 供菜单行为测试核对声明式 popup_open 状态；调用前节点必须仍在 surface 树中并带模板元数据。
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
