use super::*;

pub(super) fn componentized_workbench_projection_fixture() -> (
    BuiltinWorkbenchWindowTemplateSurfaceBridge,
    ModelRc<TemplatePaneNodeData>,
) {
    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    (bridge, nodes)
}

pub(super) fn round_to_layout_pixel(value: f32) -> f32 {
    value.round()
}

pub(super) fn numbered_scene_entries(count: usize, selected_index: usize) -> SceneEntries {
    SceneEntries::from_entries(
        (0..count)
            .map(|index| SceneEntry {
                id: (index + 1) as u64,
                name: format!("SceneNode_{:02}", index + 1),
                depth: if index == 0 { 0 } else { 1 + index % 3 },
            })
            .collect::<Vec<_>>(),
        [(selected_index + 1) as u64],
    )
}

pub(super) fn template_contract_node<'a>(
    nodes: &'a ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> &'a TemplatePaneNodeData {
    (0..nodes.row_count())
        .filter_map(|row| nodes.get(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to the host contract"))
}

pub(super) fn template_contract_option(
    options: &ModelRc<TemplatePaneOptionData>,
    row: usize,
) -> TemplatePaneOptionData {
    options
        .row_data(row)
        .unwrap_or_else(|| panic!("structured option row {row} should be projected"))
}

pub(super) fn template_contract_menu_item(
    items: &ModelRc<TemplatePaneMenuItemData>,
    row: usize,
) -> TemplatePaneMenuItemData {
    items
        .row_data(row)
        .unwrap_or_else(|| panic!("structured menu item row {row} should be projected"))
}

pub(super) fn control_bool(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> bool {
    bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
                .and_then(|metadata| metadata.attributes.get(property))
                .and_then(toml::Value::as_bool)
        })
        .unwrap_or(false)
}

pub(super) fn control_string(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<String> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))
            .and_then(toml::Value::as_str)
            .map(str::to_string)
    })
}

pub(super) fn control_has_class(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    class_name: &str,
) -> bool {
    bridge.surface().tree.nodes.values().any(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .is_some_and(|metadata| {
                metadata
                    .classes
                    .iter()
                    .any(|class| class.as_str() == class_name)
            })
    })
}

pub(super) fn control_integer(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<i64> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))
            .and_then(toml::Value::as_integer)
    })
}

pub(super) fn control_attribute<'a>(
    bridge: &'a BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<&'a toml::Value> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))
    })
}

pub(super) fn assert_virtual_row_repeat(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    prototype_control_id: &str,
    virtual_control_prefix: &str,
    authored_count: i64,
    node_path_namespace: &str,
) {
    let repeat = control_attribute(bridge, control_id, UI_V2_REPEAT_ATTRIBUTE)
        .and_then(toml::Value::as_table)
        .unwrap_or_else(|| panic!("{control_id} should expose a repeat declaration"));
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_KIND)
            .and_then(toml::Value::as_str),
        Some(UI_V2_REPEAT_KIND_VIRTUAL_ROWS)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_PROTOTYPE)
            .and_then(toml::Value::as_str),
        Some(prototype_control_id)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX)
            .and_then(toml::Value::as_str),
        Some(virtual_control_prefix)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_AUTHORED_COUNT)
            .and_then(toml::Value::as_integer),
        Some(authored_count)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE)
            .and_then(toml::Value::as_str),
        Some(node_path_namespace)
    );
}

pub(super) fn control_center(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> UiPoint {
    let frame = bridge
        .control_frame(control_id)
        .unwrap_or_else(|| panic!("{control_id} should have a frame"));
    UiPoint::new(frame.x + frame.width * 0.5, frame.y + frame.height * 0.5)
}

pub(super) fn control_component_focused(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> bool {
    let Some(node_id) = bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .map(|_| node.node_id)
    }) else {
        return false;
    };
    bridge.surface().focus.focused == Some(node_id)
        && bridge
            .surface()
            .component_state(node_id)
            .is_some_and(|state| state.flags.focused)
}

pub(super) fn control_visibility(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> Option<UiVisibility> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .map(|_| node.visibility)
    })
}

pub(super) fn slot_padding_for_control(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> Option<UiMargin> {
    let node_id = bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .map(|_| node.node_id)
    })?;
    bridge
        .surface()
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == node_id)
        .map(|slot| slot.padding)
}

pub(super) fn render_background_for_control(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> Option<String> {
    let node_id = bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .map(|_| node.node_id)
    })?;
    bridge
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .find_map(|command| {
            (command.node_id == node_id)
                .then(|| command.style.background_color.clone())
                .flatten()
        })
}

pub(super) fn render_border_for_control(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> Option<String> {
    let node_id = bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .map(|_| node.node_id)
    })?;
    bridge
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .find_map(|command| {
            (command.node_id == node_id)
                .then(|| command.style.border_color.clone())
                .flatten()
        })
}

pub(super) fn style_color_u8(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Role(_) | UiStyleColor::Inherit => None,
    }
}
