use zircon_runtime_interface::ui::event_ui::UiNodeId;

use crate::ui::surface::UiSurface;

// 优先服从 widget 声明的 value alias，回退到已存在的 value/text；不凭动作请求创建不存在的可编辑属性。
pub(super) fn set_value_property(surface: &UiSurface, target: UiNodeId) -> Option<String> {
    let metadata = surface
        .tree
        .nodes
        .get(&target)?
        .template_metadata
        .as_ref()?;
    let attributes = &metadata.attributes;
    if let Some(property) = metadata.widget.value_property.as_deref() {
        return (attributes.contains_key(property)
            || surface
                .component_states
                .get(target)
                .and_then(|state| state.value(property))
                .is_some()
            || metadata.widget.value.is_some())
        .then(|| property.to_string());
    }
    if attributes.contains_key("value") {
        Some("value".to_string())
    } else if attributes.contains_key("text") {
        Some("text".to_string())
    } else {
        None
    }
}
