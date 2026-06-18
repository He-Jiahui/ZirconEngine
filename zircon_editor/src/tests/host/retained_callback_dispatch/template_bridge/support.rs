use super::super::support::*;
use zircon_runtime_interface::ui::tree::UiVisibility;

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

pub(super) fn control_center(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> UiPoint {
    let frame = bridge
        .control_frame(control_id)
        .unwrap_or_else(|| panic!("{control_id} should have a frame"));
    UiPoint::new(frame.x + frame.width * 0.5, frame.y + frame.height * 0.5)
}

pub(super) fn control_float(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<f64> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        let value = node
            .template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))?;
        match value {
            toml::Value::Float(value) => Some(*value),
            toml::Value::Integer(value) => Some(*value as f64),
            toml::Value::String(value) => value.parse::<f64>().ok(),
            _ => None,
        }
    })
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

pub(super) fn control_component_pressed(
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
    bridge
        .surface()
        .component_state(node_id)
        .is_some_and(|state| state.flags.pressed)
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
